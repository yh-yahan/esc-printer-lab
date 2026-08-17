use crate::parser::command::{Alignment, UnderlineMode, CutMode};

#[derive(Debug, Clone, Default)]
pub struct Receipt {
    pub items: Vec<ReceiptItem>,
}

#[derive(Debug, Clone)]
pub enum ReceiptItem {
    Line(ReceiptLine),
    Event(ReceiptEvent),
}

#[derive(Debug, Clone)]
pub struct ReceiptLine {
    pub alignment: Alignment,
    pub segments: Vec<ReceiptSegment>,
}

#[derive(Debug, Clone)]
pub struct ReceiptSegment {
    pub text: String,
    pub bold: bool,
    pub underline: UnderlineMode,
}

#[derive(Debug, Clone)]
pub enum ReceiptEvent {
    Cut(CutMode),
}

impl Receipt {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_line(&mut self, line: ReceiptLine) {
        self.items.push(ReceiptItem::Line(line));
    }

    pub fn add_event(&mut self, event: ReceiptEvent) {
        self.items.push(ReceiptItem::Event(event));
    }
}

