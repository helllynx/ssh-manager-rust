use crate::ui::style::{ALT_ROW_COLOR, NORMAL_ROW_COLOR, NOT_AVAILABLE_TEXT_COLOR, TEXT_COLOR};
use ratatui::prelude::Line;
use ratatui::style::Stylize;
use ratatui::widgets::{ListItem, ListState};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::fmt::{Display, Formatter};

#[derive(Default, Debug, Serialize, Deserialize)]
pub(crate) struct Config {
    pub(crate) path_to_data_json: String,
}

// impl Default for Config {
//     fn default() -> Self {
//         Self { path_to_data_json: "data/".to_string() }
//     }
// }

#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct ConnectionItem {
    pub(crate) label: String,
    pub(crate) host: String,
    pub(crate) port: String,
    pub(crate) user: String,
    pub(crate) password: String,
    pub(crate) details: String,
    pub(crate) status: Status,
}

impl ConnectionItem {
    pub(crate) fn to_list_item(&self, index: usize) -> ListItem {
        let bg_color = match index % 2 {
            0 => NORMAL_ROW_COLOR,
            _ => ALT_ROW_COLOR,
        };
        let line = match self.status {
            Status::Available => {
                Line::styled(format!(" > {} {}", self.label, self.host), TEXT_COLOR)
            }
            Status::NotAvailable => Line::styled(
                format!(" X {} {}", self.label, self.host),
                (NOT_AVAILABLE_TEXT_COLOR, bg_color),
            ),
        };

        ListItem::new(line).bg(bg_color)
    }

    pub(crate) fn display(&self) -> String {
        let info = format!(
            "label: {}\n\
             host: {}\n\
             port: {}\n\
             user: {}\n\
             details: {}\n",
            self.label, self.host, self.port, self.user, self.details
        );
        info
    }
}

pub(crate) struct StatefulList {
    pub(crate) state: ListState,
    pub(crate) items: Vec<ConnectionItem>,
    pub(crate) last_selected: Option<usize>,
}

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct StoredConnection {
    pub(crate) label: String,
    pub(crate) host: String,
    pub(crate) port: Option<String>,
    pub(crate) user: Option<String>,
    pub(crate) password: Option<String>,
    pub(crate) details: Option<String>,
}

impl Display for StoredConnection {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Label: {}, Host: {}, Port: {}, User: {}, Password: {}, Details: {}",
            self.label,
            self.host,
            self.port.as_deref().unwrap_or("None"),
            self.user.as_deref().unwrap_or("None"),
            self.password.as_deref().unwrap_or("None"),
            self.details.as_deref().unwrap_or("None"),
        )
    }
}

impl StoredConnection {
    pub(crate) fn new() -> Self {
        Self {
            label: String::new(),
            host: String::new(),
            port: Option::from(String::from("22")),
            user: Option::from(String::new()),
            password: Option::from(String::new()),
            details: Option::from(String::new()),
        }
    }
}

#[derive(Copy, Clone, Serialize, Deserialize)]
pub(crate) enum Status {
    Available,
    NotAvailable,
}

impl From<StoredConnection> for ConnectionItem {
    fn from(stored: StoredConnection) -> Self {
        ConnectionItem {
            label: stored.label,
            host: stored.host,
            port: stored.port.unwrap_or_else(|| "22".to_string()),
            user: stored.user.unwrap_or_else(String::new),
            password: stored.password.unwrap_or_else(String::new),
            details: stored.details.unwrap_or_else(String::new),
            status: Status::Available,
        }
    }
}

impl From<ConnectionItem> for StoredConnection {
    fn from(connection: ConnectionItem) -> Self {
        StoredConnection {
            label: connection.label,
            host: connection.host,
            port: Some(connection.port),
            user: Some(connection.user),
            password: Some(connection.password),
            details: Some(connection.details),
        }
    }
}
