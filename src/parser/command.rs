#[derive(Debug, Clone)]
pub enum Command {
    Initialize,
    Text(String),
    SelectCodePage { n: u8, applied: bool },
    SelectCharacterSet { n: u8, applied: bool },
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
    RasterImage(RasterImage),
    Qr(QrCommand),
    Barcode(BarcodeCommand),
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RasterScale {
    Normal,
    DoubleWidth,
    DoubleHeight,
    Quadruple,
}

impl RasterScale {
    pub fn from_m(m: u8) -> Option<Self> {
        match m {
            0 | 48 => Some(Self::Normal),
            1 | 49 => Some(Self::DoubleWidth),
            2 | 50 => Some(Self::DoubleHeight),
            3 | 51 => Some(Self::Quadruple),
            _ => None,
        }
    }

    pub fn m(self) -> u8 {
        match self {
            Self::Normal => 0,
            Self::DoubleWidth => 1,
            Self::DoubleHeight => 2,
            Self::Quadruple => 3,
        }
    }

    pub fn width_mult(self) -> u8 {
        match self {
            Self::Normal | Self::DoubleHeight => 1,
            Self::DoubleWidth | Self::Quadruple => 2,
        }
    }

    pub fn height_mult(self) -> u8 {
        match self {
            Self::Normal | Self::DoubleWidth => 1,
            Self::DoubleHeight | Self::Quadruple => 2,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RasterImage {
    pub scale: RasterScale,
    pub width_bytes: u16,
    pub height: u16,
    pub data: Vec<u8>,
}

impl RasterImage {
    pub fn width_dots(&self) -> u32 {
        self.width_bytes as u32 * 8
    }

    pub fn printed_width_dots(&self) -> u32 {
        self.width_dots() * self.scale.width_mult() as u32
    }

    pub fn printed_height_dots(&self) -> u32 {
        self.height as u32 * self.scale.height_mult() as u32
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QrEcLevel {
    L,
    M,
    Q,
    H,
}

impl QrEcLevel {
    pub fn from_n(n: u8) -> Option<Self> {
        match n {
            48 => Some(Self::L),
            49 => Some(Self::M),
            50 => Some(Self::Q),
            51 => Some(Self::H),
            _ => None,
        }
    }

    pub fn n(self) -> u8 {
        match self {
            Self::L => 48,
            Self::M => 49,
            Self::Q => 50,
            Self::H => 51,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::L => "L",
            Self::M => "M",
            Self::Q => "Q",
            Self::H => "H",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QrCommand {
    SetModel { model: u8 },
    SetModuleSize { size: u8 },
    SetErrorCorrection { level: QrEcLevel },
    Store { data: Vec<u8> },
    Print,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BarcodeSymbology {
    UpcA,
    UpcE,
    Ean13,
    Ean8,
    Code39,
    Itf,
    Codabar,
    Code93,
    Code128,
}

impl BarcodeSymbology {
    pub fn from_m(m: u8) -> Option<Self> {
        match m {
            0 | 65 => Some(Self::UpcA),
            1 | 66 => Some(Self::UpcE),
            2 | 67 => Some(Self::Ean13),
            3 | 68 => Some(Self::Ean8),
            4 | 69 => Some(Self::Code39),
            5 | 70 => Some(Self::Itf),
            6 | 71 => Some(Self::Codabar),
            72 => Some(Self::Code93),
            73 => Some(Self::Code128),
            _ => None,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::UpcA => "UPC-A",
            Self::UpcE => "UPC-E",
            Self::Ean13 => "EAN-13",
            Self::Ean8 => "EAN-8",
            Self::Code39 => "CODE39",
            Self::Itf => "ITF",
            Self::Codabar => "CODABAR",
            Self::Code93 => "CODE93",
            Self::Code128 => "CODE128",
        }
    }

    pub fn is_function_a(m: u8) -> bool {
        matches!(m, 0..=6)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HriPosition {
    None,
    Above,
    Below,
    Both,
}

impl HriPosition {
    pub fn from_n(n: u8) -> Option<Self> {
        match n {
            0 | 48 => Some(Self::None),
            1 | 49 => Some(Self::Above),
            2 | 50 => Some(Self::Below),
            3 | 51 => Some(Self::Both),
            _ => None,
        }
    }

    pub fn n(self) -> u8 {
        match self {
            Self::None => 0,
            Self::Above => 1,
            Self::Below => 2,
            Self::Both => 3,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::None => "not printed",
            Self::Above => "above",
            Self::Below => "below",
            Self::Both => "above and below",
        }
    }

    pub fn above(self) -> bool {
        matches!(self, Self::Above | Self::Both)
    }

    pub fn below(self) -> bool {
        matches!(self, Self::Below | Self::Both)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HriFont {
    A,
    B,
}

impl HriFont {
    pub fn from_n(n: u8) -> Option<Self> {
        match n {
            0 | 48 => Some(Self::A),
            1 | 49 => Some(Self::B),
            _ => None,
        }
    }

    pub fn n(self) -> u8 {
        match self {
            Self::A => 0,
            Self::B => 1,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BarcodeCommand {
    SetHriPosition(HriPosition),
    SetHriFont(HriFont),
    SetWidth(u8),
    SetHeight(u8),
    Print {
        m: u8,
        symbology: BarcodeSymbology,
        data: Vec<u8>,
    },
}
