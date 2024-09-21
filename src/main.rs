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

use std::{error::Error, fs, io, io::stdout};
use std::net::IpAddr;
use std::process::Command;
use std::process::exit;
use std::time::Duration;

use color_eyre::config::HookBuilder;
use crossterm::{event::{self, Event, KeyCode, KeyEventKind}, ExecutableCommand, terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen}};
use crossterm::cursor::{EnableBlinking, Hide, SetCursorStyle, Show};
use execute::Execute;
use ratatui::{prelude::*, style::palette::tailwind, widgets::*};
use serde::{Deserialize, Serialize};

use model::{Config, ConnectionItem, StatefulList, Status, StoredConnection};
use rand::random;

mod utils;
mod model;

const APP_HEADER_BG: Color = tailwind::BLACK;
const NORMAL_ROW_COLOR: Color = tailwind::SLATE.c950;
const ALT_ROW_COLOR: Color = tailwind::SLATE.c900;
const SELECTED_STYLE_FG: Color = tailwind::YELLOW.c700;
const TEXT_COLOR: Color = tailwind::SLATE.c200;
const NOT_AVAILABLE_TEXT_COLOR: Color = tailwind::RED.c500;

/// This struct holds the current state of the app. In particular, it has the `items` field which is
/// a wrapper around `ListState`. Keeping track of the items state let us render the associated
/// widget with its state and have access to features such as natural scrolling.
///
/// Check the event handling at the bottom to see how to change the state on incoming events.
/// Check the drawing logic for items on how to specify the highlighting style for selected items.
struct App {
    items: StatefulList,
    new_item_popup: bool,
}

fn main() -> Result<(), Box<dyn Error>> {
    let cfg: Config = confy::load_path("/home/zybc/Code/ssh-manager-rust/config.toml")?;
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

fn init_error_hooks() -> color_eyre::Result<()> {
    let (panic, error) = HookBuilder::default().into_hooks();
    let panic = panic.into_panic_hook();
    let error = error.into_eyre_hook();
    color_eyre::eyre::set_hook(Box::new(move |e| {
        let _ = restore_terminal();
        error(e)
    }))?;
    std::panic::set_hook(Box::new(move |info| {
        let _ = restore_terminal();
        panic(info);
    }));
    Ok(())
}

fn init_terminal() -> color_eyre::Result<Terminal<impl Backend>> {
    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout());
    let terminal = Terminal::new(backend)?;
    Ok(terminal)
}

fn restore_terminal() -> color_eyre::Result<()> {
    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;
    Ok(())
}

fn enable_cursor() {
    stdout().execute(Show).unwrap();
    stdout().execute(EnableBlinking).unwrap();
    stdout().execute(SetCursorStyle::BlinkingBar).unwrap();
}

fn disable_cursor() {
    stdout().execute(Hide).unwrap();
}

impl App {
    fn new(connections: Vec<StoredConnection>) -> Self {
        Self {
            items: StatefulList::with_items(connections),
            new_item_popup: false,
        }
    }

    /// Changes the status of the selected list item
    fn connect_ssh(&mut self) {
        if let Some(i) = self.items.state.selected() {
            match self.items.items[i].status {
                Status::Available => {
                    restore_terminal().unwrap();
                    enable_cursor();
                    let output = if !self.items.items[i].password.is_empty() {
                        let ssh_command = format!(
                            "sshpass -p {} ssh -o StrictHostKeyChecking=no {}@{} -p {}",
                            self.items.items[i].password,
                            self.items.items[i].user,
                            self.items.items[i].host,
                            self.items.items[i].port
                        );
                        Command::new("sh")
                            .arg("-c")
                            .arg(ssh_command)
                            .execute_output().unwrap()
                    } else {
                        Command::new("ssh")
                            .arg("-o ServerAliveInterval=15")
                            .arg("-o ServerAliveCountMax=3")
                            .arg("-o StrictHostKeyChecking=no")
                            .arg(format!("{}@{}", self.items.items[i].user, self.items.items[i].host))
                            .arg(format!("-p {}", self.items.items[i].port))
                            .execute_output().unwrap()
                    };

                    if let Some(exit_code) = output.status.code() {
                        println!("{}", output.status);
                        if exit_code != 0 {
                            println!("stderr: {}", String::from_utf8_lossy(&output.stderr));
                        }
                    } else {
                        eprintln!("Interrupted!");
                    };
                    exit(0);
                }
                Status::NotAvailable => {}
            }
        }
        disable_cursor();
    }

    fn connect_sshfs(&mut self) {
        if let Some(i) = self.items.state.selected() {
            match self.items.items[i].status {
                Status::Available => {
                    restore_terminal().unwrap();

                    let mount_path = format!("/tmp/{}", utils::remove_whitespace(self.items.items[i].label.as_str()));
                    fs::create_dir_all(&mount_path).expect(&format!("Can't create temp directory {}", mount_path));

                    let output = Command::new("sshfs")
                        // .arg("-o reconnect")
                        // .arg("-o ServerAliveInterval=15")
                        // .arg("-o ServerAliveCountMax=3")
                        .arg(format!("{}@{}:/", self.items.items[i].user, self.items.items[i].host))
                        .arg(mount_path)
                        .arg(format!("-p {}", self.items.items[i].port))
                        .execute_output().unwrap();

                    if let Some(exit_code) = output.status.code() {
                        if exit_code == 0 {
                            println!("Ok.");
                        } else {
                            eprintln!("Failed.");
                            println!("stderr: {}", String::from_utf8_lossy(&output.stderr));
                        }
                    } else {
                        eprintln!("Interrupted!");
                    };
                    exit(0);
                }
                Status::NotAvailable => {}
            }
        }
    }

