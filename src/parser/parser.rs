use super::command::{
    Alignment, CharSize, Command, CutMode, QrCommand, QrEcLevel, RasterImage, RasterScale,
    UnderlineMode,
};
use super::state::ParserState;

use crate::printer::codepage::{
    decode_byte, is_supported_character_set, is_supported_code_page, DEFAULT_CHARACTER_SET,
    DEFAULT_CODE_PAGE,
};
use crate::shared::print_session::ParsedCommand;

const MAX_RASTER_BYTES: usize = 2 * 1024 * 1024;
const MAX_GS_PAREN_BYTES: usize = 2 * 1024 * 1024;

pub struct Parser {
    state: ParserState,
    text_buffer: String,
    offset: usize,
    command_start: Option<usize>,
    text_start: Option<usize>,
    code_page: u8,
    character_set: u8,
}

impl Parser {
    pub fn new() -> Self {
        Self {
            state: ParserState::Normal,
            text_buffer: String::new(),
            offset: 0,
            command_start: None,
            text_start: None,
            code_page: DEFAULT_CODE_PAGE,
            character_set: DEFAULT_CHARACTER_SET,
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
        self.flush_incomplete(&mut commands);

        commands
    }

    fn flush_incomplete(&mut self, commands: &mut Vec<ParsedCommand>) {
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
            ParserState::GsParen => vec![0x1D, 0x28],
            ParserState::GsParenHeader { ident, bytes } => {
                let mut raw = vec![0x1D, 0x28, *ident];
                raw.extend_from_slice(bytes);
                raw
            }
            ParserState::GsParenData {
                ident,
                p_l,
                p_h,
                data,
                ..
            } => {
                let mut raw = vec![0x1D, 0x28, *ident, *p_l, *p_h];
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
        let Some(ch) = decode_byte(self.code_page, self.character_set, byte) else {
            return;
        };

        if self.text_start.is_none() {
            self.text_start = Some(self.offset);
        }

        self.text_buffer.push(ch);
    }

    fn apply_code_page(&mut self, n: u8) -> bool {
        if is_supported_code_page(n) {
            self.code_page = n;
            true
        } else {
            false
        }
    }

    fn apply_character_set(&mut self, n: u8) -> bool {
        if is_supported_character_set(n) {
            self.character_set = n;
            true
        } else {
            false
        }
    }

    fn reset_encoding(&mut self) {
        self.code_page = DEFAULT_CODE_PAGE;
        self.character_set = DEFAULT_CHARACTER_SET;
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
            ParserState::EscCodePage => self.handle_esc_code_page(byte, commands),
            ParserState::EscCharacterSet => self.handle_esc_character_set(byte, commands),
            ParserState::Gs => self.handle_gs(byte, commands),
            ParserState::GsCut => self.handle_gs_cut(byte, commands),
            ParserState::GsCharSize => self.handle_gs_char_size(byte, commands),
            ParserState::GsRasterZero => self.handle_gs_raster_zero(byte, commands),
            ParserState::GsRasterHeader { .. } => self.handle_gs_raster_header(byte, commands),
            ParserState::GsRasterData { .. } => self.handle_gs_raster_data(byte, commands),
            ParserState::GsParen => self.handle_gs_paren(byte, commands),
            ParserState::GsParenHeader { .. } => self.handle_gs_paren_header(byte, commands),
            ParserState::GsParenData { .. } => self.handle_gs_paren_data(byte, commands),
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

                self.reset_encoding();
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

            // ESC t n
            0x74 => {
                self.state = ParserState::EscCodePage;
            }

            // ESC R n
            0x52 => {
                self.state = ParserState::EscCharacterSet;
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

    fn handle_esc_code_page(&mut self, byte: u8, commands: &mut Vec<ParsedCommand>) {
        self.flush_text(commands);

        let applied = self.apply_code_page(byte);
        self.push_command(
            Command::SelectCodePage { n: byte, applied },
            self.offset + 1,
            commands,
        );

        self.state = ParserState::Normal;
    }

    fn handle_esc_character_set(&mut self, byte: u8, commands: &mut Vec<ParsedCommand>) {
        self.flush_text(commands);

        let applied = self.apply_character_set(byte);
        self.push_command(
            Command::SelectCharacterSet { n: byte, applied },
            self.offset + 1,
            commands,
        );

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

            0x28 => {
                self.state = ParserState::GsParen;
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

    fn handle_gs_paren(&mut self, byte: u8, _commands: &mut Vec<ParsedCommand>) {
        self.state = ParserState::GsParenHeader {
            ident: byte,
            bytes: Vec::new(),
        };
    }

    fn handle_gs_paren_header(&mut self, byte: u8, commands: &mut Vec<ParsedCommand>) {
        let ParserState::GsParenHeader { ident, bytes } = &mut self.state else {
            return;
        };

        bytes.push(byte);
        if bytes.len() < 2 {
            return;
        }

        let ident = *ident;
        let p_l = bytes[0];
        let p_h = bytes[1];
        let k = p_l as usize + (p_h as usize) * 256;

        if k == 0 || k > MAX_GS_PAREN_BYTES {
            self.flush_text(commands);
            self.push_unknown(vec![0x1D, 0x28, ident, p_l, p_h], commands);
            self.state = ParserState::Normal;
            return;
        }

        self.state = ParserState::GsParenData {
            ident,
            p_l,
            p_h,
            data: Vec::with_capacity(k),
            remaining: k,
        };
    }

    fn handle_gs_paren_data(&mut self, byte: u8, commands: &mut Vec<ParsedCommand>) {
        let finished = {
            let ParserState::GsParenData {
                ident,
                p_l,
                p_h,
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

            Some((*ident, *p_l, *p_h, std::mem::take(data)))
        };

        let Some((ident, p_l, p_h, data)) = finished else {
            return;
        };

        self.flush_text(commands);

        if let Some(command) = parse_gs_paren(ident, &data) {
            self.push_command(command, self.offset + 1, commands);
        } else {
            let mut raw = vec![0x1D, 0x28, ident, p_l, p_h];
            raw.extend_from_slice(&data);
            self.push_unknown(raw, commands);
        }

        self.state = ParserState::Normal;
    }
}

fn parse_gs_paren(ident: u8, data: &[u8]) -> Option<Command> {
    if ident != b'k' || data.len() < 2 {
        return None;
    }

    let cn = data[0];
    let fn_code = data[1];
    let params = &data[2..];

    if cn != 49 {
        return None;
    }

    let command = match fn_code {
        65 if params.len() == 2 && matches!(params[0], 49 | 50) => {
            QrCommand::SetModel { model: params[0] }
        }
        67 if params.len() == 1 => QrCommand::SetModuleSize { size: params[0] },
        69 if params.len() == 1 => {
            QrCommand::SetErrorCorrection {
                level: QrEcLevel::from_n(params[0])?,
            }
        }
        80 if !params.is_empty() && params[0] == 48 => QrCommand::Store {
            data: params[1..].to_vec(),
        },
        81 if params.len() == 1 && params[0] == 48 => QrCommand::Print,
        _ => return None,
    };

    Some(Command::Qr(command))
}
