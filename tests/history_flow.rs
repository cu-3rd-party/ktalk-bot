use httptest::matchers::{all_of, contains, request};
use httptest::responders::json_encoded;
use httptest::{Expectation, Server};
use serde_json::json;

use ktalk_bot::{AuthContext, KTalkHttpClient};

#[test]
fn fetches_history_records_end_to_end() {
    let server = Server::run();
    server.expect(
        Expectation::matching(all_of![
            request::method_path("GET", "/api/conferenceshistory"),
            request::headers(contains(("authorization", "Session test-token")))
        ])
        .respond_with(json_encoded(json!({
            "conferences": [
                {
                    "key": "conf-1",
                    "roomName": "seminar-room",
                    "title": "Demo Seminar",
                    "startTime": "2026-04-17T09:00:00Z",
                    "endTime": "2026-04-17T10:00:00Z",
                    "participantsCount": 2,
                    "artifacts": {
                        "participants": [
                            {
                                "isAnonymous": false,
                                "userInfo": {
                                    "firstname": "Alice",
                                    "surname": "Smith"
                                }
                            },
                            {
                                "isAnonymous": true,
                                "anonymousName": "Гость"
                            }
                        ],
                        "content": [
                            {
                                "type": "record",
                                "id": "recording-42"
                            }
                        ]
                    }
                }
            ]
        }))),
    );

    let client = KTalkHttpClient::with_base_url(server.url_str("")).unwrap();
    let mut auth =
        AuthContext::parse("ngtoken=warm; kontur_ngtoken=hot", Some("test-token")).unwrap();

    let records = client.fetch_all_history(&mut auth, 3, 25).unwrap();

    assert_eq!(records.len(), 1);
    assert_eq!(records[0].room_name, "seminar-room");
    assert_eq!(records[0].participants.len(), 2);
    assert_eq!(
        records[0].recording.as_ref().unwrap().playback_url,
        format!("{}recordings/recording-42", server.url_str("/"))
    );
}