    fn go_top(&mut self) {
        self.items.state.select(Some(0));
    }

    fn go_bottom(&mut self) {
        self.items.state.select(Some(self.items.items.len() - 1));
    }
}

impl App {
    fn run(&mut self, mut terminal: Terminal<impl Backend>) -> io::Result<()> {
        loop {
            if !self.new_item_popup {
                self.draw_main_layout(&mut terminal)?;
            } else {
                self.draw_popup(&mut terminal)?;
            }

            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    use KeyCode::*;
                    match key.code {
                        Char('q') | Esc => return Ok(()),
                        Char('h') | Left => self.items.unselect(),
                        Char('j') | Down => self.items.next(),
                        Char('k') | Up => self.items.previous(),
                        Char('l') | Right | Enter => self.connect_ssh(),
                        Char('f') => self.connect_sshfs(),
                        Char('g') => self.go_top(),
                        Char('G') => self.go_bottom(),
                        Char('p') => self.new_item_popup = if self.new_item_popup { false } else { true },
                        _ => {}
                    }
                }
            }
        }
    }

    fn draw_main_layout(&mut self, terminal: &mut Terminal<impl Backend>) -> io::Result<()> {
        terminal.draw(|f| f.render_widget(self, f.size()))?;
        Ok(())
    }

    fn draw_popup(&mut self, terminal: &mut Terminal<impl Backend>) -> io::Result<()> {
        terminal.draw(|f| ui(f, &self))?;
        Ok(())
    }
}

impl Widget for &mut App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Create a space for header, todo list and the footer.
        let vertical = Layout::vertical([
            Constraint::Length(2),
            Constraint::Min(0),
            Constraint::Length(2),
        ]);
        let [header_area, rest_area, footer_area] = vertical.areas(area);

        // Create two chunks with equal vertical screen space. One for the list and the other for
        // the info block.
        let vertical = Layout::horizontal([Constraint::Percentage(70), Constraint::Percentage(30)]);
        let [upper_item_list_area, lower_item_list_area] = vertical.areas(rest_area);

        render_title(header_area, buf);
        self.render_app(upper_item_list_area, buf);
        self.render_info(lower_item_list_area, buf);
        render_footer(footer_area, buf);
    }
}

impl App {
    fn render_app(&mut self, area: Rect, buf: &mut Buffer) {
        // We create two blocks, one is for the header (outer) and the other is for list (inner).
        let outer_block = Block::default()
            .borders(Borders::NONE)
            .fg(TEXT_COLOR)
            .bg(APP_HEADER_BG)
            .title("Connections list")
            .title_alignment(Alignment::Center);
        let inner_block = Block::default()
            .borders(Borders::NONE)
            .fg(TEXT_COLOR)
            .bg(NORMAL_ROW_COLOR);

        // We get the inner area from outer_block. We'll use this area later to render the table.
        let outer_area = area;
        let inner_area = outer_block.inner(outer_area);

        // We can render the header in outer_area.
        outer_block.render(outer_area, buf);

        // Iterate through all elements in the `items` and stylize them.
        let items: Vec<ListItem> = self
            .items
            .items
            .iter()
            .enumerate()
            .map(|(i, connection_item)| connection_item.to_list_item(i))
            .collect();

        // Create a List from all list items and highlight the currently selected one
        let items = List::new(items)
            .block(inner_block)
            .highlight_style(
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .add_modifier(Modifier::REVERSED)
                    .fg(SELECTED_STYLE_FG),
            )
            .highlight_symbol(">")
            .highlight_spacing(HighlightSpacing::Always);

        // We can now render the item list
        // (look careful we are using StatefulWidget's render.)
        // ratatui::widgets::StatefulWidget::render as stateful_render
        StatefulWidget::render(items, inner_area, buf, &mut self.items.state);
    }

