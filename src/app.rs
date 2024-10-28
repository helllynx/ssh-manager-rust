use crate::model::model::{Config, StatefulList, StoredConnection};
use crate::terminal::{centered_rect, InputMode};
use crate::utils::{append_json_to_file, edit_connection_and_save, write_json_to_file};
use crossterm::event;
use crossterm::event::{Event, KeyCode, KeyEventKind};
use ratatui::backend::Backend;
use ratatui::widgets::{Block, BorderType, Borders, Clear, Paragraph, Wrap};
use ratatui::Terminal;
use std::io;

pub(crate) struct App {
    pub(crate) items: StatefulList,
    pub(crate) new_item_popup: bool,
    pub(crate) is_edit_mode: bool,
    pub(crate) new_connection: StoredConnection,
    pub(crate) input_mode: InputMode,
}

impl App {
    pub(crate) fn run(&mut self, mut terminal: Terminal<impl Backend>, cfg: &Config) -> io::Result<()> {
        loop {
            if !self.new_item_popup {
                self.draw_main_layout(&mut terminal)?;
            } else {
                self.draw_popup(&mut terminal)?;
            }

            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    use KeyCode::*;

                    if self.new_item_popup {
                        match key.code {
                            Esc => {
                                self.new_item_popup = false;
                            }
                            Enter => {
                                if self.is_edit_mode {
                                    self.edit_connection(&cfg.path_to_data_json);
                                } else {
                                    self.save_connection(&cfg.path_to_data_json);
                                }
                                self.new_item_popup = false;
                            }
                            _ => {
                                self.handle_new_connection_input(key.code);
                            }
                        }
                    } else {
                        match key.code {
                            Char('q') | Esc => return Ok(()),
                            Char('h') | Left => self.items.unselect(),
                            Char('j') | Down => self.items.next(),
                            Char('k') | Up => self.items.previous(),
                            Char('l') | Right => self.connect_ssh(),
                            Char('f') => self.connect_sshfs(),
                            Char('g') => self.go_top(),
                            Char('G') => self.go_bottom(),
                            Char('n') => self.new_item_popup = !self.new_item_popup,
                            Char('e') => {
                                self.start_editing_connection();
                                self.new_item_popup = true;
                            }
                            Char('d') => {
                                self.confirm_deletion(&mut terminal, &cfg.path_to_data_json);
                            }
                            Enter => self.connect_ssh(),
                            _ => {}
                        }
                    }
                }
            }
        }
    }

    pub(crate) fn start_editing_connection(&mut self) {
        if let Some(selected) = self.items.state.selected() {
            let current_connection = self.items.items[selected].clone();
            self.new_connection = StoredConnection::from(current_connection);
            self.is_edit_mode = true;
        }
    }

    fn edit_connection(&mut self, path: &String) {
        let host = self.new_connection.host.clone();
        if let Err(e) = edit_connection_and_save(&self.new_connection, path, &host) {
            eprintln!("Failed to replace data in file: {}", e);
        }
        self.new_connection = StoredConnection::new();
        self.reload_connections_from_file(path);
        self.is_edit_mode = false;
    }

    fn handle_new_connection_input(&mut self, code: KeyCode) {
        match code {
            KeyCode::Char(c) => {
                match self.input_mode {
                    InputMode::Label => self.new_connection.label.push(c),
                    InputMode::Host => self.new_connection.host.push(c),
                    InputMode::Port => {
                        if self.new_connection.port.is_none() {
                            self.new_connection.port = Some(String::new());
                        }
                        if let Some(port) = self.new_connection.port.as_mut() {
                            port.push(c);
                        }
                    }
                    InputMode::User => {
                        if self.new_connection.user.is_none() {
                            self.new_connection.user = Some(String::new());
                        }
                        if let Some(user) = self.new_connection.user.as_mut() {
                            user.push(c);
                        }
                    }
                    InputMode::Password => {
                        if self.new_connection.password.is_none() {
                            self.new_connection.password = Some(String::new());
                        }
                        if let Some(password) = self.new_connection.password.as_mut() {
                            password.push(c);
                        }
                    }
                }
            }
            KeyCode::Backspace => {
                match self.input_mode {
                    InputMode::Label => { self.new_connection.label.pop(); }
                    InputMode::Host => { self.new_connection.host.pop(); }
                    InputMode::Port => {
                        if let Some(port) = self.new_connection.port.as_mut() {
                            port.pop();
                        }
                    }
                    InputMode::User => {
                        if let Some(user) = self.new_connection.user.as_mut() {
                            user.pop();
                        }
                    }
                    InputMode::Password => {
                        if let Some(password) = self.new_connection.password.as_mut() {
                            password.pop();
                        }
                    }
                }
            }
            KeyCode::Tab => {
                // Switch between input fields
                self.input_mode = match self.input_mode {
                    InputMode::Label => InputMode::Host,
                    InputMode::Host => InputMode::Port,
                    InputMode::Port => InputMode::User,
                    InputMode::User => InputMode::Password,
                    InputMode::Password => InputMode::Label,
                };
            }
            _ => {}
        }
    }

    fn save_connection(&mut self, path: &String) {
        if let Err(e) = append_json_to_file(&self.new_connection, path) {
            eprintln!("Failed to write to file: {}", e);
            return;
        }
        self.reload_connections_from_file(path)
    }

    fn reload_connections_from_file(&mut self, path: &String) {
        if let Ok(content) = std::fs::read_to_string(path) {
            match serde_json::from_str(&content) {
                Ok(items) => {
                    self.items = StatefulList::with_items(items);
                    self.new_connection = StoredConnection::new()
                }
                Err(e) => eprintln!("Failed to parse JSON: {}", e),
            }
        } else {
            eprintln!("Failed to read file: {}", path);
        }
    }

    fn confirm_deletion<B: Backend>(&mut self, terminal: &mut Terminal<B>, path: &String) {
        if let Some(selected) = self.items.state.selected() {
            let connection = &self.items.items[selected];
            let confirm_area = centered_rect(50, 20, terminal.size().unwrap());

            let block = Block::default()
                .title("Confirm Deletion")
                .borders(Borders::ALL)
                .border_type(BorderType::Plain);

            let paragraph = Paragraph::new(format!(
                "Are you sure you want to delete the connection '{}'? (y/n)",
                connection.label
            ))
                .block(block)
                .wrap(Wrap { trim: true });

            terminal.draw(|f| {
                f.render_widget(Clear, confirm_area); // Clear the area for the popup
                f.render_widget(paragraph, confirm_area);
            }).unwrap();

            if let Event::Key(key) = event::read().unwrap() {
                match key.code {
                    KeyCode::Char('y') => {
                        self.delete_connection(path, selected);
                    }
                    KeyCode::Char('n') => {
                        self.draw_main_layout(terminal).unwrap();
                    }
                    _ => {
                        self.draw_main_layout(terminal).unwrap();
                    }
                }
            }
        }
    }

    fn delete_connection(&mut self, path: &String, index: usize) {
        self.items.items.remove(index);
        if let Err(e) = write_json_to_file(&self.items.items, path) {
            eprintln!("Failed to write to file: {}", e);
        }
        self.reload_connections_from_file(path);
    }

    fn save_all_connections(&self, path: &String) {
        if let Err(e) = write_json_to_file(&self.items.items, path) {
            eprintln!("Failed to write to file: {}", e);
        }
    }

    fn draw_main_layout(&mut self, terminal: &mut Terminal<impl Backend>) -> io::Result<()> {
        terminal.draw(|f| f.render_widget(self, f.size()))?;
        Ok(())
    }

    fn draw_popup(&mut self, terminal: &mut Terminal<impl Backend>) -> io::Result<()> {
        terminal.draw(|f| crate::terminal::add_new_connection_ui(f, &self))?;
        Ok(())
    }
}
