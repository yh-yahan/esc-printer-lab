use super::command::{Alignment, Command, CutMode, RasterScale, UnderlineMode};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandCategory {
    Text,
    Style,
    Layout,
    Control,
    Cut,
}

#[derive(Debug, Clone, Copy)]
pub struct CommandSpec {
    pub name: &'static str,
    pub mnemonic: &'static str,
    pub hex: &'static str,
    pub summary: &'static str,
    pub notes: &'static str,
    pub docs_url: Option<&'static str>,
    pub category: CommandCategory,
}

impl Command {
    pub fn spec(&self) -> CommandSpec {
        match self {
            Command::Initialize => CommandSpec {
                name: "Initialize printer",
                mnemonic: "ESC @",
                hex: "1B 40",
                summary: "Clears the print buffer and resets modes to power-on defaults.",
                notes: "Does not clear the receive buffer, NV memory, macros, or software settings.",
                docs_url: Some("https://download4.epson.biz/sec_pubs/pos/reference_en/escpos/esc_atsign.html"),
                category: CommandCategory::Control,
            },
            Command::Align(_) => CommandSpec {
                name: "Select justification",
                mnemonic: "ESC a n",
                hex: "1B 61 n",
                summary: "Sets left, center, or right alignment for following text.",
                notes: "Takes effect from the next printed line.",
                docs_url: Some("https://download4.epson.biz/sec_pubs/pos/reference_en/escpos/esc_la.html"),
                category: CommandCategory::Layout,
            },
            Command::Bold(_) => CommandSpec {
                name: "Turn emphasized mode on/off",
                mnemonic: "ESC E n",
                hex: "1B 45 n",
                summary: "Turns bold (emphasized) printing on or off.",
                notes: "Only the least significant bit of n is used.",
                docs_url: Some("https://download4.epson.biz/sec_pubs/pos/reference_en/escpos/esc_ce.html"),
                category: CommandCategory::Style,
            },
            Command::Underline(_) => CommandSpec {
                name: "Turn underline mode on/off",
                mnemonic: "ESC - n",
                hex: "1B 2D n",
                summary: "Sets underline off, thin, or thick.",
                notes: "Accepts both binary values (0, 1, 2) and ASCII digits ('0', '1', '2').",
                docs_url: Some("https://download4.epson.biz/sec_pubs/pos/reference_en/escpos/esc_minus.html"),
                category: CommandCategory::Style,
            },
            Command::Cut(_) => CommandSpec {
                name: "Select cut mode and cut paper",
                mnemonic: "GS V n",
                hex: "1D 56 n",
                summary: "Cuts the paper using a full or partial cut.",
                notes: "Accepts both binary values (0, 1) and ASCII digits ('0', '1').",
                docs_url: Some("https://download4.epson.biz/sec_pubs/pos/reference_en/escpos/gs_cv.html"),
                category: CommandCategory::Cut,
            },
            Command::RasterImage(_) => CommandSpec {
                name: "Print raster bit image",
                mnemonic: "GS v 0",
                hex: "1D 76 30 m xL xH yL yH d1...dk",
                summary: "Prints a raster bit image. x is bytes per row; y is height in dots; payload is x × y bytes.",
                notes: "Accepts 0x00 or 0x30 as the command id after v. Bit 7 of each data byte is the leftmost pixel. m selects normal, double-width, double-height, or quadruple size.",
                docs_url: Some("https://download4.epson.biz/sec_pubs/pos/reference_en/escpos/gs_lv_0.html"),
                category: CommandCategory::Layout,
            },
            Command::CharSize(_) => CommandSpec {
                name: "Select character size",
                mnemonic: "GS ! n",
                hex: "1D 21 n",
                summary: "Sets character width and height magnification from 1x to 8x.",
                notes: "Width is bits 4-6 of n. Height is bits 0-2 of n. Each field is encoded as size - 1.",
                docs_url: Some("https://download4.epson.biz/sec_pubs/pos/reference_en/escpos/gs_exclamation.html"),
                category: CommandCategory::Style,
            },
            Command::LineFeed => CommandSpec {
                name: "Print and line feed",
                mnemonic: "LF",
                hex: "0A",
                summary: "Prints the buffer and advances the paper by one line.",
                notes: "",
                docs_url: Some("https://download4.epson.biz/sec_pubs/pos/reference_en/escpos/lf.html"),
                category: CommandCategory::Control,
            },
            Command::SetDefaultLineSpacing => CommandSpec {
                name: "Set default line spacing",
                mnemonic: "ESC 2",
                hex: "1B 32",
                summary: "Sets the line spacing to the printer's default value.",
                notes: "The default line spacing depends on the printer model.",
                docs_url: Some("https://download4.epson.biz/sec_pubs/pos/reference_en/escpos/esc_2.html"),
                category: CommandCategory::Layout,
            },
            Command::SetLineSpacing(_) => CommandSpec {
                name: "Set line spacing",
                mnemonic: "ESC 3 n",
                hex: "1B 33 n",
                summary: "Sets the line spacing to n dots.",
                notes: "The specified spacing is used for subsequent line feeds and line-based paper feeding.",
                docs_url: Some("https://download4.epson.biz/sec_pubs/pos/reference_en/escpos/esc_3.html"),
                category: CommandCategory::Layout,
            },
            Command::CarriageReturn => CommandSpec {
                name: "Carriage return",
                mnemonic: "CR",
                hex: "0D",
                summary: "Returns the print position to the beginning of the current line.",
                notes: "Does not advance the paper or print a new line.",
                docs_url: Some("https://download4.epson.biz/sec_pubs/pos/reference_en/escpos/cr.html"),
                category: CommandCategory::Control,
            },
            Command::PrintAndFeedLines(_) => CommandSpec {
                name: "Print and feed n lines",
                mnemonic: "ESC d n",
                hex: "1B 64 n",
                summary: "Prints the data in the print buffer and feeds the paper by n lines.",
                notes: "n specifies the number of lines to feed.",
                docs_url: Some("https://download4.epson.biz/sec_pubs/pos/reference_en/escpos/esc_ld.html"),
                category: CommandCategory::Control,
            },
            Command::PrintAndFeedDots(_) => CommandSpec {
                name: "Print and feed n dots",
                mnemonic: "ESC J n",
                hex: "1B 4A n",
                summary: "Prints the data in the print buffer and feeds the paper by n dots.",
                notes: "n specifies the number of dots to feed. The valid range depends on the printer model.",
                docs_url: Some("https://download4.epson.biz/sec_pubs/pos/reference_en/escpos/esc_cj.html"),
                category: CommandCategory::Control,
            },
            Command::Text(_) => CommandSpec {
                name: "Print data",
                mnemonic: "text",
                hex: "printable bytes",
                summary: "Printable characters sent to the printer. Not a control command.",
                notes: "",
                docs_url: None,
                category: CommandCategory::Text,
            },
            Command::Unknown(_) => CommandSpec {
                name: "Unknown command",
                mnemonic: "unknown",
                hex: "raw bytes",
                summary: "Command bytes that are not currently recognized by the parser.",
                notes: "May be a valid ESC/POS command that is not yet supported by the emulator, or invalid command data.",
                docs_url: None,
                category: CommandCategory::Control,
            },
        }
    }

