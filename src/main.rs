//! # [Ratatui] List example
//!
//! The latest version of this example is available in the [examples] folder in the repository.
//!
//! Please note that the examples are designed to be run against the `main` branch of the Github
//! repository. This means that you may not be able to compile with the latest release version on
//! crates.io, or the one that you have installed locally.
//!
//! See the [examples readme] for more information on finding examples that match the version of the
//! library you are using.
//!
//! [Ratatui]: https://github.com/ratatui-org/ratatui
//! [examples]: https://github.com/ratatui-org/ratatui/blob/main/examples
//! [examples readme]: https://github.com/ratatui-org/ratatui/blob/main/examples/README.md

#![allow(clippy::enum_glob_use, clippy::wildcard_imports)]

use crossterm::ExecutableCommand;
use execute::Execute;
use ratatui::{prelude::*, widgets::*};
use serde::{Deserialize, Serialize};
use std::{error::Error, fs};

use crate::terminal::{init_error_hooks, init_terminal, restore_terminal};
use model::{Config, StatefulList, StoredConnection};

mod utils;
mod model;
mod terminal;
mod ui;

/// This struct holds the current state of the app. In particular, it has the `items` field which is
/// a wrapper around `ListState`. Keeping track of the items state let us render the associated
/// widget with its state and have access to features such as natural scrolling.
///
/// Check the event handling at the bottom to see how to change the state on incoming events.
/// Check the drawing logic for items on how to specify the highlighting style for selected items.
struct App {
    items: StatefulList,
    new_item_popup: bool,
    new_connection: StoredConnection,
}

fn main() -> Result<(), Box<dyn Error>> {
    let cfg: Config = confy::load_path("/home/yenqw/Code/ssh-manager-rust/config.toml")?;
    let file: String = fs::read_to_string(cfg.path_to_data_json)?.parse()?;
    let connections: Vec<StoredConnection> = serde_json::from_str(&file).unwrap();
    // setup terminal
    init_error_hooks()?;
    let terminal = init_terminal()?;

    // create app and run it
    App::new(connections).run(terminal)?;

    restore_terminal()?;

    Ok(())
}
