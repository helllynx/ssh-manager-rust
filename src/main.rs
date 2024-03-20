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
use std::process::Command;
use std::process::exit;

use color_eyre::config::HookBuilder;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    ExecutableCommand,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use execute::Execute;
use ratatui::{prelude::*, style::palette::tailwind, widgets::*};

mod utils;

const TODO_HEADER_BG: Color = tailwind::BLACK;
const NORMAL_ROW_COLOR: Color = tailwind::SLATE.c950;
const ALT_ROW_COLOR: Color = tailwind::SLATE.c900;
const SELECTED_STYLE_FG: Color = tailwind::YELLOW.c700;
const TEXT_COLOR: Color = tailwind::SLATE.c200;
const NOT_AVAILABLE_TEXT_COLOR: Color = tailwind::RED.c500;

#[derive(Copy, Clone)]
enum Status {
    Available,
    NotAvailable,
}

struct ConnectionItem<'a> {
    label: &'a str,
    host: &'a str,
    port: &'a u16,
    user: &'a str,
    password: &'a str,
    details: &'a str,
    status: Status,
}

struct StatefulList<'a> {
    state: ListState,
    items: Vec<ConnectionItem<'a>>,
    last_selected: Option<usize>,
}

/// This struct holds the current state of the app. In particular, it has the `items` field which is
/// a wrapper around `ListState`. Keeping track of the items state let us render the associated
/// widget with its state and have access to features such as natural scrolling.
///
/// Check the event handling at the bottom to see how to change the state on incoming events.
/// Check the drawing logic for items on how to specify the highlighting style for selected items.
struct App<'a> {
    items: StatefulList<'a>,
}

fn main() -> Result<(), Box<dyn Error>> {
    // setup terminal
    init_error_hooks()?;
    let terminal = init_terminal()?;

    // create app and run it
    App::new().run(terminal)?;

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

impl<'a> App<'a> {
    fn new() -> Self {
        Self {
            items: StatefulList::with_items(vec![
                ConnectionItem {label: "Local server 0", host: "localhost", port: &22, user: "root", password: "", details: "This is some server", status: Status::Available},
            ]),
        }
    }

    /// Changes the status of the selected list item
    fn connect(&mut self, sshfs: bool) {
        if let Some(i) = self.items.state.selected() {
            match self.items.items[i].status {
                Status::Available => {
                    restore_terminal().unwrap();
                    let output = if sshfs {
                        let mount_path = format!("/tmp/{}", utils::remove_whitespace(self.items.items[i].label));
                        fs::create_dir_all(&mount_path).expect(&format!("Can't create temp directory {}", mount_path));

                        Command::new("sshfs")
                            // .arg("-o reconnect")
                            // .arg("-o ServerAliveInterval=15")
                            // .arg("-o ServerAliveCountMax=3")
                            .arg(format!("{}@{}:/", self.items.items[i].user, self.items.items[i].host))
                            .arg(mount_path)
                            .arg(format!("-p {}", self.items.items[i].port))
                            .execute_output().unwrap()
                    } else {
                        let command =  Command::new("ssh")
                            .arg("-o ServerAliveInterval=15")
                            .arg("-o ServerAliveCountMax=3")
                            .arg(format!("{}@{}", self.items.items[i].user, self.items.items[i].host))
                            .arg(format!("-p {}", self.items.items[i].port))
                            .execute_output().unwrap();
                        command
                        //TODO implement sshpass
                        // command.execute_output().unwrap()
                    };

                    if let Some(exit_code) = output.status.code() {
                        if exit_code == 0 {
                            println!("Ok.");
                        } else {
                            eprintln!("Failed.");
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

impl App<'_> {
    fn run(&mut self, mut terminal: Terminal<impl Backend>) -> io::Result<()> {
        loop {
            self.draw(&mut terminal)?;

            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    use KeyCode::*;
                    match key.code {
                        Char('q') | Esc => return Ok(()),
                        Char('h') | Left => self.items.unselect(),
                        Char('j') | Down => self.items.next(),
                        Char('k') | Up => self.items.previous(),
                        Char('l') | Right | Enter => self.connect(false),
                        Char('f') => self.connect(true),
                        Char('g') => self.go_top(),
                        Char('G') => self.go_bottom(),
                        _ => {}
                    }
                }
            }
        }
    }

    fn draw(&mut self, terminal: &mut Terminal<impl Backend>) -> io::Result<()> {
        terminal.draw(|f| f.render_widget(self, f.size()))?;
        Ok(())
    }
}

impl Widget for &mut App<'_> {
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
        let vertical = Layout::vertical([Constraint::Percentage(50), Constraint::Percentage(50)]);
        let [upper_item_list_area, lower_item_list_area] = vertical.areas(rest_area);

        render_title(header_area, buf);
        self.render_todo(upper_item_list_area, buf);
        self.render_info(lower_item_list_area, buf);
        render_footer(footer_area, buf);
    }
}

impl App<'_> {
    fn render_todo(&mut self, area: Rect, buf: &mut Buffer) {
        // We create two blocks, one is for the header (outer) and the other is for list (inner).
        let outer_block = Block::default()
            .borders(Borders::NONE)
            .fg(TEXT_COLOR)
            .bg(TODO_HEADER_BG)
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
            .map(|(i, todo_item)| todo_item.to_list_item(i))
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
                Status::NotAvailable => "TODO: ".to_string() + self.items.items[i].host,
            }
        } else {
            "Nothing to see here...".to_string()
        };

        // We show the list item's info under the list in this paragraph
        let outer_info_block = Block::default()
            .borders(Borders::NONE)
            .fg(TEXT_COLOR)
            .bg(TODO_HEADER_BG)
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

impl StatefulList<'_> {
    fn with_items(items: Vec<ConnectionItem>) -> StatefulList {
        StatefulList {
            state: ListState::default(),
            items: items,
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

impl ConnectionItem<'_> {
    fn to_list_item(&self, index: usize) -> ListItem {
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
