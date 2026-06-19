#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParserState {
    Normal,
    Esc,
    EscAlignment,
    EscEmphasis,
    EscUnderline,
}
