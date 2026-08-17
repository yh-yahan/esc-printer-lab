use super::command::{Alignment, UnderlineMode, CutMode, Command};
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
            ParserState::EscEmphasis => self.handle_esc_emphasis(byte, commands),
            ParserState::EscUnderline => self.handle_esc_underline(byte, commands),
            ParserState::Gs => self.handle_gs(byte),
            ParserState::GsCut => self.handle_gs_cut(byte, commands),
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

            0x1D => {
                self.flush_text(commands);
                self.state = ParserState::Gs;
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

            0x45 => {
                self.state = ParserState::EscEmphasis;
            }

            0x61 => {
                self.state = ParserState::EscAlignment;
            }

            0x2D => {
                self.state = ParserState::EscUnderline;
            }

            _ => {
                self.state = ParserState::Normal;
            }
        }
    }

    fn handle_esc_alignment(&mut self, byte: u8, commands: &mut Vec<Command>) {
        self.flush_text(commands);

        let align = match byte {
            0x00 => Some(Alignment::Left),
            0x01 => Some(Alignment::Center),
            0x02 => Some(Alignment::Right),
            _ => None,
        };

        if let Some(align) = align {
            commands.push(Command::Align(align));
        }

        self.state = ParserState::Normal;
    }

    fn handle_esc_emphasis(&mut self, byte: u8, commands: &mut Vec<Command>) {
        self.flush_text(commands);

        let enabled = (byte & 0x01) != 0;

        commands.push(Command::Bold(enabled));

        self.state = ParserState::Normal;
    }

    fn handle_esc_underline(&mut self, byte: u8, commands: &mut Vec<Command>) {
        self.flush_text(commands);

        let underline = match byte {
            0x00 | 0x30 => Some(UnderlineMode::Off),
            0x01 | 0x31 => Some(UnderlineMode::Thin),
            0x02 | 0x32 => Some(UnderlineMode::Thick),
            _ => None,
        };

        if let Some(underline) = underline {
            commands.push(Command::Underline(underline));
        }

        self.state = ParserState::Normal;
    }

    fn handle_gs(&mut self, byte: u8) {
        match byte {
            0x56 => {
                self.state = ParserState::GsCut;
            }

            _ => {
                self.state = ParserState::Normal;
            }
        }
    }

    fn handle_gs_cut(&mut self, byte: u8, commands: &mut Vec<Command>) {
        self.flush_text(commands);

        let cut = match byte {
            0x00 | 0x30 => Some(CutMode::Full),
            0x01 | 0x31 => Some(CutMode::Partial),
            _ => None,
        };

        if let Some(cut) = cut {
            commands.push(Command::Cut(cut));
        }

        self.state = ParserState::Normal;
    }
}
