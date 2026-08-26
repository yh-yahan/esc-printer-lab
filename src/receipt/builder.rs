use crate::parser::command::{Alignment, Command, CharSize, UnderlineMode};

use super::receipt::{
    Receipt,
    ReceiptEvent,
    ReceiptLine,
    ReceiptSegment,
};

pub struct ReceiptBuilder {
    current_bold: bool,
    current_alignment: Alignment,
    current_underline: UnderlineMode,
    current_char_size: CharSize,
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
            segments: Vec::new(),
            receipt: Receipt::new(),
        }
    }

    pub fn process(&mut self, command: &Command) {
        match command {
            Command::Initialize => {
                self.current_alignment = Alignment::Left;
                self.segments.clear();
                self.current_bold = false;
                self.current_underline = UnderlineMode::Off;
                self.current_char_size = CharSize { width: 1, height: 1 };
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

            Command::Text(text) => {
                self.segments.push(ReceiptSegment {
                    text: text.clone(),
                    bold: self.current_bold,
                    underline: self.current_underline,
                    char_size: self.current_char_size,
                });
            }

            Command::LineFeed => {
                self.flush_current_line();
            }

            Command::Cut(cut_mode) => {
                self.flush_current_line();

                self.receipt.add_event(
                    ReceiptEvent::Cut(*cut_mode)
                );
            }
        }
    }

    pub fn process_commands(&mut self, commands: Vec<Command>) {
        for command in &commands {
            self.process(command);
        }
    }

    pub fn build(mut self) -> Receipt {
        self.flush_current_line();
        self.receipt
    }

    fn flush_current_line(&mut self) {
        if self.segments.is_empty() {
            return;
        }

        self.receipt.add_line(ReceiptLine {
            alignment: self.current_alignment,
            segments: std::mem::take(&mut self.segments),
        });
    }
}

