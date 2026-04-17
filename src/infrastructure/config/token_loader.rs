use std::fs;

use crate::domain::auth::SessionToken;
use crate::error::{KTalkError, Result};

#[derive(Debug, Clone, Default)]
pub struct TokenLoader;

impl TokenLoader {
    pub fn load(&self, path: &str) -> Result<SessionToken> {
        let content = fs::read_to_string(path).map_err(|error| {
            if error.kind() == std::io::ErrorKind::NotFound {
                KTalkError::AuthFileNotFound {
                    path: path.to_owned(),
                }
            } else {
                KTalkError::Io(error)
            }
        })?;

        SessionToken::parse(&content)
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::TokenLoader;

    #[test]
    fn loads_and_normalizes_token_from_file() {
        let path = std::env::temp_dir().join(format!(
            "ktalk_token_loader_{}.txt",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::write(&path, "Session abc123").unwrap();

        let token = TokenLoader::default().load(path.to_str().unwrap()).unwrap();
        assert_eq!(token.as_str(), "abc123");

        fs::remove_file(path).unwrap();
    }
}
