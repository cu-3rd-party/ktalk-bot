use std::sync::Mutex;
use std::time::Duration;

use napi::bindgen_prelude::Result as NapiResult;
use napi::{Error as NapiError, Status};
use napi_derive::napi;

use crate::application::bot_engine::KTalkBotEngine;
use crate::application::history::{FetchConferenceHistory, FetchConferenceHistoryInput};
use crate::domain::bot::{JoinRoomReport, ParticipantSnapshot, UserProfile};
use crate::domain::history::{ConferenceHistoryRecord, Participant, Recording};
use crate::error::KTalkError;

#[napi(object)]
pub struct UserProfileResult {
    pub user_id: String,
    pub first_name: String,
    pub last_name: String,
}

#[napi(object)]
pub struct ParticipantDetailResult {
    pub kind: String,
    pub display_name: String,
}

#[napi(object)]
pub struct RecordingResult {
    pub recording_id: String,
    pub playback_url: String,
}

#[napi(object)]
pub struct HistoryRecordResult {
    pub key: Option<String>,
    pub room_name: String,
    pub title: String,
    pub start_time: Option<String>,
    pub end_time: Option<String>,
    pub participants_count: u32,
    pub participants: Vec<String>,
    pub participant_details: Vec<ParticipantDetailResult>,
    pub has_recording: bool,
    pub recording_id: Option<String>,
    pub recording_url: Option<String>,
    pub recording: Option<RecordingResult>,
}

#[napi(object)]
pub struct ParticipantSnapshotResult {
    pub occupant_id: String,
    pub display_name: String,
    pub user_id: Option<String>,
}

#[napi(object)]
pub struct JoinRoomResult {
    pub room_name: String,
    pub conference_id: String,
    pub joined: bool,
    pub participants: Vec<ParticipantSnapshotResult>,
}

#[napi]
pub struct KTalkClient {
    cookie_header: String,
    base_url: String,
    session_token: Option<String>,
    room_link: Mutex<Option<String>>,
}

#[napi]
impl KTalkClient {
    #[napi(constructor)]
    pub fn new(
        cookie_header: String,
        base_url: Option<String>,
        room_link: Option<String>,
        session_token: Option<String>,
    ) -> Self {
        let base_url = base_url.unwrap_or_else(|| "https://centraluniversity.ktalk.ru".to_owned());
        Self {
            cookie_header,
            base_url,
            session_token,
            room_link: Mutex::new(room_link),
        }
    }

    #[napi]
    pub fn bind_room(&self, link: String) -> NapiResult<()> {
        crate::domain::room::RoomLink::parse(&link).map_err(to_napi_error)?;
        *self.room_link.lock().expect("room link lock poisoned") = Some(link);
        Ok(())
    }

    #[napi]
    pub fn current_room(&self) -> Option<String> {
        self.room_link
            .lock()
            .expect("room link lock poisoned")
            .clone()
    }

    #[napi]
    pub async fn renew_cookies(&self) -> NapiResult<UserProfileResult> {
        let cookie_header = self.cookie_header.clone();
        let base_url = self.base_url.clone();
        let session_token = self.session_token.clone();

        let profile = tokio::task::spawn_blocking(move || {
            KTalkBotEngine::new(&cookie_header, &base_url, session_token.as_deref())?.renew_cookies()
        })
        .await
        .map_err(join_error)?
        .map_err(to_napi_error)?;

        Ok(profile.into())
    }

    #[napi]
    pub async fn get_history(
        &self,
        max_pages: Option<u32>,
        page_size: Option<u32>,
    ) -> NapiResult<Vec<HistoryRecordResult>> {
        let cookie_header = self.cookie_header.clone();
        let records = tokio::task::spawn_blocking(move || {
            FetchConferenceHistory::new().execute(FetchConferenceHistoryInput {
                cookie_header,
                max_pages: max_pages.unwrap_or(10) as usize,
                page_size: page_size.unwrap_or(25) as usize,
            })
        })
        .await
        .map_err(join_error)?
        .map_err(to_napi_error)?;

        Ok(records.into_iter().map(Into::into).collect())
    }

    #[napi]
    pub async fn join_room(
        &self,
        link: Option<String>,
        duration_seconds: Option<u32>,
    ) -> NapiResult<JoinRoomResult> {
        let resolved_link = self.resolve_room_link(link)?;
        let cookie_header = self.cookie_header.clone();
        let base_url = self.base_url.clone();
        let session_token = self.session_token.clone();
        let duration = Duration::from_secs(duration_seconds.unwrap_or(15) as u64);

        let report = tokio::task::spawn_blocking(move || {
            KTalkBotEngine::new(&cookie_header, &base_url, session_token.as_deref())?
                .join_room(&resolved_link, duration)
        })
        .await
        .map_err(join_error)?
        .map_err(to_napi_error)?;

        Ok(report.into())
    }

    #[napi]
    pub async fn record_participants(
        &self,
        link: Option<String>,
        duration_seconds: Option<u32>,
    ) -> NapiResult<Vec<ParticipantSnapshotResult>> {
        let resolved_link = self.resolve_room_link(link)?;
        let cookie_header = self.cookie_header.clone();
        let base_url = self.base_url.clone();
        let session_token = self.session_token.clone();
        let duration = Duration::from_secs(duration_seconds.unwrap_or(15) as u64);

        let participants = tokio::task::spawn_blocking(move || {
            KTalkBotEngine::new(&cookie_header, &base_url, session_token.as_deref())?
                .record_participants(&resolved_link, duration)
        })
        .await
        .map_err(join_error)?
        .map_err(to_napi_error)?;

        Ok(participants.into_iter().map(Into::into).collect())
    }

