use crate::parser::command::{Alignment, CharSize, Command, QrCommand, QrEcLevel, UnderlineMode};

use super::qr::encode_qr_raster;
use super::receipt::{Receipt, ReceiptEvent, ReceiptLine, ReceiptSegment};

const DEFAULT_LINE_SPACING: u8 = 30;
const DEFAULT_QR_MODULE_SIZE: u8 = 3;
const DEFAULT_QR_MODEL: u8 = 50;

pub struct ReceiptBuilder {
    current_bold: bool,
    current_alignment: Alignment,
    current_underline: UnderlineMode,
    current_char_size: CharSize,
    current_line_spacing: u8,
    qr_model: u8,
    qr_module_size: u8,
    qr_ec_level: QrEcLevel,
    qr_data: Option<Vec<u8>>,
    segments: Vec<ReceiptSegment>,
    receipt: Receipt,
}

impl ReceiptBuilder {
    pub fn new() -> Self {
        Self {
            current_bold: false,
            current_alignment: Alignment::Left,
            current_underline: UnderlineMode::Off,
            current_char_size: CharSize { width: 1, height: 1 },
            current_line_spacing: DEFAULT_LINE_SPACING,
            qr_model: DEFAULT_QR_MODEL,
            qr_module_size: DEFAULT_QR_MODULE_SIZE,
            qr_ec_level: QrEcLevel::L,
            qr_data: None,
            segments: Vec::new(),
            receipt: Receipt::new(),
        }
    }

    fn reset_formatting(&mut self) {
        self.current_alignment = Alignment::Left;
        self.current_bold = false;
        self.current_underline = UnderlineMode::Off;
        self.current_char_size = CharSize { width: 1, height: 1 };
        self.current_line_spacing = DEFAULT_LINE_SPACING;
        self.qr_model = DEFAULT_QR_MODEL;
        self.qr_module_size = DEFAULT_QR_MODULE_SIZE;
        self.qr_ec_level = QrEcLevel::L;
        self.qr_data = None;
    }

    pub fn process(&mut self, command: &Command) {
        match command {
            Command::Initialize => {
                self.reset_formatting();
                self.segments.clear();
                self.receipt = Receipt::new();
            }

            Command::Bold(bold) => {
                self.current_bold = *bold;
            }

            Command::Align(alignment) => {
                self.current_alignment = *alignment;
            }

            Command::Underline(underline_mode) => {
                self.current_underline = *underline_mode;
            }

            Command::CharSize(char_size) => {
                self.current_char_size = *char_size;
            }
            
            Command::SetDefaultLineSpacing => {
                self.current_line_spacing = DEFAULT_LINE_SPACING;
            }

            Command::SetLineSpacing(spacing) => {
                self.current_line_spacing = *spacing;
            }

            Command::PrintAndFeedLines(lines) => {
                self.flush_current_line();
                self.receipt.add_event(ReceiptEvent::FeedLines {
                    lines: *lines,
                    spacing: self.current_line_spacing,
                });
            }

            Command::PrintAndFeedDots(dots) => {
                self.flush_current_line();
                self.receipt.add_event(ReceiptEvent::FeedDots { dots: *dots });
            }

            Command::Text(text) => {
                self.segments.push(ReceiptSegment {
                    text: text.clone(),
                    bold: self.current_bold,
                    underline: self.current_underline,
                    char_size: self.current_char_size,
                });
            }

            Command::CarriageReturn => {
                self.segments.clear();
            }

            Command::LineFeed => {
                if self.segments.is_empty() {
                    self.receipt.add_event(ReceiptEvent::FeedLines {
                        lines: 1,
                        spacing: self.current_line_spacing,
                    });
                } else {
                    self.flush_current_line();
                }
            }

            Command::Cut(cut_mode) => {
                self.flush_current_line();
                self.receipt.add_event(ReceiptEvent::Cut(*cut_mode));
            }

            Command::RasterImage(image) => {
                self.flush_current_line();
                self.receipt.add_event(ReceiptEvent::RasterImage {
                    alignment: self.current_alignment,
                    image: image.clone(),
                });
            }

            Command::Qr(qr) => self.process_qr(qr),

            Command::Unknown(_) => {}
        }
    }

    fn process_qr(&mut self, qr: &QrCommand) {
        match qr {
            QrCommand::SetModel { model } => {
                self.qr_model = *model;
            }
            QrCommand::SetModuleSize { size } => {
                self.qr_module_size = *size;
            }
            QrCommand::SetErrorCorrection { level } => {
                self.qr_ec_level = *level;
            }
            QrCommand::Store { data } => {
                self.qr_data = Some(data.clone());
            }
            QrCommand::Print => {
                let _model = self.qr_model;
                let Some(data) = self.qr_data.as_deref() else {
                    return;
                };
                let Some(image) = encode_qr_raster(data, self.qr_module_size, self.qr_ec_level)
                else {
                    return;
                };
                self.flush_current_line();
                self.receipt.add_event(ReceiptEvent::RasterImage {
                    alignment: self.current_alignment,
                    image,
                });
            }
        }
    }

    pub fn process_commands(&mut self, commands: Vec<Command>) {
        for command in &commands {
            self.process(command);
        }
    }

    pub fn build(&self) -> Receipt {
        self.preview(None)
    }

    pub fn preview(&self, pending_text: Option<&str>) -> Receipt {
        let mut receipt = self.receipt.clone();
        let mut segments = self.segments.clone();

        if let Some(text) = pending_text.filter(|text| !text.is_empty()) {
            segments.push(ReceiptSegment {
                text: text.to_string(),
                bold: self.current_bold,
                underline: self.current_underline,
                char_size: self.current_char_size,
            });
        }

        if !segments.is_empty() {
            receipt.add_line(ReceiptLine {
                alignment: self.current_alignment,
                segments,
                spacing: self.current_line_spacing,
            });
        }

        receipt
    }

    pub fn start_new(&mut self) {
        self.segments.clear();
        self.receipt = Receipt::new();
    }

    fn flush_current_line(&mut self) {
        if self.segments.is_empty() {
            return;
        }

        self.receipt.add_line(ReceiptLine {
            alignment: self.current_alignment,
            segments: std::mem::take(&mut self.segments),
            spacing: self.current_line_spacing,
        });
    }
}
