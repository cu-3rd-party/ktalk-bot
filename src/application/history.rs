use crate::domain::auth::AuthContext;
use crate::domain::history::ConferenceHistoryRecord;
use crate::error::Result;
use crate::infrastructure::http::ktalk_http_client::KTalkHttpClient;

#[derive(Debug, Clone)]
pub struct FetchConferenceHistoryInput {
    pub cookie_header: String,
    pub max_pages: usize,
    pub page_size: usize,
}

impl Default for FetchConferenceHistoryInput {
    fn default() -> Self {
        Self {
            cookie_header: String::new(),
            max_pages: 10,
            page_size: 25,
        }
    }
}

#[derive(Debug, Clone)]
pub struct FetchConferenceHistory {
    http_client: KTalkHttpClient,
}

impl FetchConferenceHistory {
    pub fn new() -> Self {
        Self {
            http_client: KTalkHttpClient::default(),
        }
    }

    pub fn execute(
        &self,
        input: FetchConferenceHistoryInput,
    ) -> Result<Vec<ConferenceHistoryRecord>> {
        let mut auth = AuthContext::parse(&input.cookie_header, None)?;
        self.http_client
            .fetch_all_history(&mut auth, input.max_pages, input.page_size)
    }
}

impl Default for FetchConferenceHistory {
    fn default() -> Self {
        Self::new()
    }
}
