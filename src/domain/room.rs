use serde::{Deserialize, Serialize};

use crate::error::{KTalkError, Result};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RoomLink(String);

impl RoomLink {
    pub fn parse(raw: &str) -> Result<Self> {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            return Err(KTalkError::InvalidRoomLink(raw.to_owned()));
        }

        Ok(Self(trimmed.to_owned()))
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

#[cfg(test)]
mod tests {
    use super::RoomLink;

    #[test]
    fn extracts_short_name_from_room_link() {
        let room = RoomLink::parse("https://centraluniversity.ktalk.ru/ewwv291ipuud").unwrap();
        assert_eq!(room.short_name().unwrap().as_str(), "ewwv291ipuud");
    }
}
