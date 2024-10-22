use std::process::exit;
use std::process::Command;
use std::{fs, io, io::stdout};

use crate::ui::style::{APP_HEADER_BG, NORMAL_ROW_COLOR, SELECTED_STYLE_FG, TEXT_COLOR};
use crate::{utils};
use color_eyre::config::HookBuilder;
use crossterm::cursor::{EnableBlinking, Hide, SetCursorStyle, Show};
use crossterm::{event::{self, Event, KeyCode, KeyEventKind}, terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen}, ExecutableCommand};
use execute::Execute;
use ratatui::{prelude::*, widgets::*};
use crate::app::App;
use crate::model::model::{ConnectionItem, StatefulList, Status, StoredConnection};

pub(crate) fn init_error_hooks() -> color_eyre::Result<()> {
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

pub(crate) fn init_terminal() -> color_eyre::Result<Terminal<impl Backend>> {
    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout());
    let terminal = Terminal::new(backend)?;
    Ok(terminal)
}

pub(crate) fn restore_terminal() -> color_eyre::Result<()> {
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
    pub(crate) fn new(connections: Vec<StoredConnection>) -> Self {
        Self {
            items: StatefulList::with_items(connections),
            new_item_popup: false,
            new_connection: StoredConnection::new(),
            input_mode: InputMode::Label,
        }
    }

    /// Changes the status of the selected list item
    pub(crate) fn connect_ssh(&mut self) {
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

    pub(crate) fn connect_sshfs(&mut self) {
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

    pub(crate) fn go_top(&mut self) {
        self.items.state.select(Some(0));
    }

    pub(crate) fn go_bottom(&mut self) {
        self.items.state.select(Some(self.items.items.len() - 1));
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
pub(crate) fn add_new_connection_ui(f: &mut Frame, app: &App) {
    let area = f.size();

    let vertical = Layout::vertical([Constraint::Percentage(10), Constraint::Percentage(80)]);
    let [instructions, content] = vertical.areas(area);

    let text = if app.new_item_popup {
        "Press Enter to save, Esc to cancel"
    } else {
        "Press p to add new connection"
    };
    let paragraph = Paragraph::new(text)
        .centered()
        .wrap(Wrap { trim: true });
    f.render_widget(paragraph, instructions);

    if app.new_item_popup {
        let block = Block::default().title("New Connection").borders(Borders::ALL);
        let area = centered_rect(60, 50, area);
        f.render_widget(Clear, area); // clear background for popup

        let input_layout = Layout::vertical([
            Constraint::Percentage(15),
            Constraint::Percentage(15),
            Constraint::Percentage(15),
            Constraint::Percentage(15),
            Constraint::Percentage(15),
        ])
            .split(area);

        // Render input fields
        let label_field = Paragraph::new(app.new_connection.label.to_string())
            .block(Block::default().title("Label").borders(Borders::ALL))
            .style(match app.input_mode {
                InputMode::Label => Style::default().fg(Color::Yellow),
                _ => Style::default(),
            });
        f.render_widget(label_field, input_layout[0]);

        let host_field = Paragraph::new(app.new_connection.host.as_str())
            .block(Block::default().title("Host").borders(Borders::ALL))
            .style(match app.input_mode {
                InputMode::Host => Style::default().fg(Color::Yellow),
                _ => Style::default(),
            });
        f.render_widget(host_field, input_layout[1]);

        let port_field = Paragraph::new(app.new_connection.port.as_ref().unwrap().to_string())
            .block(Block::default().title("Port").borders(Borders::ALL))
            .style(match app.input_mode {
                InputMode::Port => Style::default().fg(Color::Yellow),
                _ => Style::default(),
            });
        f.render_widget(port_field, input_layout[2]);

        let user_field = Paragraph::new(app.new_connection.user.as_ref().unwrap().to_string())
            .block(Block::default().title("User").borders(Borders::ALL))
            .style(match app.input_mode {
                InputMode::User => Style::default().fg(Color::Yellow),
                _ => Style::default(),
            });
        f.render_widget(user_field, input_layout[3]);

        let password_field = Paragraph::new(app.new_connection.password.as_ref().unwrap().to_string())
            .block(Block::default().title("Password").borders(Borders::ALL))
            .style(match app.input_mode {
                InputMode::Password => Style::default().fg(Color::Yellow),
                _ => Style::default(),
            });
        f.render_widget(password_field, input_layout[4]);
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

    pub(crate) fn next(&mut self) {
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

    pub(crate) fn previous(&mut self) {
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

    pub(crate) fn unselect(&mut self) {
        let offset = self.state.offset();
        self.last_selected = self.state.selected();
        self.state.select(None);
        *self.state.offset_mut() = offset;
    }
}

#[derive(Copy, Clone)]
pub(crate) enum InputMode {
    Label,
    Host,
    Port,
    User,
    Password,
}
