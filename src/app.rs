use std::io;
use crossterm::event;
use crossterm::event::{Event, KeyCode, KeyEventKind};
use crossterm::event::KeyCode::{Char, Down, Enter, Esc, Left, Right, Up};
use ratatui::backend::Backend;
use ratatui::Terminal;
use crate::model::model::{StatefulList, StoredConnection};
use crate::terminal::InputMode;

pub(crate) struct App {
    pub(crate) items: StatefulList,
    pub(crate) new_item_popup: bool,
    pub(crate) new_connection: StoredConnection,
    pub(crate) input_mode: InputMode
}

impl App {
    pub(crate) fn run(&mut self, mut terminal: Terminal<impl Backend>) -> io::Result<()> {
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
                        Char('p') => self.new_item_popup = !self.new_item_popup,
                        _ => {
                            if self.new_item_popup {
                                self.handle_new_connection_input(key.code);
                            }
                        }
                    }
                }
            }
        }
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
            KeyCode::Enter => {
                // Save the connection (implement the saving logic)
                self.save_connection();
            }
            _ => {}
        }
    }



    fn save_connection(&self) {
        // Implement the logic to save the connection
        // You can use `self.new_connection` to get the input data
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
