use crate::parser::command::Alignment;

#[derive(Debug, Clone, Default)]
pub struct Receipt {
    pub lines: Vec<ReceiptLine>,
}

#[derive(Debug, Clone)]
pub struct ReceiptLine {
    pub text: String,
    pub alignment: Alignment,
}

impl Receipt {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_line(&mut self, line: ReceiptLine) {
        self.lines.push(line);
    }
}
