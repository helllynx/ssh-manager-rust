use ratatui::widgets::ListState;
use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Serialize, Deserialize)]
pub(crate) struct Config {
    pub(crate) path_to_data_json: String,
}

// impl Default for Config {
//     fn default() -> Self {
//         Self { path_to_data_json: "data/".to_string() }
//     }
// }

pub(crate) struct ConnectionItem {
    pub(crate) label: String,
    pub(crate) host: String,
    pub(crate) port: String,
    pub(crate) user: String,
    pub(crate) password: String,
    pub(crate) details: String,
    pub(crate) status: Status,
}

pub(crate) struct StatefulList {
    pub(crate) state: ListState,
    pub(crate) items: Vec<ConnectionItem>,
    pub(crate) last_selected: Option<usize>,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct StoredConnection {
    pub(crate) label: String,
    pub(crate) host: String,
    pub(crate) port: Option<String>,
    pub(crate) user: Option<String>,
    pub(crate) password: Option<String>,
}

#[derive(Copy, Clone)]
pub(crate) enum Status {
    Available,
    NotAvailable,
}
