use std::sync::Mutex;
use std::time::Duration;

use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};

use crate::application::bot_engine::KTalkBotEngine;
use crate::application::history::{FetchConferenceHistory, FetchConferenceHistoryInput};
use crate::domain::bot::{JoinRoomReport, ParticipantSnapshot};
use crate::domain::history::{ConferenceHistoryRecord, Participant};

/// Python-facing KTalk client backed by an internal engine created from cookies.
#[pyclass]
pub struct KTalkClient {
    cookie_header: String,
    base_url: String,
    session_token: Option<String>,
    room_link: Mutex<Option<String>>,
}

#[pymethods]
impl KTalkClient {
    #[new]
    #[pyo3(
        signature = (cookie_header, base_url = "https://centraluniversity.ktalk.ru".to_owned(), room_link = None, session_token = None),
        text_signature = "(cookie_header: str, base_url: str = 'https://centraluniversity.ktalk.ru', room_link: str | None = None, session_token: str | None = None)"
    )]
    /// Создает Python-клиент KTalk из заголовка `Cookie`.
    ///
    /// Параметры:
    ///     cookie_header (str): Полная строка заголовка `Cookie`, обычно содержащая `ngtoken` и `kontur_ngtoken`.
    ///     base_url (str): Базовый URL инстанса KTalk.
    ///     room_link (str | None): Комната по умолчанию для последующих realtime-команд.
    ///     session_token (str | None): Отдельный session token для realtime-авторизации, если он уже известен.
    ///
    /// Возвращает:
    ///     KTalkClient: Клиент с внутренним engine-объектом, создаваемым под капотом.
    pub fn new(
        cookie_header: String,
        base_url: String,
        room_link: Option<String>,
        session_token: Option<String>,
    ) -> Self {
        Self {
            cookie_header,
            base_url,
            session_token,
            room_link: Mutex::new(room_link),
        }
    }

    #[pyo3(text_signature = "($self, link: str)")]
    /// Привязывает комнату к клиенту. После этого realtime-команды можно вызывать без `link`.
    pub fn bind_room(&self, link: &str) -> PyResult<()> {
        crate::domain::room::RoomLink::parse(link).map_err(PyErr::from)?;
        *self.room_link.lock().expect("room link lock poisoned") = Some(link.to_owned());
        Ok(())
    }

    #[pyo3(text_signature = "($self)")]
    /// Возвращает привязанную комнату или `None`.
    pub fn current_room(&self) -> Option<String> {
        self.room_link
            .lock()
            .expect("room link lock poisoned")
            .clone()
    }

    #[pyo3(text_signature = "($self)")]
    /// Обновляет cookies через KTalk API и возвращает профиль авторизованного пользователя.
    ///
    /// Возвращает:
    ///     dict[str, str]: Словарь с ключами `user_id`, `first_name`, `last_name`.
    pub fn renew_cookies<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyDict>> {
        let profile = self.engine()?.renew_cookies().map_err(PyErr::from)?;
        let dict = PyDict::new(py);
        dict.set_item("user_id", profile.user_id)?;
        dict.set_item("first_name", profile.first_name)?;
        dict.set_item("last_name", profile.last_name)?;
        Ok(dict)
    }

    #[pyo3(
        signature = (max_pages = 10, page_size = 25),
        text_signature = "($self, max_pages: int = 10, page_size: int = 25)"
    )]
    /// Возвращает историю конференций.
    ///
    /// Параметры:
    ///     max_pages (int): Максимальное количество страниц истории.
    ///     page_size (int): Размер одной страницы.
    ///
    /// Возвращает:
    ///     list[dict[str, object]]: Список конференций с полями комнаты, участников и записи.
    pub fn get_history<'py>(
        &self,
        py: Python<'py>,
        max_pages: usize,
        page_size: usize,
    ) -> PyResult<Bound<'py, PyList>> {
        let records = FetchConferenceHistory::new()
            .execute(FetchConferenceHistoryInput {
                cookie_header: self.cookie_header.clone(),
                max_pages,
                page_size,
            })
            .map_err(PyErr::from)?;

        records_to_pylist(py, &records)
    }

    #[pyo3(
        signature = (duration_seconds = 15, link = None),
        text_signature = "($self, duration_seconds: int = 15, link: str | None = None)"
    )]
    /// Подключается к комнате на ограниченное время и возвращает отчет о подключении.
    ///
    /// Параметры:
    ///     duration_seconds (int): Сколько секунд удерживать подключение.
    ///     link (str | None): Полная ссылка на комнату KTalk. Если не передана, используется ранее привязанная комната.
    ///
    /// Возвращает:
    ///     dict[str, object]: Словарь с ключами `room_name`, `conference_id`, `joined`, `participants`.
    pub fn join_room<'py>(
        &self,
        py: Python<'py>,
        duration_seconds: u64,
        link: Option<&str>,
    ) -> PyResult<Bound<'py, PyDict>> {
        let report = self
            .engine()?
            .join_room(
                &self.resolve_room_link(link)?,
                Duration::from_secs(duration_seconds),
            )
            .map_err(PyErr::from)?;
        join_report_to_pydict(py, &report)
    }

    #[pyo3(
        signature = (text, link = None),
        text_signature = "($self, text: str, link: str | None = None)"
    )]
    /// Отправляет текстовое сообщение в чат комнаты.
    ///
    /// Параметры:
    ///     text (str): Текст сообщения.
    ///     link (str | None): Полная ссылка на комнату KTalk. Если не передана, используется ранее привязанная комната.
    pub fn send_chat_message(&self, text: &str, link: Option<&str>) -> PyResult<()> {
        self.engine()?
            .send_chat_message(&self.resolve_room_link(link)?, text)
            .map_err(PyErr::from)
    }

    #[pyo3(
        signature = (duration_seconds = 15, link = None),
        text_signature = "($self, duration_seconds: int = 15, link: str | None = None)"
    )]
    /// Подключается к комнате и возвращает список участников, замеченных во время сессии.
    ///
    /// Параметры:
    ///     duration_seconds (int): Длительность сессии в секундах.
    ///     link (str | None): Полная ссылка на комнату KTalk. Если не передана, используется ранее привязанная комната.
    ///
    /// Возвращает:
    ///     list[dict[str, object]]: Список участников с полями `occupant_id`, `display_name`, `user_id`.
    pub fn record_participants<'py>(
        &self,
        py: Python<'py>,
        duration_seconds: u64,
        link: Option<&str>,
    ) -> PyResult<Bound<'py, PyList>> {
        let participants = self
            .engine()?
            .record_participants(
                &self.resolve_room_link(link)?,
                Duration::from_secs(duration_seconds),
            )
            .map_err(PyErr::from)?;
        participant_snapshots_to_pylist(py, &participants)
    }

    #[pyo3(
        signature = (audio_path, duration_seconds = 15, link = None),
        text_signature = "($self, audio_path: str, duration_seconds: int = 15, link: str | None = None)"
    )]
    /// Пытается воспроизвести аудио в микрофонный канал.
    ///
    /// Параметры:
    ///     audio_path (str): Путь к аудиофайлу.
    ///     duration_seconds (int): Длительность активной сессии.
    ///     link (str | None): Полная ссылка на комнату KTalk. Если не передана, используется ранее привязанная комната.
    ///
    /// Исключения:
    ///     NotImplementedError: Полная публикация WebRTC-аудио пока не реализована.
    pub fn play_audio_on_mic(
        &self,
        audio_path: &str,
        duration_seconds: u64,
        link: Option<&str>,
    ) -> PyResult<()> {
        self.engine()?
            .play_audio_on_mic(
                &self.resolve_room_link(link)?,
                audio_path,
                Duration::from_secs(duration_seconds),
            )
            .map_err(PyErr::from)
    }
}

