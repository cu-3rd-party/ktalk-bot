use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConferenceHistoryRecord {
    pub key: Option<String>,
    pub room_name: String,
    pub title: String,
    pub start_time: Option<String>,
    pub end_time: Option<String>,
    pub participants_count: u32,
    pub participants: Vec<Participant>,
    pub recording: Option<Recording>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Participant {
    Authenticated { display_name: String },
    Anonymous { display_name: String },
}

impl Participant {
    pub fn display_name(&self) -> &str {
        match self {
            Participant::Authenticated { display_name }
            | Participant::Anonymous { display_name } => display_name,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Recording {
    pub recording_id: String,
    pub playback_url: String,
}
