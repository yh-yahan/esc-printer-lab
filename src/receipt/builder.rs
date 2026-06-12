use crate::parser::command::{Alignment, Command};

use super::receipt::{Receipt, ReceiptLine};

pub struct ReceiptBuilder {
    current_alignment: Alignment,
    current_line: String,
    receipt: Receipt,
}

impl ReceiptBuilder {
    pub fn new() -> Self {
        Self {
            current_alignment: Alignment::Left,
            current_line: String::new(),
            receipt: Receipt::new(),
        }
    }

    pub fn process(&mut self, command: &Command) {
        match command {
            Command::Initialize => {
                self.current_alignment = Alignment::Left;
                self.current_line.clear();
            }

            Command::Align(alignment) => {
                self.current_alignment = *alignment;
            }

            Command::Text(text) => {
                self.current_line.push_str(&text);
            }

            Command::LineFeed => {
                self.flush_current_line();
            }

            Command::Bold(_) => {}
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
        if self.current_line.is_empty() {
            return;
        }

        self.receipt.lines.push(ReceiptLine {
            text: self.current_line.clone(),
            alignment: self.current_alignment,
        });

        self.current_line.clear();
    }
}
