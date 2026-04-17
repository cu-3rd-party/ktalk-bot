use reqwest::blocking::Client;
use reqwest::header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE, HeaderMap, HeaderValue, USER_AGENT};

use crate::domain::auth::SessionToken;
use crate::domain::history::ConferenceHistoryRecord;
use crate::error::Result;
use crate::infrastructure::http::dto::ConferenceHistoryResponse;
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
        let client = Client::builder()
            .default_headers(default_headers())
            .build()?;
        Ok(Self {
            client,
            base_url: base_url.into().trim_end_matches('/').to_owned(),
        })
    }

    pub fn fetch_all_history(
        &self,
        token: &SessionToken,
        max_pages: usize,
        page_size: usize,
    ) -> Result<Vec<ConferenceHistoryRecord>> {
        let mut all_records = Vec::new();

        for page_index in 0..max_pages {
            let skip = page_index * page_size;
            let url = format!("{}/api/conferenceshistory", self.base_url);
            let response = self
                .client
                .get(url)
                .header(AUTHORIZATION, token.as_authorization_header())
                .query(&[
                    ("skip", skip.to_string()),
                    ("top", page_size.to_string()),
                    ("includeUnfinished", "true".to_owned()),
                ])
                .send()?
                .error_for_status()?;

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
