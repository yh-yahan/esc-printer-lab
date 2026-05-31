use super::command::{Alignment, Command};
use super::state::ParserState;

pub struct Parser {
    state: ParserState,
    text_buffer: String,
}

impl Parser {
    pub fn new() -> Self {
        Self {
            state: ParserState::Normal,
            text_buffer: String::new(),
        }
    }

    pub fn feed(&mut self, bytes: &[u8]) -> Vec<Command> {
        let mut commands = Vec::new();

        for &byte in bytes {
            self.process_byte(byte, &mut commands);
        }

        self.flush_text(&mut commands);

        commands
    }

    fn flush_text(&mut self, commands: &mut Vec<Command>) {
        if !self.text_buffer.is_empty() {
            commands.push(Command::Text(self.text_buffer.clone()));
            self.text_buffer.clear();
        }
    }

    fn push_text_byte(&mut self, byte: u8) {
        if byte == 0x00 {
            return;
        }

        self.text_buffer.push(byte as char);
    }

    fn process_byte(&mut self, byte: u8, commands: &mut Vec<Command>) {
        match self.state {
            ParserState::Normal => self.handle_normal(byte, commands),
            ParserState::Esc => self.handle_esc(byte, commands),
            ParserState::EscAlignment => self.handle_esc_alignment(byte, commands),
        }
    }

    fn handle_normal(&mut self, byte: u8, commands: &mut Vec<Command>) {
        match byte {
            0x1B => {
                self.flush_text(commands);
                self.state = ParserState::Esc;
            }

            0x0A => {
                self.flush_text(commands);
                commands.push(Command::LineFeed);
            }

            _ => {
                self.push_text_byte(byte);
            }
        }
    }

    fn handle_esc(&mut self, byte: u8, commands: &mut Vec<Command>) {
        match byte {
            0x40 => {
                self.flush_text(commands);
                commands.push(Command::Initialize);
                self.state = ParserState::Normal;
            }

            0x61 => {
                self.state = ParserState::EscAlignment;
            }

            _ => {
                self.state = ParserState::Normal;
            }
        }
    }

    fn handle_esc_alignment(&mut self, byte: u8, commands: &mut Vec<Command>) {
        self.flush_text(commands);

        let align = match byte {
            0x00 => Alignment::Left,
            0x01 => Alignment::Center,
            0x02 => Alignment::Right,
            _ => Alignment::Left,
        };

        commands.push(Command::Align(align));
        self.state = ParserState::Normal;
    }
}
