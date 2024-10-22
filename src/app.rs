use std::io;
use crossterm::event;
use crossterm::event::{Event, KeyCode, KeyEventKind};
use crossterm::event::KeyCode::{Char, Down, Enter, Esc, Left, Right, Up};
use ratatui::backend::Backend;
use ratatui::Terminal;
use crate::model::model::{StatefulList, StoredConnection};

pub(crate) struct App {
    pub(crate) items: StatefulList,
    pub(crate) new_item_popup: bool,
    pub(crate) new_connection: StoredConnection,
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
                // Add character to the corresponding field
                // Here you would implement logic to determine which field is currently active
                self.new_connection.label.push(c); // Example for label field
            }
            KeyCode::Backspace => {
                // Remove character from the corresponding field
                // Implement logic to determine which field is currently active
                self.new_connection.label.pop(); // Example for label field
            }
            _ => {}
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
