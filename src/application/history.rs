use crate::domain::history::ConferenceHistoryRecord;
use crate::error::Result;
use crate::infrastructure::config::token_loader::TokenLoader;
use crate::infrastructure::http::ktalk_http_client::KTalkHttpClient;

#[derive(Debug, Clone)]
pub struct FetchConferenceHistoryInput {
    pub auth_file: String,
    pub max_pages: usize,
    pub page_size: usize,
}

impl Default for FetchConferenceHistoryInput {
    fn default() -> Self {
        Self {
            auth_file: "ktalk_auth.txt".to_owned(),
            max_pages: 10,
            page_size: 25,
        }
    }
}

#[derive(Debug, Clone)]
pub struct FetchConferenceHistory {
    token_loader: TokenLoader,
    http_client: KTalkHttpClient,
}

impl FetchConferenceHistory {
    pub fn new() -> Self {
        Self {
            token_loader: TokenLoader::default(),
            http_client: KTalkHttpClient::default(),
        }
    }

    pub fn execute(
        &self,
        input: FetchConferenceHistoryInput,
    ) -> Result<Vec<ConferenceHistoryRecord>> {
        let token = self.token_loader.load(&input.auth_file)?;
        self.http_client
            .fetch_all_history(&token, input.max_pages, input.page_size)
    }
}

impl Default for FetchConferenceHistory {
    fn default() -> Self {
        Self::new()
    }
}
