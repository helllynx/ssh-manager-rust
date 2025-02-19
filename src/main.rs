use std::env;
use std::{error::Error, fs};

use crate::app::App;
use crate::model::model::{Config, StoredConnection};
use crate::terminal::{init_error_hooks, init_terminal, restore_terminal};

mod app;
mod model;
mod terminal;
mod ui;
mod utils;

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();
    let config_path = args
        .get(1)
        .map(|s| s.as_str())
        .unwrap_or("/home/zybc/Code/ssh-manager-rust/config.toml");

    let cfg: Config = confy::load_path(config_path)?;

    let file = fs::read_to_string(&cfg.path_to_data_json)?;

    let mut connections: Vec<StoredConnection> = serde_json::from_str(&file).unwrap();
    // sort by label
    connections.sort_by_key(|conn| conn.label.clone());

    // setup terminal
    init_error_hooks()?;
    let terminal = init_terminal()?;

    // create app and run it
    App::new(connections).run(terminal, &cfg)?;

    // restore default terminal
    restore_terminal()?;

    Ok(())
}
