use super::command::{BarcodeSymbology, RasterScale};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParserState {
    Normal,
    Esc,
    EscAlignment,
    EscEmphasis,
    EscUnderline,
    EscCodePage,
    EscCharacterSet,
    EscPrintAndFeedLines,
    EscPrintAndFeedDots,
    EscLineSpacing,
    Gs,
    GsCut,
    GsCharSize,
    GsHriPosition,
    GsHriFont,
    GsBarWidth,
    GsBarHeight,
    GsBarcode,
    GsBarcodeNul {
        m: u8,
        symbology: BarcodeSymbology,
        data: Vec<u8>,
    },
    GsBarcodeLen {
        m: u8,
        symbology: BarcodeSymbology,
        n: Option<u8>,
        data: Vec<u8>,
        remaining: usize,
    },
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
    GsParen,
    GsParenHeader {
        ident: u8,
        bytes: Vec<u8>,
    },
    GsParenData {
        ident: u8,
        p_l: u8,
        p_h: u8,
        data: Vec<u8>,
        remaining: usize,
    },
}
