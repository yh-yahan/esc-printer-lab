use super::command::RasterScale;

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
    GsRasterZero,
    GsRasterHeader {
        bytes: Vec<u8>,
    },
    GsRasterData {
        scale: RasterScale,
        width_bytes: u16,
        height: u16,
        data: Vec<u8>,
        remaining: usize,
    },
}
