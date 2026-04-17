use serde::{Deserialize, Serialize};
use url::Url;

use crate::error::{KTalkError, Result};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RoomLink(String);

impl RoomLink {
    pub fn parse(raw: &str) -> Result<Self> {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            return Err(KTalkError::InvalidRoomLink(raw.to_owned()));
        }

        let parsed =
            Url::parse(trimmed).map_err(|_| KTalkError::InvalidRoomLink(raw.to_owned()))?;
        let host = parsed
            .host_str()
            .ok_or_else(|| KTalkError::InvalidRoomLink(raw.to_owned()))?;
        if !is_supported_ktalk_host(host) {
            return Err(KTalkError::InvalidKTalkHost(host.to_owned()));
        }

        Ok(Self(trimmed.to_owned()))
    }

    pub fn base_url(&self) -> Result<String> {
        let parsed =
            Url::parse(&self.0).map_err(|_| KTalkError::InvalidRoomLink(self.0.clone()))?;
        let host = parsed
            .host_str()
            .ok_or_else(|| KTalkError::InvalidRoomLink(self.0.clone()))?;
        let scheme = parsed.scheme();
        Ok(format!("{scheme}://{host}"))
    }

    pub fn short_name(&self) -> Result<RoomShortName> {
        let short_name = self
            .0
            .trim_end_matches('/')
            .rsplit('/')
            .next()
            .filter(|value| !value.is_empty())
            .ok_or_else(|| KTalkError::InvalidRoomLink(self.0.clone()))?;

        RoomShortName::new(short_name)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RoomShortName(String);

impl RoomShortName {
    pub fn new(value: &str) -> Result<Self> {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            return Err(KTalkError::InvalidRoomLink(value.to_owned()));
        }

        Ok(Self(trimmed.to_owned()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

pub fn is_supported_ktalk_host(host: &str) -> bool {
    let normalized = host.trim_matches(['[', ']']);
    normalized == "ktalk.ru"
        || normalized.ends_with(".ktalk.ru")
        || matches!(normalized, "localhost" | "127.0.0.1" | "::1")
}

#[cfg(test)]
mod tests {
    use super::{RoomLink, is_supported_ktalk_host};

    #[test]
    fn extracts_short_name_from_room_link() {
        let room = RoomLink::parse("https://centraluniversity.ktalk.ru/ewwv291ipuud").unwrap();
        assert_eq!(room.short_name().unwrap().as_str(), "ewwv291ipuud");
    }

    #[test]
    fn extracts_base_url_from_room_link() {
        let room = RoomLink::parse("https://demo.ktalk.ru/ewwv291ipuud").unwrap();
        assert_eq!(room.base_url().unwrap(), "https://demo.ktalk.ru");
    }

    #[test]
    fn rejects_non_ktalk_hosts() {
        let error = RoomLink::parse("https://example.com/room").unwrap_err();
        assert_eq!(error.to_string(), "unsupported KTalk host: example.com");
    }

    #[test]
    fn accepts_ktalk_hosts() {
        assert!(is_supported_ktalk_host("ktalk.ru"));
        assert!(is_supported_ktalk_host("centraluniversity.ktalk.ru"));
        assert!(is_supported_ktalk_host("localhost"));
        assert!(!is_supported_ktalk_host("example.com"));
    }
}
