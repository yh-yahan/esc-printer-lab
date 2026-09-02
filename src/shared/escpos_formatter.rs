use crate::parser::command::{Alignment, Command, CutMode, UnderlineMode};

pub struct EscPosFormatter;

impl EscPosFormatter {
    pub fn format_command(command: &Command) -> String {
        match command {
            Command::Initialize => "ESC @".into(),
            Command::LineFeed => "LF".into(),
            Command::Text(text) => format!("\"{text}\""),
            Command::Bold(on) => format!("ESC E {}", *on as u8),
            Command::Align(align) => {
                let n = match align {
                    Alignment::Left => 0,
                    Alignment::Center => 1,
                    Alignment::Right => 2,
                };
                format!("ESC a {n}")
            }
            Command::Underline(mode) => {
                let n = match mode {
                    UnderlineMode::Off => 0,
                    UnderlineMode::Thin => 1,
                    UnderlineMode::Thick => 2,
                };
                format!("ESC - {n}")
            }
            Command::Cut(mode) => {
                let n = match mode {
                    CutMode::Full => 0,
                    CutMode::Partial => 1,
                };
                format!("GS V {n}")
            }
            Command::CharSize(size) => {
                let n = ((size.width.saturating_sub(1) & 0x07) << 4)
                    | (size.height.saturating_sub(1) & 0x07);
                format!("GS ! {n:02X}")
            }

            Command::Unknown(bytes) => {
                let hex = bytes.iter()
                    .map(|byte| format!("{byte:02X}"))
                    .collect::<Vec<_>>()
                    .join(" ");

                format!("UNKNOWN [{hex}]")
            }
        }
    }
}