#[pyfunction]
#[pyo3(
    signature = (cookie_header, base_url = "https://centraluniversity.ktalk.ru".to_owned(), room_link = None, session_token = None),
    text_signature = "(cookie_header: str, base_url: str = 'https://centraluniversity.ktalk.ru', room_link: str | None = None, session_token: str | None = None)"
)]
/// Создает black-box engine и возвращает готовый `KTalkClient`.
///
/// Параметры:
///     cookie_header (str): Полная строка заголовка `Cookie`, обычно содержащая `ngtoken` и `kontur_ngtoken`.
///     base_url (str): Базовый URL инстанса KTalk.
///     room_link (str | None): Комната по умолчанию для последующих realtime-команд.
///     session_token (str | None): Отдельный session token для realtime-авторизации, если он уже известен.
///
/// Возвращает:
///     KTalkClient: Клиент, который использует внутренний engine под капотом.
pub fn create_engine(
    cookie_header: String,
    base_url: String,
    room_link: Option<String>,
    session_token: Option<String>,
) -> PyResult<KTalkClient> {
    let _ = KTalkBotEngine::new(&cookie_header, &base_url, session_token.as_deref())
        .map_err(PyErr::from)?;
    Ok(KTalkClient::new(
        cookie_header,
        base_url,
        room_link,
        session_token,
    ))
}

impl KTalkClient {
    fn engine(&self) -> Result<KTalkBotEngine, crate::error::KTalkError> {
        KTalkBotEngine::new(
            &self.cookie_header,
            &self.base_url,
            self.session_token.as_deref(),
        )
    }

