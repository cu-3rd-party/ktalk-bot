use std::sync::{Arc, Mutex};
use std::time::Duration;

use futures_util::{SinkExt, StreamExt};
use http::Request;
use serde_json::{Value, json};
use tokio::runtime::Runtime;
use tokio::time::{Instant, interval};
use tokio_tungstenite::{connect_async, tungstenite::Message};
use uuid::Uuid;

use crate::domain::auth::CookieBundle;
use crate::domain::bot::{JoinRoomReport, ParticipantSnapshot, RoomConnection, UserProfile};
use crate::domain::room::RoomLink;
use crate::error::{KTalkError, Result};
use crate::infrastructure::http::ktalk_http_client::KTalkHttpClient;
use crate::infrastructure::parsing::xmpp::parse_participant_presence;

#[derive(Debug, Clone)]
pub struct KTalkBotEngine {
    http_client: KTalkHttpClient,
    cookies: Arc<Mutex<CookieBundle>>,
}

impl KTalkBotEngine {
    pub fn new(cookie_header: &str, base_url: impl Into<String>) -> Result<Self> {
        let cookies = CookieBundle::parse(cookie_header)?;
        Ok(Self {
            http_client: KTalkHttpClient::with_base_url(base_url)?,
            cookies: Arc::new(Mutex::new(cookies)),
        })
    }

    pub fn renew_cookies(&self) -> Result<UserProfile> {
        let mut cookies = self.cookies.lock().expect("engine cookie lock poisoned");
        self.http_client.bootstrap(&mut cookies)
    }

    pub fn fetch_history(
        &self,
        max_pages: usize,
        page_size: usize,
    ) -> Result<Vec<crate::domain::history::ConferenceHistoryRecord>> {
        let mut cookies = self.cookies.lock().expect("engine cookie lock poisoned");
        self.http_client
            .fetch_all_history(&mut cookies, max_pages, page_size)
    }

    pub fn join_room(&self, link: &str, duration: Duration) -> Result<JoinRoomReport> {
        let room_link = RoomLink::parse(link)?;
        let room_name = room_link.short_name()?.as_str().to_owned();
        let room_client = self.client_for_room_link(&room_link)?;
        let (profile, room, cookie_header, session_token, base_url) =
            self.bootstrap_room(&room_client, &room_name)?;

        room_client.send_activity(&room_name, &mut self.cookies.lock().unwrap())?;

        Runtime::new()?.block_on(run_join_flow(
            base_url,
            cookie_header,
            session_token,
            profile,
            room,
            duration,
        ))
    }

    pub fn record_participants(
        &self,
        link: &str,
        duration: Duration,
    ) -> Result<Vec<ParticipantSnapshot>> {
        let report = self.join_room(link, duration)?;
        Ok(report.participants)
    }

    pub fn send_chat_message(&self, link: &str, text: &str) -> Result<()> {
        let room_link = RoomLink::parse(link)?;
        let room_name = room_link.short_name()?.as_str().to_owned();
        let room_client = self.client_for_room_link(&room_link)?;
        let (profile, _room, cookie_header, session_token, base_url) =
            self.bootstrap_room(&room_client, &room_name)?;

        Runtime::new()?.block_on(send_chat_message_inner(
            &base_url,
            &cookie_header,
            session_token.as_str(),
            &profile.user_id,
            &room_name,
            text,
        ))
    }

    pub fn play_audio_on_mic(
        &self,
        _link: &str,
        _audio_path: &str,
        _duration: Duration,
    ) -> Result<()> {
        Err(KTalkError::UnsupportedAudioPublishing)
    }

    fn bootstrap_room(
        &self,
        http_client: &KTalkHttpClient,
        room_name: &str,
    ) -> Result<(
        UserProfile,
        RoomConnection,
        String,
        crate::domain::auth::SessionToken,
        String,
    )> {
        let mut cookies = self.cookies.lock().expect("engine cookie lock poisoned");
        let profile = http_client.bootstrap(&mut cookies)?;
        let conference_id = http_client.resolve_room(room_name, &mut cookies)?;
        let room = RoomConnection {
            room_name: room_name.to_owned(),
            conference_id,
        };
        let session_token = cookies.session_token()?;
        let cookie_header = cookies.as_cookie_header();
        Ok((
            profile,
            room,
            cookie_header,
            session_token,
            http_client.base_url().to_owned(),
        ))
    }