    #[napi]
    pub async fn send_chat_message(&self, text: String, link: Option<String>) -> NapiResult<()> {
        let resolved_link = self.resolve_room_link(link)?;
        let cookie_header = self.cookie_header.clone();
        let base_url = self.base_url.clone();
        let session_token = self.session_token.clone();

        tokio::task::spawn_blocking(move || {
            KTalkBotEngine::new(&cookie_header, &base_url, session_token.as_deref())?
                .send_chat_message(&resolved_link, &text)
        })
        .await
        .map_err(join_error)?
        .map_err(to_napi_error)
    }

    #[napi]
    pub async fn play_audio_on_mic(
        &self,
        audio_path: String,
        duration_seconds: Option<u32>,
        link: Option<String>,
    ) -> NapiResult<()> {
        let resolved_link = self.resolve_room_link(link)?;
        let cookie_header = self.cookie_header.clone();
        let base_url = self.base_url.clone();
        let session_token = self.session_token.clone();
        let duration = Duration::from_secs(duration_seconds.unwrap_or(15) as u64);

        tokio::task::spawn_blocking(move || {
            KTalkBotEngine::new(&cookie_header, &base_url, session_token.as_deref())?
                .play_audio_on_mic(&resolved_link, &audio_path, duration)
        })
        .await
        .map_err(join_error)?
        .map_err(to_napi_error)
    }
}

#[napi]
pub fn create_engine(
    cookie_header: String,
    base_url: Option<String>,
    room_link: Option<String>,
    session_token: Option<String>,
) -> NapiResult<KTalkClient> {
    let base_url = base_url.unwrap_or_else(|| "https://centraluniversity.ktalk.ru".to_owned());
    let _ = KTalkBotEngine::new(&cookie_header, &base_url, session_token.as_deref())
        .map_err(to_napi_error)?;

    Ok(KTalkClient::new(
        cookie_header,
        Some(base_url),
        room_link,
        session_token,
    ))
}

impl KTalkClient {
    fn resolve_room_link(&self, link: Option<String>) -> NapiResult<String> {
        match link {
            Some(link) => {
                crate::domain::room::RoomLink::parse(&link).map_err(to_napi_error)?;
                *self.room_link.lock().expect("room link lock poisoned") = Some(link.clone());
                Ok(link)
            }
            None => self
                .room_link
                .lock()
                .expect("room link lock poisoned")
                .clone()
                .ok_or_else(|| {
                    to_napi_error(KTalkError::InvalidRoomLink(
                        "room link is required; pass link explicitly or call bind_room() first"
                            .to_owned(),
                    ))
                }),
        }
    }
}

impl From<UserProfile> for UserProfileResult {
    fn from(value: UserProfile) -> Self {
        Self {
            user_id: value.user_id,
            first_name: value.first_name,
            last_name: value.last_name,
        }
    }
}

impl From<ParticipantSnapshot> for ParticipantSnapshotResult {
    fn from(value: ParticipantSnapshot) -> Self {
        Self {
            occupant_id: value.occupant_id,
            display_name: value.display_name,
            user_id: value.user_id,
        }
    }
}

impl From<JoinRoomReport> for JoinRoomResult {
    fn from(value: JoinRoomReport) -> Self {
        Self {
            room_name: value.room_name,
            conference_id: value.conference_id,
            joined: value.joined,
            participants: value.participants.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<Recording> for RecordingResult {
    fn from(value: Recording) -> Self {
        Self {
            recording_id: value.recording_id,
            playback_url: value.playback_url,
        }
    }
}

impl From<Participant> for ParticipantDetailResult {
    fn from(value: Participant) -> Self {
        match value {
            Participant::Authenticated { display_name } => Self {
                kind: "authenticated".to_owned(),
                display_name,
            },
            Participant::Anonymous { display_name } => Self {
                kind: "anonymous".to_owned(),
                display_name,
            },
        }
    }
}

impl From<ConferenceHistoryRecord> for HistoryRecordResult {
    fn from(value: ConferenceHistoryRecord) -> Self {
        let participants = value
            .participants
            .iter()
            .map(|participant| participant.display_name().to_owned())
            .collect::<Vec<_>>();
        let participant_details = value
            .participants
            .into_iter()
            .map(Into::into)
            .collect::<Vec<_>>();
        let (has_recording, recording_id, recording_url, recording) = match value.recording {
            Some(recording) => {
                let recording_id = Some(recording.recording_id.clone());
                let recording_url = Some(recording.playback_url.clone());
                (true, recording_id, recording_url, Some(recording.into()))
            }
            None => (false, None, None, None),
        };

        Self {
            key: value.key,
            room_name: value.room_name,
            title: value.title,
            start_time: value.start_time,
            end_time: value.end_time,
            participants_count: value.participants_count,
            participants,
            participant_details,
            has_recording,
            recording_id,
            recording_url,
            recording,
        }
    }
}

fn to_napi_error(error: KTalkError) -> NapiError {
    let status = match error {
        KTalkError::EmptyAuthToken
        | KTalkError::InvalidCookieBundle(_)
        | KTalkError::MissingSessionToken
        | KTalkError::InvalidRoomLink(_)
        | KTalkError::InvalidKTalkHost(_) => Status::InvalidArg,
        KTalkError::UnsupportedAudioPublishing => Status::GenericFailure,
        KTalkError::Http(_)
        | KTalkError::Json(_)
        | KTalkError::WebSocket(_)
        | KTalkError::HttpRequest(_)
        | KTalkError::Io(_)
        | KTalkError::Protocol(_) => Status::GenericFailure,
    };

    NapiError::new(status, error.to_string())
}

fn join_error(error: tokio::task::JoinError) -> NapiError {
    NapiError::new(
        Status::GenericFailure,
        format!("failed to execute blocking Node binding task: {error}"),
    )
}
