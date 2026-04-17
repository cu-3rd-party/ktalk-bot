use regex::Regex;
use serde_json::Value;

use crate::domain::bot::ParticipantSnapshot;

pub fn parse_participant_presence(text: &str) -> Option<ParticipantSnapshot> {
    if !text.contains("<presence") {
        return None;
    }

    let occupant_id = capture(text, r#"from="[^"]+/([^"]+)""#)?;
    let display_name = capture(text, r#"<nick [^>]*>([^<]+)</nick>"#)
        .or_else(|| capture(text, r#"<nick>([^<]+)</nick>"#))
        .unwrap_or_else(|| occupant_id.clone());
    let user_id = capture(
        text,
        r#"<jitsi_participant_user-info>([^<]+)</jitsi_participant_user-info>"#,
    )
    .and_then(|raw| decode_user_id(&raw));

    Some(ParticipantSnapshot {
        occupant_id,
        display_name,
        user_id,
    })
}

fn capture(text: &str, pattern: &str) -> Option<String> {
    let regex = Regex::new(pattern).ok()?;
    regex
        .captures(text)
        .and_then(|captures| captures.get(1))
        .map(|matched| matched.as_str().to_owned())
}

fn decode_user_id(raw: &str) -> Option<String> {
    let decoded = raw
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
        .replace("&amp;", "&");
    let value: Value = serde_json::from_str(&decoded).ok()?;
    value.get("key").and_then(Value::as_str).map(str::to_owned)
}

#[cfg(test)]
mod tests {
    use super::parse_participant_presence;

    #[test]
    fn parses_presence_snapshot() {
        let text = "<presence from=\"room@muc.meet.jitsi/abc123\"><nick xmlns=\"http://jabber.org/protocol/nick\">Alice</nick><jitsi_participant_user-info>{&quot;key&quot;:&quot;user-1&quot;}</jitsi_participant_user-info></presence>";
        let snapshot = parse_participant_presence(text).unwrap();
        assert_eq!(snapshot.occupant_id, "abc123");
        assert_eq!(snapshot.display_name, "Alice");
        assert_eq!(snapshot.user_id.as_deref(), Some("user-1"));
    }
}
