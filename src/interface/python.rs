use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};

use crate::application::history::{FetchConferenceHistory, FetchConferenceHistoryInput};
use crate::domain::history::{ConferenceHistoryRecord, Participant};

/// Python-facing KTalk client.
#[pyclass]
pub struct KTalkClient {
    auth_file: String,
    history_use_case: FetchConferenceHistory,
}

#[pymethods]
impl KTalkClient {
    #[new]
    #[pyo3(signature = (auth_file = "ktalk_auth.txt".to_owned()))]
    /// Create a client configured with an auth token file.
    pub fn new(auth_file: String) -> Self {
        Self {
            auth_file,
            history_use_case: FetchConferenceHistory::new(),
        }
    }

    /// Fetch conference history records from KTalk.
    #[pyo3(signature = (max_pages = 10, page_size = 25))]
    pub fn get_history<'py>(
        &self,
        py: Python<'py>,
        max_pages: usize,
        page_size: usize,
    ) -> PyResult<Bound<'py, PyList>> {
        let records = self
            .history_use_case
            .execute(FetchConferenceHistoryInput {
                auth_file: self.auth_file.clone(),
                max_pages,
                page_size,
            })
            .map_err(pyo3::PyErr::from)?;

        records_to_pylist(py, &records)
    }
}

/// Fetch conference history records using a single function call.
#[pyfunction]
#[pyo3(signature = (auth_file = "ktalk_auth.txt".to_owned(), max_pages = 10, page_size = 25))]
pub fn get_history<'py>(
    py: Python<'py>,
    auth_file: String,
    max_pages: usize,
    page_size: usize,
) -> PyResult<Bound<'py, PyList>> {
    let records = FetchConferenceHistory::new()
        .execute(FetchConferenceHistoryInput {
            auth_file,
            max_pages,
            page_size,
        })
        .map_err(pyo3::PyErr::from)?;

    records_to_pylist(py, &records)
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

#[pymodule]
pub fn ktalk_bot(_py: Python<'_>, module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_class::<KTalkClient>()?;
    module.add_function(wrap_pyfunction!(get_history, module)?)?;
    Ok(())
}
