use crate::parser::command::{Alignment, UnderlineMode};

#[derive(Debug, Clone, Default)]
pub struct Receipt {
    pub lines: Vec<ReceiptLine>,
}

#[derive(Debug, Clone)]
pub struct ReceiptLine {
    pub text: String,
    pub alignment: Alignment,
    pub bold: bool,
    pub underline: UnderlineMode
}

impl Receipt {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_line(&mut self, line: ReceiptLine) {
        self.lines.push(line);
    }
}
