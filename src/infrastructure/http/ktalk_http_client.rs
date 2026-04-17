use chrono::Utc;
use reqwest::blocking::Client;
use reqwest::header::{
    ACCEPT, AUTHORIZATION, CONTENT_TYPE, COOKIE, HeaderMap, HeaderValue, USER_AGENT,
};

use crate::domain::auth::CookieBundle;
use crate::domain::bot::UserProfile;
use crate::domain::history::ConferenceHistoryRecord;
use crate::domain::room::is_supported_ktalk_host;
use crate::error::{KTalkError, Result};
use crate::infrastructure::http::dto::{ConferenceHistoryResponse, ContextResponse, RoomResponse};
use crate::infrastructure::parsing::history_mapper::map_history_record;

const BASE_URL: &str = "https://centraluniversity.ktalk.ru";

#[derive(Debug, Clone)]
pub struct KTalkHttpClient {
    client: Client,
    base_url: String,
}

impl KTalkHttpClient {
    pub fn new() -> Result<Self> {
        Self::with_base_url(BASE_URL)
    }

    pub fn with_base_url(base_url: impl Into<String>) -> Result<Self> {
        let base_url = base_url.into().trim_end_matches('/').to_owned();
        let parsed = url::Url::parse(&base_url)
            .map_err(|_| KTalkError::InvalidRoomLink(base_url.clone()))?;
        let host = parsed
            .host_str()
            .ok_or_else(|| KTalkError::InvalidRoomLink(base_url.clone()))?;
        if !is_supported_ktalk_host(host) {
            return Err(KTalkError::InvalidKTalkHost(host.to_owned()));
        }

        let client = Client::builder()
            .default_headers(default_headers())
            .build()?;
        Ok(Self { client, base_url })
    }

    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    pub fn bootstrap(&self, cookies: &mut CookieBundle) -> Result<UserProfile> {
        let response = self
            .authorized_get(format!("{}/api/context", self.base_url), cookies)?
            .send()?
            .error_for_status()?;
        cookies.merge_set_cookie_headers(response.headers());
        let payload: ContextResponse = response.json()?;

        Ok(UserProfile {
            user_id: payload.user.id.or(payload.user.login).ok_or_else(|| {
                KTalkError::Protocol("context response did not contain user id".to_owned())
            })?,
            first_name: payload
                .user
                .firstname
                .unwrap_or_else(|| "Студент".to_owned()),
            last_name: payload.user.surname.unwrap_or_default(),
        })
    }

    pub fn resolve_room(&self, room_name: &str, cookies: &mut CookieBundle) -> Result<String> {
        let response = self
            .authorized_get(format!("{}/api/rooms/{room_name}", self.base_url), cookies)?
            .send()?
            .error_for_status()?;
        cookies.merge_set_cookie_headers(response.headers());
        let payload: RoomResponse = response.json()?;
        Ok(payload.conference_id)
    }

    pub fn send_activity(&self, room_name: &str, cookies: &mut CookieBundle) -> Result<()> {
        let response = self
            .authorized_post(format!("{}/api/UserActivities", self.base_url), cookies)?
            .json(&vec![serde_json::json!({
                "$type": "GotoRoom",
                "cameraEnabled": false,
                "micEnabled": false,
                "roomName": room_name,
                "timestamp": Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
            })])
            .send()?;
        cookies.merge_set_cookie_headers(response.headers());
        Ok(())
    }

    pub fn fetch_all_history(
        &self,
        cookies: &mut CookieBundle,
        max_pages: usize,
        page_size: usize,
    ) -> Result<Vec<ConferenceHistoryRecord>> {
        let mut all_records = Vec::new();

        for page_index in 0..max_pages {
            let skip = page_index * page_size;
            let response = self
                .authorized_get(format!("{}/api/conferenceshistory", self.base_url), cookies)?
                .query(&[
                    ("skip", skip.to_string()),
                    ("top", page_size.to_string()),
                    ("includeUnfinished", "true".to_owned()),
                ])
                .send()?
                .error_for_status()?;
            cookies.merge_set_cookie_headers(response.headers());
            let payload: ConferenceHistoryResponse = response.json()?;
            let batch = payload
                .conferences
                .into_iter()
                .map(|raw| map_history_record(raw, &self.base_url))
                .collect::<Vec<_>>();

            if batch.is_empty() {
                break;
            }

            let is_last_page = batch.len() < page_size;
            all_records.extend(batch);
            if is_last_page {
                break;
            }
        }

        Ok(all_records)
    }

    fn authorized_get(
        &self,
        url: String,
        cookies: &CookieBundle,
    ) -> Result<reqwest::blocking::RequestBuilder> {
        Ok(self
            .client
            .get(url)
            .header(
                AUTHORIZATION,
                cookies.session_token()?.as_authorization_header(),
            )
            .header(COOKIE, cookies.as_cookie_header()))
    }

    fn authorized_post(
        &self,
        url: String,
        cookies: &CookieBundle,
    ) -> Result<reqwest::blocking::RequestBuilder> {
        Ok(self
            .client
            .post(url)
            .header(
                AUTHORIZATION,
                cookies.session_token()?.as_authorization_header(),
            )
            .header(COOKIE, cookies.as_cookie_header()))
    }
}

impl Default for KTalkHttpClient {
    fn default() -> Self {
        Self::new().expect("default HTTP client configuration should be valid")
    }
}

fn default_headers() -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert(USER_AGENT, HeaderValue::from_static("ktalk-bot/0.1"));
    headers.insert(ACCEPT, HeaderValue::from_static("application/json"));
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    headers.insert("x-platform", HeaderValue::from_static("web"));
    headers
}
