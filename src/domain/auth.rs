use std::collections::BTreeMap;

use reqwest::header::{HeaderMap, SET_COOKIE};
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CookieBundle {
    cookies: BTreeMap<String, String>,
}

impl CookieBundle {
    pub fn parse(raw: &str) -> Result<Self> {
        let mut cookies = BTreeMap::new();

        for entry in raw.split(';') {
            let trimmed = entry.trim();
            if trimmed.is_empty() {
                continue;
            }

            let (name, value) = trimmed
                .split_once('=')
                .ok_or_else(|| KTalkError::InvalidCookieBundle(raw.to_owned()))?;

            let name = name.trim();
            let value = value.trim();
            if name.is_empty() || value.is_empty() {
                return Err(KTalkError::InvalidCookieBundle(raw.to_owned()));
            }

            cookies.insert(name.to_owned(), value.to_owned());
        }

        if cookies.is_empty() {
            return Err(KTalkError::InvalidCookieBundle(raw.to_owned()));
        }

        Ok(Self { cookies })
    }

    pub fn as_cookie_header(&self) -> String {
        self.cookies
            .iter()
            .map(|(name, value)| format!("{name}={value}"))
            .collect::<Vec<_>>()
            .join("; ")
    }

    pub fn merge_set_cookie_headers(&mut self, headers: &HeaderMap) {
        for value in headers.get_all(SET_COOKIE) {
            if let Ok(raw) = value.to_str() {
                if let Some((cookie, _attributes)) = raw.split_once(';') {
                    if let Some((name, value)) = cookie.split_once('=') {
                        let name = name.trim();
                        let value = value.trim();
                        if !name.is_empty() && !value.is_empty() {
                            self.cookies.insert(name.to_owned(), value.to_owned());
                        }
                    }
                }
            }
        }
    }

    pub fn get(&self, name: &str) -> Option<&str> {
        self.cookies.get(name).map(String::as_str)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthContext {
    cookies: CookieBundle,
    session_token: Option<SessionToken>,
}

impl AuthContext {
    pub fn parse(cookie_header: &str, session_token: Option<&str>) -> Result<Self> {
        Ok(Self {
            cookies: CookieBundle::parse(cookie_header)?,
            session_token: session_token.map(SessionToken::parse).transpose()?,
        })
    }

    pub fn cookies(&self) -> &CookieBundle {
        &self.cookies
    }

    pub fn cookies_mut(&mut self) -> &mut CookieBundle {
        &mut self.cookies
    }

    pub fn session_token(&self) -> Result<SessionToken> {
        self.session_token
            .clone()
            .ok_or(KTalkError::MissingSessionToken)
    }

    pub fn set_session_token(&mut self, session_token: &str) -> Result<()> {
        self.session_token = Some(SessionToken::parse(session_token)?);
        Ok(())
    }

    pub fn session_token_str(&self) -> Option<&str> {
        self.session_token.as_ref().map(SessionToken::as_str)
    }

    pub fn as_cookie_header(&self) -> String {
        self.cookies.as_cookie_header()
    }

    pub fn merge_set_cookie_headers(&mut self, headers: &HeaderMap) {
        self.cookies.merge_set_cookie_headers(headers);
    }

    pub fn get_cookie(&self, name: &str) -> Option<&str> {
        self.cookies.get(name)
    }
}

#[cfg(test)]
mod tests {
    use reqwest::header::{HeaderMap, HeaderValue, SET_COOKIE};

    use super::{AuthContext, CookieBundle, SessionToken};

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

    #[test]
    fn parses_cookie_bundle_without_session_cookie() {
        let cookies = CookieBundle::parse("ngtoken=xyz; kontur_ngtoken=qwe").unwrap();
        assert_eq!(cookies.get("ngtoken"), Some("xyz"));
        assert_eq!(cookies.get("kontur_ngtoken"), Some("qwe"));
    }

    #[test]
    fn auth_context_accepts_separate_session_token() {
        let auth = AuthContext::parse("ngtoken=xyz", Some("Session abc123")).unwrap();
        assert_eq!(auth.session_token().unwrap().as_str(), "abc123");
    }

    #[test]
    fn merges_set_cookie_headers() {
        let mut auth = AuthContext::parse("ngtoken=stale", None).unwrap();
        let mut headers = HeaderMap::new();
        headers.append(
            SET_COOKIE,
            HeaderValue::from_static("ngtoken=fresh; Path=/; HttpOnly"),
        );

        auth.merge_set_cookie_headers(&headers);
        assert_eq!(auth.get_cookie("ngtoken"), Some("fresh"));
    }
}
