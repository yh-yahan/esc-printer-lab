use crate::parser::command::{Alignment, Command};

use super::receipt::{Receipt, ReceiptLine};

pub struct ReceiptBuilder {
    current_bold: bool,
    current_alignment: Alignment,
    segments: Vec<(String, bool)>,
    receipt: Receipt,
}

impl ReceiptBuilder {
    pub fn new() -> Self {
        Self {
            current_bold: false,
            current_alignment: Alignment::Left,
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
            }

            Command::Bold(bold) => {
                self.current_bold = *bold;
            }

            Command::Align(alignment) => {
                self.current_alignment = *alignment;
            }

            Command::Text(text) => {
                self.segments.push((text.clone(), self.current_bold));
            }

            Command::LineFeed => {
                self.flush_current_line();
            }

            Command::Cut => {}
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

        let text = self
            .segments
            .iter()
            .map(|(t, _)| t.as_str())
            .collect::<String>();

        let bold = self.segments.iter().any(|(_, b)| *b);

        self.receipt.lines.push(ReceiptLine {
            text,
            alignment: self.current_alignment,
            bold,
        });

        self.segments.clear();
    }
}