    fn resolve_room_link(&self, link: Option<&str>) -> PyResult<String> {
        match link {
            Some(link) => {
                crate::domain::room::RoomLink::parse(link).map_err(PyErr::from)?;
                *self.room_link.lock().expect("room link lock poisoned") = Some(link.to_owned());
                Ok(link.to_owned())
            }
            None => self
                .room_link
                .lock()
                .expect("room link lock poisoned")
                .clone()
                .ok_or_else(|| {
                    PyErr::from(crate::error::KTalkError::InvalidRoomLink(
                        "room link is required; pass link explicitly or call bind_room() first"
                            .to_owned(),
                    ))
                }),
        }
    }
}

fn records_to_pylist<'py>(
    py: Python<'py>,
    records: &[ConferenceHistoryRecord],
) -> PyResult<Bound<'py, PyList>> {
    let items = records
        .iter()
        .map(|record| record_to_pydict(py, record))
        .collect::<PyResult<Vec<_>>>()?;

    PyList::new(py, items)
}

fn record_to_pydict<'py>(
    py: Python<'py>,
    record: &ConferenceHistoryRecord,
) -> PyResult<Bound<'py, PyDict>> {
    let dict = PyDict::new(py);
    dict.set_item("key", record.key.as_deref())?;
    dict.set_item("room_name", &record.room_name)?;
    dict.set_item("title", &record.title)?;
    dict.set_item("start_time", record.start_time.as_deref())?;
    dict.set_item("end_time", record.end_time.as_deref())?;
    dict.set_item("participants_count", record.participants_count)?;
    dict.set_item(
        "participants",
        record
            .participants
            .iter()
            .map(|participant| participant.display_name().to_owned())
            .collect::<Vec<_>>(),
    )?;
    dict.set_item(
        "participant_details",
        participant_details(py, &record.participants)?,
    )?;
    match &record.recording {
        Some(recording) => {
            let recording_dict = PyDict::new(py);
            recording_dict.set_item("recording_id", &recording.recording_id)?;
            recording_dict.set_item("playback_url", &recording.playback_url)?;
            dict.set_item("recording", recording_dict)?;
            dict.set_item("has_recording", true)?;
            dict.set_item("recording_id", &recording.recording_id)?;
            dict.set_item("recording_url", &recording.playback_url)?;
        }
        None => {
            dict.set_item("recording", py.None())?;
            dict.set_item("has_recording", false)?;
            dict.set_item("recording_id", py.None())?;
            dict.set_item("recording_url", py.None())?;
        }
    }

    Ok(dict)
}

fn participant_details<'py>(
    py: Python<'py>,
    participants: &[Participant],
) -> PyResult<Bound<'py, PyList>> {
    let entries = participants
        .iter()
        .map(|participant| {
            let dict = PyDict::new(py);
            match participant {
                Participant::Authenticated { display_name } => {
                    dict.set_item("kind", "authenticated")?;
                    dict.set_item("display_name", display_name)?;
                }
                Participant::Anonymous { display_name } => {
                    dict.set_item("kind", "anonymous")?;
                    dict.set_item("display_name", display_name)?;
                }
            }
            Ok(dict)
        })
        .collect::<PyResult<Vec<_>>>()?;

    PyList::new(py, entries)
}

fn join_report_to_pydict<'py>(
    py: Python<'py>,
    report: &JoinRoomReport,
) -> PyResult<Bound<'py, PyDict>> {
    let dict = PyDict::new(py);
    dict.set_item("room_name", &report.room_name)?;
    dict.set_item("conference_id", &report.conference_id)?;
    dict.set_item("joined", report.joined)?;
    dict.set_item(
        "participants",
        participant_snapshots_to_pylist(py, &report.participants)?,
    )?;
    Ok(dict)
}

fn participant_snapshots_to_pylist<'py>(
    py: Python<'py>,
    participants: &[ParticipantSnapshot],
) -> PyResult<Bound<'py, PyList>> {
    let items = participants
        .iter()
        .map(|participant| {
            let dict = PyDict::new(py);
            dict.set_item("occupant_id", &participant.occupant_id)?;
            dict.set_item("display_name", &participant.display_name)?;
            dict.set_item("user_id", participant.user_id.as_deref())?;
            Ok(dict)
        })
        .collect::<PyResult<Vec<_>>>()?;
    PyList::new(py, items)
}

#[pymodule]
pub fn ktalk_bot(_py: Python<'_>, module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_class::<KTalkClient>()?;
    module.add_function(wrap_pyfunction!(create_engine, module)?)?;
    Ok(())
}
