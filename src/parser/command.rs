#[derive(Debug, Clone)]
pub enum Command {
    Initialize,
    Text(String),
    LineFeed,
    CarriageReturn,
    PrintAndFeedLines(u8),
    PrintAndFeedDots(u8),
    SetDefaultLineSpacing,
    SetLineSpacing(u8),
    Bold(bool),
    Align(Alignment),
    Underline(UnderlineMode),
    Cut(CutMode),
    CharSize(CharSize),
    Unknown(Vec<u8>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Alignment {
    Left,
    Center,
    Right
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnderlineMode {
    Off,
    Thin,
    Thick
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CutMode {
    Full,
    Partial
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CharSize {
    pub width: u8,
    pub height: u8,
}
