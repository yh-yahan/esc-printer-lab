use super::command::{
    Alignment, CharSize, Command, CutMode, RasterImage, RasterScale, UnderlineMode,
};
use super::state::ParserState;

use crate::shared::print_session::ParsedCommand;

const MAX_RASTER_BYTES: usize = 2 * 1024 * 1024;

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
        self.flush_incomplete_raster(&mut commands);

        commands
    }

    fn flush_incomplete_raster(&mut self, commands: &mut Vec<ParsedCommand>) {
        let start = self.command_start.unwrap_or(self.offset);
        let bytes = match &self.state {
            ParserState::GsRasterZero => vec![0x1D, 0x76],
            ParserState::GsRasterHeader { bytes } => {
                let mut raw = vec![0x1D, 0x76, 0x30];
                raw.extend_from_slice(bytes);
                raw
            }
            ParserState::GsRasterData {
                scale,
                width_bytes,
                height,
                data,
                ..
            } => {
                let mut raw = vec![
                    0x1D,
                    0x76,
                    0x30,
                    scale.m(),
                    (*width_bytes & 0xFF) as u8,
                    (*width_bytes >> 8) as u8,
                    (*height & 0xFF) as u8,
                    (*height >> 8) as u8,
                ];
                raw.extend_from_slice(data);
                raw
            }
            _ => return,
        };

        commands.push(ParsedCommand {
            command: Command::Unknown(bytes),
            span: start..self.offset,
        });

        self.command_start = None;
        self.state = ParserState::Normal;
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
            ParserState::EscLineSpacing => self.handle_esc_line_spacing(byte, commands),
            ParserState::EscPrintAndFeedLines => self.handle_esc_print_and_feed_lines(byte, commands),
            ParserState::EscPrintAndFeedDots => self.handle_esc_print_and_feed_dots(byte, commands),
            ParserState::EscAlignment => self.handle_esc_alignment(byte, commands),
            ParserState::EscEmphasis => self.handle_esc_emphasis(byte, commands),
            ParserState::EscUnderline => self.handle_esc_underline(byte, commands),
            ParserState::Gs => self.handle_gs(byte, commands),
            ParserState::GsCut => self.handle_gs_cut(byte, commands),
            ParserState::GsCharSize => self.handle_gs_char_size(byte, commands),
            ParserState::GsRasterZero => self.handle_gs_raster_zero(byte, commands),
            ParserState::GsRasterHeader { .. } => self.handle_gs_raster_header(byte, commands),
            ParserState::GsRasterData { .. } => self.handle_gs_raster_data(byte, commands),
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

            0x0D => {
                self.flush_text(commands);

                self.push_command(
                    Command::CarriageReturn,
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
            // ESC @
            0x40 => {
                self.flush_text(commands);

                self.push_command(
                    Command::Initialize,
                    self.offset + 1,
                    commands,
                );

                self.state = ParserState::Normal;
            }

            // ESC E n
            0x45 => {
                self.state = ParserState::EscEmphasis;
            }

            // ESC a n
            0x61 => {
                self.state = ParserState::EscAlignment;
            }

            // ESC 2
            0x32 => {
                self.flush_text(commands);

                self.push_command(
                    Command::SetDefaultLineSpacing,
                    self.offset + 1,
                    commands,
                );

                self.state = ParserState::Normal;
            }

            // ESC 3 n
            0x33 => {
                self.state = ParserState::EscLineSpacing;
            }

            0x4A => {
                self.state = ParserState::EscPrintAndFeedDots;
            }

            0x64 => {
                self.state = ParserState::EscPrintAndFeedLines;
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

    fn handle_esc_line_spacing(&mut self, byte: u8, commands: &mut Vec<ParsedCommand>) {
        self.flush_text(commands);

        self.push_command(
            Command::SetLineSpacing(byte),
            self.offset + 1,
            commands,
        );

        self.state = ParserState::Normal;
    }

    fn handle_esc_print_and_feed_lines(&mut self, byte: u8, commands: &mut Vec<ParsedCommand>) {
        self.flush_text(commands);

        self.push_command(
            Command::PrintAndFeedLines(byte),
            self.offset + 1,
            commands,
        );

        self.state = ParserState::Normal;
    }

    fn handle_esc_print_and_feed_dots(&mut self, byte: u8, commands: &mut Vec<ParsedCommand>) {
        self.flush_text(commands);
        self.push_command(
            Command::PrintAndFeedDots(byte),
            self.offset + 1,
            commands,
        );
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

            0x76 => {
                self.state = ParserState::GsRasterZero;
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

    fn handle_gs_raster_zero(&mut self, byte: u8, commands: &mut Vec<ParsedCommand>) {
        match byte {
            0x00 | 0x30 => {
                self.state = ParserState::GsRasterHeader { bytes: Vec::new() };
            }
            _ => {
                self.flush_text(commands);
                self.push_unknown(vec![0x1D, 0x76, byte], commands);
                self.state = ParserState::Normal;
            }
        }
    }

    fn handle_gs_raster_header(&mut self, byte: u8, commands: &mut Vec<ParsedCommand>) {
        let ParserState::GsRasterHeader { bytes } = &mut self.state else {
            return;
        };

        bytes.push(byte);

        if bytes.len() < 5 {
            return;
        }

        let header = bytes.clone();
        let m = header[0];
        let width_bytes = u16::from_le_bytes([header[1], header[2]]);
        let height = u16::from_le_bytes([header[3], header[4]]);
        let k = width_bytes as usize * height as usize;

        let scale = RasterScale::from_m(m).unwrap_or(RasterScale::Normal);

        if k == 0 || k > MAX_RASTER_BYTES {
            self.flush_text(commands);
            let mut raw = vec![0x1D, 0x76, 0x30];
            raw.extend_from_slice(&header);
            self.push_unknown(raw, commands);
            self.state = ParserState::Normal;
            return;
        }

        self.state = ParserState::GsRasterData {
            scale,
            width_bytes,
            height,
            data: Vec::with_capacity(k),
            remaining: k,
        };
    }

    fn handle_gs_raster_data(&mut self, byte: u8, commands: &mut Vec<ParsedCommand>) {
        let finished = {
            let ParserState::GsRasterData {
                scale,
                width_bytes,
                height,
                data,
                remaining,
            } = &mut self.state
            else {
                return;
            };

            data.push(byte);
            *remaining = remaining.saturating_sub(1);

            if *remaining > 0 {
                return;
            }

            Some((
                *scale,
                *width_bytes,
                *height,
                std::mem::take(data),
            ))
        };

        let Some((scale, width_bytes, height, data)) = finished else {
            return;
        };

        self.flush_text(commands);

        let image = RasterImage {
            scale,
            width_bytes,
            height,
            data,
        };

        self.push_command(
            Command::RasterImage(image),
            self.offset + 1,
            commands,
        );

        self.state = ParserState::Normal;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn feed(bytes: &[u8]) -> Vec<Command> {
        let mut parser = Parser::new();
        let mut commands: Vec<_> = parser
            .feed(bytes)
            .into_iter()
            .map(|parsed| parsed.command)
            .collect();
        commands.extend(parser.finish().into_iter().map(|parsed| parsed.command));
        commands
    }

    #[test]
    fn parses_gs_v_0_raster() {
        let mut bytes = vec![0x1D, 0x76, 0x30, 0x00, 0x02, 0x00, 0x02, 0x00];
        bytes.extend_from_slice(&[0xC0, 0x00, 0x00, 0x00]);

        let commands = feed(&bytes);
        match &commands[..] {
            [Command::RasterImage(image)] => {
                assert_eq!(image.scale, RasterScale::Normal);
                assert_eq!(image.width_bytes, 2);
                assert_eq!(image.height, 2);
                assert_eq!(image.data, vec![0xC0, 0x00, 0x00, 0x00]);
            }
            other => panic!("unexpected commands: {other:?}"),
        }
    }

    #[test]
    fn accepts_binary_zero_command_id() {
        let bytes = [0x1D, 0x76, 0x00, 0x00, 0x01, 0x00, 0x01, 0x00, 0xFF];
        let commands = feed(&bytes);
        assert!(matches!(commands.as_slice(), [Command::RasterImage(_)]));
    }

    #[test]
    fn splits_raster_across_feeds() {
        let mut parser = Parser::new();
        let first = parser.feed(&[0x1D, 0x76, 0x30, 0x00, 0x01, 0x00, 0x02, 0x00, 0xAA]);
        assert!(first.is_empty());

        let second = parser.feed(&[0x55]);
        match &second[..] {
            [parsed] => match &parsed.command {
                Command::RasterImage(image) => {
                    assert_eq!(image.data, vec![0xAA, 0x55]);
                    assert_eq!(image.height, 2);
                }
                other => panic!("unexpected command: {other:?}"),
            },
            other => panic!("unexpected commands: {other:?}"),
        }
    }

    #[test]
    fn incomplete_raster_becomes_unknown() {
        let commands = feed(&[0x1D, 0x76, 0x30, 0x00, 0x02, 0x00, 0x02, 0x00, 0xC0]);
        assert!(matches!(commands.as_slice(), [Command::Unknown(_)]));
    }
}
