use super::command::{Alignment, CharSize, Command, CutMode, UnderlineMode};
use super::state::ParserState;

use crate::shared::print_session::ParsedCommand;

pub struct Parser {
    state: ParserState,
    text_buffer: String,
    offset: usize,
    command_start: Option<usize>,
    text_start: Option<usize>,
}

impl Parser {
    pub fn new() -> Self {
        Self {
            state: ParserState::Normal,
            text_buffer: String::new(),
            offset: 0,
            command_start: None,
            text_start: None,
        }
    }

    pub fn feed(&mut self, bytes: &[u8]) -> Vec<ParsedCommand> {
        let mut commands = Vec::new();

        for &byte in bytes {
            self.process_byte(byte, &mut commands);
            self.offset += 1;
        }

        commands
    }

    pub fn pending_text(&self) -> Option<&str> {
        if self.text_buffer.is_empty() {
            None
        } else {
            Some(self.text_buffer.as_str())
        }
    }

    pub fn finish(&mut self) -> Vec<ParsedCommand> {
        let mut commands = Vec::new();

        self.flush_text(&mut commands);

        commands
    }

    fn flush_text(&mut self, commands: &mut Vec<ParsedCommand>) {
        if !self.text_buffer.is_empty() {
            let start = self.text_start.unwrap_or(self.offset);

            commands.push(ParsedCommand {
                command: Command::Text(self.text_buffer.clone()),
                span: start..self.offset,
            });

            self.text_buffer.clear();
            self.text_start = None;
        }
    }

    fn push_text_byte(&mut self, byte: u8) {
        if byte == 0x00 {
            return;
        }

        if self.text_start.is_none() {
            self.text_start = Some(self.offset);
        }

        self.text_buffer.push(byte as char);
    }

    fn push_command(&mut self, command: Command, end: usize, commands: &mut Vec<ParsedCommand>) {
        let start = self.command_start.unwrap_or(self.offset);

        commands.push(ParsedCommand {
            command,
            span: start..end,
        });

        self.command_start = None;
    }

    fn push_unknown(&mut self, bytes: Vec<u8>, commands: &mut Vec<ParsedCommand>) {
        let start = self.command_start.unwrap_or(self.offset);

        commands.push(ParsedCommand {
            command: Command::Unknown(bytes),
            span: start..self.offset + 1,
        });

        self.command_start = None;
    }

    fn process_byte(&mut self, byte: u8, commands: &mut Vec<ParsedCommand>) {
        match self.state {
            ParserState::Normal => self.handle_normal(byte, commands),
            ParserState::Esc => self.handle_esc(byte, commands),
            ParserState::EscAlignment => self.handle_esc_alignment(byte, commands),
            ParserState::EscEmphasis => self.handle_esc_emphasis(byte, commands),
            ParserState::EscUnderline => self.handle_esc_underline(byte, commands),
            ParserState::Gs => self.handle_gs(byte, commands),
            ParserState::GsCut => self.handle_gs_cut(byte, commands),
            ParserState::GsCharSize => self.handle_gs_char_size(byte, commands),
        }
    }

    fn handle_normal(&mut self, byte: u8, commands: &mut Vec<ParsedCommand>) {
        match byte {
            0x1B => {
                self.flush_text(commands);

                self.command_start = Some(self.offset);
                self.state = ParserState::Esc;
            }

            0x0A => {
                self.flush_text(commands);

                self.push_command(
                    Command::LineFeed,
                    self.offset + 1,
                    commands,
                );
            }

            0x1D => {
                self.flush_text(commands);

                self.command_start = Some(self.offset);
                self.state = ParserState::Gs;
            }

            _ => {
                self.push_text_byte(byte);
            }
        }
    }

    fn handle_esc(&mut self, byte: u8, commands: &mut Vec<ParsedCommand>) {
        match byte {
            0x40 => {
                self.flush_text(commands);

                self.push_command(
                    Command::Initialize,
                    self.offset + 1,
                    commands,
                );

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
                self.flush_text(commands);

                self.push_unknown(
                    vec![0x1B, byte],
                    commands,
                );

                self.state = ParserState::Normal;
            }
        }
    }

    fn handle_esc_alignment(&mut self, byte: u8, commands: &mut Vec<ParsedCommand>) {
        self.flush_text(commands);

        let align = match byte {
            0x00 => Some(Alignment::Left),
            0x01 => Some(Alignment::Center),
            0x02 => Some(Alignment::Right),
            _ => None,
        };

        if let Some(align) = align {
            self.push_command(
                Command::Align(align),
                self.offset + 1,
                commands,
            );
        } else {
            self.push_unknown(
                vec![0x1B, 0x61, byte],
                commands,
            );
        }

        self.state = ParserState::Normal;
    }

    fn handle_esc_emphasis(&mut self, byte: u8, commands: &mut Vec<ParsedCommand>) {
        self.flush_text(commands);

        let enabled = (byte & 0x01) != 0;

        self.push_command(
            Command::Bold(enabled),
            self.offset + 1,
            commands,
        );

        self.state = ParserState::Normal;
    }

    fn handle_esc_underline(&mut self, byte: u8, commands: &mut Vec<ParsedCommand>) {
        self.flush_text(commands);

        let underline = match byte {
            0x00 | 0x30 => Some(UnderlineMode::Off),
            0x01 | 0x31 => Some(UnderlineMode::Thin),
            0x02 | 0x32 => Some(UnderlineMode::Thick),
            _ => None,
        };

        if let Some(underline) = underline {
            self.push_command(
                Command::Underline(underline),
                self.offset + 1,
                commands,
            );
        } else {
            self.push_unknown(
                vec![0x1B, 0x2D, byte],
                commands,
            );
        }

        self.state = ParserState::Normal;
    }

    fn handle_gs(&mut self, byte: u8, commands: &mut Vec<ParsedCommand>) {
        match byte {
            0x21 => {
                self.state = ParserState::GsCharSize;
            }

            0x56 => {
                self.state = ParserState::GsCut;
            }

            _ => {
                self.flush_text(commands);

                self.push_unknown(
                    vec![0x1D, byte],
                    commands,
                );

                self.state = ParserState::Normal;
            }
        }
    }

    fn handle_gs_cut(&mut self, byte: u8, commands: &mut Vec<ParsedCommand>) {
        self.flush_text(commands);

        let cut = match byte {
            0x00 | 0x30 => Some(CutMode::Full),
            0x01 | 0x31 => Some(CutMode::Partial),
            _ => None,
        };

        if let Some(cut) = cut {
            self.push_command(
                Command::Cut(cut),
                self.offset + 1,
                commands,
            );
        } else {
            self.push_unknown(
                vec![0x1D, 0x56, byte],
                commands,
            );
        }

        self.state = ParserState::Normal;
    }

    fn handle_gs_char_size(&mut self, byte: u8, commands: &mut Vec<ParsedCommand>) {
        let width = ((byte >> 4) & 0x07) + 1;
        let height = (byte & 0x07) + 1;

        self.push_command(
            Command::CharSize(CharSize { width, height }),
            self.offset + 1,
            commands,
        );

        self.state = ParserState::Normal;
    }
}