    fn client_for_room_link(&self, room_link: &RoomLink) -> Result<KTalkHttpClient> {
        let link_base_url = room_link.base_url()?;
        if link_base_url == self.http_client.base_url() {
            Ok(self.http_client.clone())
        } else {
            KTalkHttpClient::with_base_url(link_base_url)
        }
    }
}

async fn run_join_flow(
    base_url: String,
    cookie_header: String,
    session_token: crate::domain::auth::SessionToken,
    profile: UserProfile,
    room: RoomConnection,
    duration: Duration,
) -> Result<JoinRoomReport> {
    let system = maintain_system_ws(
        &base_url,
        &cookie_header,
        session_token.as_str(),
        &profile.user_id,
        &room.room_name,
        duration,
    );
    let xmpp = capture_xmpp_presence(
        &base_url,
        &cookie_header,
        session_token.as_str(),
        &profile,
        &room,
        duration,
    );

    let (system_result, xmpp_result) = tokio::join!(system, xmpp);
    system_result?;
    let (joined, participants) = xmpp_result?;

    Ok(JoinRoomReport {
        room_name: room.room_name,
        conference_id: room.conference_id,
        joined,
        participants,
    })
}

async fn maintain_system_ws(
    base_url: &str,
    cookie_header: &str,
    session_token: &str,
    user_id: &str,
    room_name: &str,
    duration: Duration,
) -> Result<()> {
    let ws_url = base_url
        .replace("https://", "wss://")
        .replace("http://", "ws://")
        + "/system/ws";
    let request = Request::builder()
        .uri(ws_url)
        .header("Origin", base_url)
        .header("Cookie", cookie_header)
        .header("User-Agent", "ktalk-bot/0.1")
        .body(())?;
    let (mut socket, _) = connect_async(request).await?;

    socket
        .send(Message::Text(
            json!({
                "a": "connect",
                "reqId": generate_request_id(),
                "data": {"signInToken": session_token, "clientType": "Web", "webAppVersion": "master"}
            })
            .to_string()
            .into(),
        ))
        .await?;

    let session_id = await_system_session_id(&mut socket).await?;
    for payload in [
        json!({"a":"message_subscribe","reqId":generate_request_id(),"data":{"topic":"personal"},"session":session_id}),
        json!({"a":"user_status","reqId":generate_request_id(),"data":{"userKey":user_id,"status":"inMeeting"},"session":session_id}),
        json!({"a":"chat_join","reqId":generate_request_id(),"data":{"name":room_name,"popup":false,"platform":"web"},"session":session_id}),
    ] {
        socket
            .send(Message::Text(payload.to_string().into()))
            .await?;
    }

    let deadline = Instant::now() + duration;
    let mut ticker = interval(Duration::from_secs(20));
    loop {
        tokio::select! {
            _ = ticker.tick() => {
                if Instant::now() >= deadline {
                    break;
                }
                for payload in [
                    json!({"a":"ping","reqId":generate_request_id(),"data":{},"session":session_id}),
                    json!({"a":"chat_ping","reqId":generate_request_id(),"data":{"name":room_name},"session":session_id}),
                ] {
                    socket.send(Message::Text(payload.to_string().into())).await?;
                }
            }
            incoming = socket.next() => {
                match incoming {
                    Some(Ok(_)) => {}
                    Some(Err(error)) => return Err(error.into()),
                    None => break,
                }
            }
        }
    }

    Ok(())
}

async fn send_chat_message_inner(
    base_url: &str,
    cookie_header: &str,
    session_token: &str,
    user_id: &str,
    room_name: &str,
    text: &str,
) -> Result<()> {
    let ws_url = base_url
        .replace("https://", "wss://")
        .replace("http://", "ws://")
        + "/system/ws";
    let request = Request::builder()
        .uri(ws_url)
        .header("Origin", base_url)
        .header("Cookie", cookie_header)
        .header("User-Agent", "ktalk-bot/0.1")
        .body(())?;
    let (mut socket, _) = connect_async(request).await?;

    socket
        .send(Message::Text(
            json!({
                "a": "connect",
                "reqId": generate_request_id(),
                "data": {"signInToken": session_token, "clientType": "Web", "webAppVersion": "master"}
            })
            .to_string()
            .into(),
        ))
        .await?;
    let session_id = await_system_session_id(&mut socket).await?;

    for payload in [
        json!({"a":"message_subscribe","reqId":generate_request_id(),"data":{"topic":"personal"},"session":session_id}),
        json!({"a":"user_status","reqId":generate_request_id(),"data":{"userKey":user_id,"status":"inMeeting"},"session":session_id}),
        json!({"a":"chat_join","reqId":generate_request_id(),"data":{"name":room_name,"popup":false,"platform":"web"},"session":session_id}),
        json!({"a":"chat_send","reqId":generate_request_id(),"data":{"name":room_name,"text":text,"platform":"web"},"session":session_id}),
    ] {
        socket
            .send(Message::Text(payload.to_string().into()))
            .await?;
    }

    Ok(())
}

