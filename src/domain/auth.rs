use serde::{Deserialize, Serialize};

use crate::error::{KTalkError, Result};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SessionToken(String);

impl SessionToken {
    pub fn parse(raw: &str) -> Result<Self> {
        let token = raw
            .trim()
            .strip_prefix("Session ")
            .unwrap_or(raw.trim())
            .trim();

        if token.is_empty() {
            return Err(KTalkError::EmptyAuthToken);
        }

        Ok(Self(token.to_owned()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn as_authorization_header(&self) -> String {
        format!("Session {}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::SessionToken;

    #[test]
    fn strips_session_prefix() {
        let token = SessionToken::parse("Session abc123").unwrap();
        assert_eq!(token.as_str(), "abc123");
    }

    #[test]
    fn rejects_empty_token() {
        let error = SessionToken::parse("  ").unwrap_err();
        assert_eq!(error.to_string(), "authentication token is empty");
    }
}
