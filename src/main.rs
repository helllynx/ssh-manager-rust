use std::{error::Error, fs};
use crate::model::model::{Config, StatefulList, StoredConnection};
use crate::terminal::{init_error_hooks, init_terminal, restore_terminal};

mod utils;
mod model;
mod terminal;
mod ui;

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

    // restore default terminal
    restore_terminal()?;

    Ok(())
}
