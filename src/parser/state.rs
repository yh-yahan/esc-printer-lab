#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParserState {
    Normal,
    Esc,
    EscAlignment,
    EscEmphasis,
    EscUnderline,
    EscPrintAndFeedLines,
    EscPrintAndFeedDots,
    EscLineSpacing,
    Gs,
    GsCut,
    GsCharSize,
}
