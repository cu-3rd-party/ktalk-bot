use crate::domain::history::{ConferenceHistoryRecord, Participant, Recording};
use crate::infrastructure::http::dto::{RawConferenceHistoryRecord, RawParticipant};

pub fn map_history_record(
    raw: RawConferenceHistoryRecord,
    base_url: &str,
) -> ConferenceHistoryRecord {
    let participants = raw
        .artifacts
        .participants
        .into_iter()
        .filter_map(map_participant)
        .collect::<Vec<_>>();

    let recording = raw
        .artifacts
        .content
        .into_iter()
        .find(|item| item.kind.as_deref() == Some("record"))
        .and_then(|item| item.id)
        .map(|recording_id| Recording {
            playback_url: format!("{base_url}/recordings/{recording_id}"),
            recording_id,
        });

    ConferenceHistoryRecord {
        key: raw.key,
        room_name: raw.room_name,
        title: raw.title.unwrap_or_else(|| "Без названия".to_owned()),
        start_time: raw.start_time,
        end_time: raw.end_time,
        participants_count: raw.participants_count.unwrap_or(0),
        participants,
        recording,
    }
}

fn map_participant(raw: RawParticipant) -> Option<Participant> {
    if raw.is_anonymous {
        return Some(Participant::Anonymous {
            display_name: raw.anonymous_name.unwrap_or_else(|| "Гость".to_owned()),
        });
    }

    let first_name = raw.user_info.firstname.unwrap_or_default();
    let last_name = raw.user_info.surname.unwrap_or_default();
    let full_name = format!("{first_name} {last_name}").trim().to_owned();

    if full_name.is_empty() {
        return None;
    }

    Some(Participant::Authenticated {
        display_name: full_name,
    })
}

#[cfg(test)]
mod tests {
    use crate::infrastructure::http::dto::ConferenceHistoryResponse;

    use super::map_history_record;

    #[test]
    fn maps_recordings_and_participants_from_raw_payload() {
        let raw = serde_json::from_str::<ConferenceHistoryResponse>(include_str!(
            "../../../tests/fixtures/history_response.json"
        ))
        .unwrap()
        .conferences
        .into_iter()
        .next()
        .unwrap();

        let record = map_history_record(raw, "https://centraluniversity.ktalk.ru");

        assert_eq!(record.room_name, "seminar-room");
        assert_eq!(record.title, "Demo Seminar");
        assert_eq!(record.participants_count, 3);
        assert_eq!(
            record
                .participants
                .iter()
                .map(|participant| participant.display_name().to_owned())
                .collect::<Vec<_>>(),
            vec!["Alice Smith".to_owned(), "Гость".to_owned()]
        );
        assert_eq!(
            record.recording.unwrap().playback_url,
            "https://centraluniversity.ktalk.ru/recordings/recording-42"
        );
    }

    #[test]
    fn falls_back_to_default_title_when_missing() {
        let raw = serde_json::from_str::<ConferenceHistoryResponse>(
            r#"{"conferences":[{"key":"k1","roomName":"room","startTime":null,"endTime":null,"participantsCount":0,"artifacts":{"participants":[],"content":[]}}]}"#,
        )
        .unwrap()
        .conferences
        .into_iter()
        .next()
        .unwrap();

        let record = map_history_record(raw, "https://centraluniversity.ktalk.ru");
        assert_eq!(record.title, "Без названия");
    }
}