async fn await_system_session_id<S>(socket: &mut S) -> Result<String>
where
    S: StreamExt<Item = std::result::Result<Message, tokio_tungstenite::tungstenite::Error>>
        + Unpin,
{
    while let Some(message) = socket.next().await {
        let message = message?;
        let text = match message {
            Message::Text(text) => text.to_string(),
            Message::Binary(data) => String::from_utf8_lossy(&data).into_owned(),
            _ => continue,
        };
        let value: Value = serde_json::from_str(&text)?;
        if let Some(session_id) = value
            .get("data")
            .and_then(|data| data.get("sessionId"))
            .and_then(Value::as_str)
        {
            return Ok(session_id.to_owned());
        }
    }

    Err(KTalkError::Protocol(
        "system websocket did not return a sessionId".to_owned(),
    ))
}

async fn capture_xmpp_presence(
    base_url: &str,
    cookie_header: &str,
    session_token: &str,
    profile: &UserProfile,
    room: &RoomConnection,
    duration: Duration,
) -> Result<(bool, Vec<ParticipantSnapshot>)> {
    let ws_url = format!(
        "{}{}",
        base_url
            .replace("https://", "wss://")
            .replace("http://", "ws://"),
        format!(
            "/jitsi/xmpp-websocket?room={}&sessionToken={}",
            room.conference_id, session_token
        )
    );
    let request = Request::builder()
        .uri(ws_url)
        .header("Origin", base_url)
        .header("Cookie", cookie_header)
        .header("User-Agent", "ktalk-bot/0.1")
        .header("Sec-WebSocket-Protocol", "xmpp")
        .body(())?;
    let (mut socket, _) = connect_async(request).await?;
    let system_nick = Uuid::new_v4().simple().to_string()[..8].to_owned();
    let mut handled_stanzas = 0usize;
    let mut state = XmppState::Auth;
    let mut joined = false;
    let mut participants = Vec::<ParticipantSnapshot>::new();
    let deadline = Instant::now() + duration;
    let mut ping_interval = interval(Duration::from_secs(10));

    socket.send(Message::Text(
        "<open to=\"meet.jitsi\" version=\"1.0\" xmlns=\"urn:ietf:params:xml:ns:xmpp-framing\"/>"
            .into(),
    )).await?;

    loop {
        tokio::select! {
            _ = ping_interval.tick() => {
                if Instant::now() >= deadline {
                    break;
                }
                if matches!(state, XmppState::Joined) {
                    let ping = format!(
                        "<iq id=\"{}:sendIQ\" to=\"meet.jitsi\" type=\"get\" xmlns=\"jabber:client\"><ping xmlns=\"urn:xmpp:ping\"/></iq>",
                        Uuid::new_v4()
                    );
                    socket.send(Message::Text(ping.into())).await?;
                }
            }
            incoming = socket.next() => {
                let Some(incoming) = incoming else { break; };
                let incoming = incoming?;
                let text = match incoming {
                    Message::Text(text) => text.to_string(),
                    Message::Binary(data) => String::from_utf8_lossy(&data).into_owned(),
                    _ => continue,
                };

                if text.starts_with("<iq") || text.starts_with("<presence") || text.starts_with("<message") {
                    handled_stanzas += 1;
                }

                if let Some(snapshot) = parse_participant_presence(&text) {
                    if !participants.iter().any(|existing| existing.occupant_id == snapshot.occupant_id) {
                        participants.push(snapshot);
                    }
                }

                match state {
                    XmppState::Auth if text.contains("ANONYMOUS") => {
                        socket.send(Message::Text(
                            "<auth mechanism=\"ANONYMOUS\" xmlns=\"urn:ietf:params:xml:ns:xmpp-sasl\"/>".into(),
                        )).await?;
                        state = XmppState::WaitSuccess;
                    }
                    XmppState::WaitSuccess if text.contains("success") => {
                        socket.send(Message::Text(
                            "<open to=\"meet.jitsi\" version=\"1.0\" xmlns=\"urn:ietf:params:xml:ns:xmpp-framing\"/>".into(),
                        )).await?;
                        state = XmppState::Bind;
                    }
                    XmppState::Bind if text.contains("urn:ietf:params:xml:ns:xmpp-bind") => {
                        socket.send(Message::Text(
                            "<iq type=\"set\" id=\"bind_1\" xmlns=\"jabber:client\"><bind xmlns=\"urn:ietf:params:xml:ns:xmpp-bind\"/></iq>".into(),
                        )).await?;
                        state = XmppState::EnableSm;
                    }
                    XmppState::EnableSm if text.contains("bind_1") => {
                        socket.send(Message::Text("<enable xmlns=\"urn:xmpp:sm:3\" resume=\"false\"/>".into())).await?;
                        let user_info = json!({
                            "key": profile.user_id,
                            "firstName": profile.first_name,
                            "lastName": profile.last_name,
                            "middleName": "",
                            "isKiosk": false
                        }).to_string().replace('"', "&quot;");
                        let source_info = json!({
                            format!("{}-a0", system_nick): {"muted": true},
                            format!("{}-v0", system_nick): {"muted": true}
                        }).to_string().replace('"', "&quot;");
                        let presence = format!(
                            "<presence to=\"{}@muc.meet.jitsi/{}\" xmlns=\"jabber:client\"><x xmlns=\"http://jabber.org/protocol/muc\"/><audiomuted>true</audiomuted><videomuted>true</videomuted><stats-id>ktalk-bot</stats-id><c hash=\"sha-1\" node=\"https://jitsi.org/jitsi-meet\" ver=\"+mpajJhafj8jFogLBKsPbQfMgzU=\" xmlns=\"http://jabber.org/protocol/caps\"/><jitsi_participant_codecList>vp9,vp8,h264,av1</jitsi_participant_codecList><nick xmlns=\"http://jabber.org/protocol/nick\">{}</nick><jitsi_participant_user-info>{}</jitsi_participant_user-info><SourceInfo>{}</SourceInfo><jitsi_participant_video-size>{{&quot;width&quot;:960,&quot;height&quot;:720}}</jitsi_participant_video-size></presence>",
                            room.conference_id,
                            system_nick,
                            profile.first_name,
                            user_info,
                            source_info
                        );
                        socket.send(Message::Text(presence.into())).await?;
                        state = XmppState::Joined;
                    }
                    _ => {}
                }

                if matches!(state, XmppState::Joined)
                    && text.contains(&format!("/{system_nick}"))
                    && text.contains("status code='110'")
                {
                    joined = true;
                }

                if text.contains("<r xmlns") {
                    let ack = format!("<a xmlns='urn:xmpp:sm:3' h='{handled_stanzas}'/>");
                    socket.send(Message::Text(ack.into())).await?;
                }

                if text.contains("urn:xmpp:ping") && text.contains("<ping") {
                    if let Some(id) = extract_attribute(&text, "id") {
                        let response = format!("<iq type=\"result\" id=\"{id}\" xmlns=\"jabber:client\"/>");
                        socket.send(Message::Text(response.into())).await?;
                    }
                }

                if matches!(state, XmppState::Joined) && text.contains("<iq") && text.contains("type=\"set\"") {
                    if let (Some(iq_id), Some(from)) = (extract_attribute(&text, "id"), extract_attribute(&text, "from")) {
                        if from.contains("focus") {
                            let response = format!("<iq type=\"result\" id=\"{iq_id}\" to=\"{from}\" xmlns=\"jabber:client\"/>");
                            socket.send(Message::Text(response.into())).await?;
                        }
                    }
                }

                if Instant::now() >= deadline {
                    break;
                }
            }
        }
    }

    Ok((joined, participants))
}

#[derive(Debug, Clone, Copy)]
enum XmppState {
    Auth,
    WaitSuccess,
    Bind,
    EnableSm,
    Joined,
}

fn generate_request_id() -> String {
    Uuid::new_v4()
        .simple()
        .to_string()
        .chars()
        .take(10)
        .collect()
}

fn extract_attribute(text: &str, name: &str) -> Option<String> {
    let pattern = format!("{name}=\"");
    let start = text.find(&pattern)? + pattern.len();
    let rest = &text[start..];
    let end = rest.find('"')?;
    Some(rest[..end].to_owned())
}
