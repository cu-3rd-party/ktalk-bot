use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ConferenceHistoryResponse {
    #[serde(default)]
    pub conferences: Vec<RawConferenceHistoryRecord>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RawConferenceHistoryRecord {
    pub key: Option<String>,
    pub room_name: String,
    pub title: Option<String>,
    pub start_time: Option<String>,
    pub end_time: Option<String>,
    pub participants_count: Option<u32>,
    #[serde(default)]
    pub artifacts: RawArtifacts,
}

#[derive(Debug, Default, Deserialize)]
pub struct RawArtifacts {
    #[serde(default)]
    pub participants: Vec<RawParticipant>,
    #[serde(default)]
    pub content: Vec<RawContentItem>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RawParticipant {
    #[serde(default)]
    pub is_anonymous: bool,
    pub anonymous_name: Option<String>,
    #[serde(default)]
    pub user_info: RawUserInfo,
}

#[derive(Debug, Default, Deserialize)]
pub struct RawUserInfo {
    pub firstname: Option<String>,
    pub surname: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
pub struct RawContentItem {
    #[serde(rename = "type")]
    pub kind: Option<String>,
    pub id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ContextResponse {
    pub user: RawContextUser,
}

#[derive(Debug, Deserialize)]
pub struct RawContextUser {
    pub id: Option<String>,
    pub login: Option<String>,
    pub firstname: Option<String>,
    pub surname: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RoomResponse {
    pub conference_id: String,
}