    fn render_info(&self, area: Rect, buf: &mut Buffer) {
        // We get the info depending on the item's state.
        let info = if let Some(i) = self.items.state.selected() {
            match self.items.items[i].status {
                Status::Available => self.items.items[i].display(),
                Status::NotAvailable => "NotAvailable - ".to_string() + self.items.items[i].host.as_str(),
            }
        } else {
            "Please select the connection".to_string()
        };

        // We show the list item's info under the list in this paragraph
        let outer_info_block = Block::default()
            .borders(Borders::NONE)
            .fg(TEXT_COLOR)
            .bg(APP_HEADER_BG)
            .title("Connection info")
            .title_alignment(Alignment::Center);
        let inner_info_block = Block::default()
            .borders(Borders::NONE)
            .bg(NORMAL_ROW_COLOR)
            .padding(Padding::horizontal(1));

        // This is a similar process to what we did for list. outer_info_area will be used for
        // header inner_info_area will be used for the list info.
        let outer_info_area = area;
        let inner_info_area = outer_info_block.inner(outer_info_area);

        // We can render the header. Inner info will be rendered later
        outer_info_block.render(outer_info_area, buf);

        let info_paragraph = Paragraph::new(info)
            .block(inner_info_block)
            .fg(TEXT_COLOR)
            .wrap(Wrap { trim: false });

        // We can now render the item info
        info_paragraph.render(inner_info_area, buf);
    }
}

fn render_title(area: Rect, buf: &mut Buffer) {
    Paragraph::new("SSH Manager")
        .bold()
        .centered()
        .render(area, buf);
}

fn render_footer(area: Rect, buf: &mut Buffer) {
    Paragraph::new("\nUse ↓↑ to move, ← to unselect, → to change status, g/G to go top/bottom. f to sshfs.")
        .centered()
        .render(area, buf);
}


// https://github.com/TheAwiteb/ratatui-textarea/blob/main/examples/single_line.rs
fn ui(f: &mut Frame, app: &App) {
    let area = f.size();

    let vertical = Layout::vertical([Constraint::Percentage(20), Constraint::Percentage(80)]);
    let [instructions, content] = vertical.areas(area);

    let text = if app.new_item_popup {
        "Press p to close the popup"
    } else {
        "Press p to show the popup"
    };
    let paragraph = Paragraph::new(text.slow_blink())
        .centered()
        .wrap(Wrap { trim: true });
    f.render_widget(paragraph, instructions);

    let block = Block::default()
        .title("Content")
        .borders(Borders::ALL)
        .on_blue();
    f.render_widget(block, content);

    if app.new_item_popup {
        let block = Block::default().title("Popup").borders(Borders::ALL);
        let area = centered_rect(60, 20, area);
        f.render_widget(Clear, area); //this clears out the background
        f.render_widget(block, area);
    }
}

/// helper function to create a centered rect using up certain percentage of the available rect `r`
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::vertical([
        Constraint::Percentage((100 - percent_y) / 2),
        Constraint::Percentage(percent_y),
        Constraint::Percentage((100 - percent_y) / 2),
    ])
        .split(r);

    Layout::horizontal([
        Constraint::Percentage((100 - percent_x) / 2),
        Constraint::Percentage(percent_x),
        Constraint::Percentage((100 - percent_x) / 2),
    ])
        .split(popup_layout[1])[1]
}


impl StatefulList {
    fn with_items(items: Vec<StoredConnection>) -> StatefulList {
        let a = items.into_iter().map(|item|
            ConnectionItem {
                label: item.label,
                host: item.host,
                port: item.port.unwrap_or("22".parse().unwrap()), // TODO maybe configure default port and default user
                user: item.user.unwrap_or("root".parse().unwrap()),
                password: if let Some(password) = item.password {
                    password
                } else {
                    "".parse().unwrap()
                },
                details: "".parse().unwrap(),
                status: Status::Available,
            }
        ).collect::<Vec<ConnectionItem>>();

        StatefulList {
            state: ListState::default(),
            items: a,
            last_selected: None,
        }
    }

    fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => self.last_selected.unwrap_or(0),
        };
        self.state.select(Some(i));
    }

    fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.items.len() - 1
                } else {
                    i - 1
                }
            }
            None => self.last_selected.unwrap_or(0),
        };
        self.state.select(Some(i));
    }

    fn unselect(&mut self) {
        let offset = self.state.offset();
        self.last_selected = self.state.selected();
        self.state.select(None);
        *self.state.offset_mut() = offset;
    }
}


fn ping_host(ip: &str) -> bool {
    let output = if cfg!(target_os = "windows") {
        Command::new("ping")
            .arg("-n")
            .arg("1")
            .arg(ip)
            .output()
    } else {
        Command::new("ping")
            .arg("-c")
            .arg("1")
            .arg(ip)
            .output()
    };

    match output {
        Ok(output) => output.status.success(),
        Err(_) => false,
    }
}


impl ConnectionItem {
    fn to_list_item(&self, index: usize) -> ListItem {
        ping_host(&self.host);
        let bg_color = match index % 2 {
            0 => NORMAL_ROW_COLOR,
            _ => ALT_ROW_COLOR,
        };
        let line = match self.status {
            Status::Available => Line::styled(format!(" > {} {}", self.label, self.host), TEXT_COLOR),
            Status::NotAvailable => Line::styled(
                format!(" X {} {}", self.label, self.host),
                (NOT_AVAILABLE_TEXT_COLOR, bg_color),
            ),
        };

        ListItem::new(line).bg(bg_color)
    }

    fn display(&self) -> String {
        let info = format!(
            "label: {}\n\
             host: {}\n\
             port: {}\n\
             user: {}\n\
             details: {}\n",
            self.label, self.host, self.port, self.user, self.details);
        return info;
    }
}
