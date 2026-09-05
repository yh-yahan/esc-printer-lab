use crate::parser::command::{Alignment, CharSize, CutMode, RasterImage, UnderlineMode};

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
    pub spacing: u8,
}

#[derive(Debug, Clone)]
pub struct ReceiptSegment {
    pub text: String,
    pub bold: bool,
    pub underline: UnderlineMode,
    pub char_size: CharSize,
}

#[derive(Debug, Clone)]
pub enum ReceiptEvent {
    Cut(CutMode),
    FeedDots { dots: u8 },
    FeedLines { lines: u8, spacing: u8 },
    RasterImage {
        alignment: Alignment,
        image: RasterImage,
    },
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
