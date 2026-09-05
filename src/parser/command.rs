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
    RasterImage(RasterImage),
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
