use crate::parser::command::{Alignment, Command, CutMode, QrCommand, UnderlineMode};

pub struct EscPosFormatter;

impl EscPosFormatter {
    pub fn format_command(command: &Command) -> String {
        match command {
            Command::Initialize => "ESC @".into(),
            Command::LineFeed => "LF".into(),
            Command::CarriageReturn => "CR".into(),
            Command::SetDefaultLineSpacing => "ESC 2".into(),
            Command::SetLineSpacing(spacing) => {
                format!("ESC 3 {spacing}")
            }
            Command::Text(text) => format!("\"{text}\""),
            Command::SelectCodePage { n, applied } => {
                if *applied {
                    format!("ESC t {n}")
                } else {
                    format!("ESC t {n} (ignored)")
                }
            }
            Command::SelectCharacterSet { n, applied } => {
                if *applied {
                    format!("ESC R {n}")
                } else {
                    format!("ESC R {n} (ignored)")
                }
            }
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
            Command::RasterImage(image) => {
                format!(
                    "GS v 0 {} {}x{} ({} bytes)",
                    image.scale.m(),
                    image.width_dots(),
                    image.height,
                    image.data.len()
                )
            }
            Command::Qr(qr) => match qr {
                QrCommand::SetModel { model } => {
                    format!("GS ( k fn65 model {}", if *model == 49 { 1 } else { 2 })
                }
                QrCommand::SetModuleSize { size } => {
                    format!("GS ( k fn67 size {size}")
                }
                QrCommand::SetErrorCorrection { level } => {
                    format!("GS ( k fn69 EC {}", level.name())
                }
                QrCommand::Store { data } => {
                    format!("GS ( k fn80 store {} byte(s)", data.len())
                }
                QrCommand::Print => "GS ( k fn81 print".into(),
            },
            Command::CharSize(size) => {
                let n = ((size.width.saturating_sub(1) & 0x07) << 4)
                    | (size.height.saturating_sub(1) & 0x07);
                format!("GS ! {n:02X}")
            }
            Command::PrintAndFeedLines(lines) => {
                format!("ESC d {lines}")
            }
            Command::PrintAndFeedDots(dots) => {
                format!("ESC J {dots}")
            }
            Command::Unknown(bytes) => {
                let hex = bytes
                    .iter()
                    .map(|byte| format!("{byte:02X}"))
                    .collect::<Vec<_>>()
                    .join(" ");

                format!("UNKNOWN [{hex}]")
            }
        }
    }
}