    pub fn param_lines(&self) -> Vec<String> {
        match self {
            Command::Align(align) => vec![match align {
                Alignment::Left => "n = 0  left".into(),
                Alignment::Center => "n = 1  center".into(),
                Alignment::Right => "n = 2  right".into(),
            }],
            Command::Bold(on) => {
                let n = *on as u8;
                let label = if *on { "on" } else { "off" };
                vec![format!("n = {n}  {label}")]
            }
            Command::Underline(mode) => vec![match mode {
                UnderlineMode::Off => "n = 0  off".into(),
                UnderlineMode::Thin => "n = 1  thin".into(),
                UnderlineMode::Thick => "n = 2  thick".into(),
            }],
            Command::Cut(mode) => vec![match mode {
                CutMode::Full => "n = 0  full cut".into(),
                CutMode::Partial => "n = 1  partial cut".into(),
            }],
            Command::CharSize(size) => vec![
                format!("width  = {}x", size.width),
                format!("height = {}x", size.height),
            ],
            Command::RasterImage(image) => {
                let mode = match image.scale {
                    RasterScale::Normal => "normal",
                    RasterScale::DoubleWidth => "double-width",
                    RasterScale::DoubleHeight => "double-height",
                    RasterScale::Quadruple => "quadruple",
                };
                vec![
                    format!("m = {}  {mode}", image.scale.m()),
                    format!("x = {}  bytes/row ({} dots)", image.width_bytes, image.width_dots()),
                    format!("y = {}  dots", image.height),
                    format!("k = {}  data byte(s)", image.data.len()),
                ]
            }
            Command::Text(text) => vec![format!("{} byte(s)", text.len())],
            Command::Initialize | Command::LineFeed | Command::CarriageReturn => vec![],
            Command::SetDefaultLineSpacing => vec!["uses printer default line spacing".into()],
            Command::SetLineSpacing(spacing) => vec![format!("n = {spacing}  spacing in dots")],
            Command::PrintAndFeedLines(lines) => vec![format!("n = {lines}  feed {lines} line(s)")],
            Command::PrintAndFeedDots(dots) => vec![format!("n = {dots}  feed {dots} dot(s)")],
            Command::Unknown(bytes) => vec![format!("{} byte(s)", bytes.len())],
        }
    }
}
