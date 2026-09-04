#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FontMetrics {
    pub cell_w: u8,
    pub cell_h: u8,
}

impl FontMetrics {
    pub const FONT_A: Self = Self {
        cell_w: 12,
        cell_h: 24,
    };

    pub const FONT_B: Self = Self {
        cell_w: 9,
        cell_h: 17,
    };

    pub fn columns_per_line(self, printable_dots: u16) -> u16 {
        printable_dots / self.cell_w as u16
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PrinterProfile {
    pub id: ProfileId,
    pub name: &'static str,
    pub paper_width_mm: f32,
    pub printable_dots: u16,
    pub dpi: u16,
    pub font_a: FontMetrics,
    pub font_b: FontMetrics,
    pub default_line_spacing: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProfileId {
    Epson80mm180,
    Generic80mm203,
    Generic58mm203,
}

impl PrinterProfile {
    pub const EPSON_80MM_180: Self = Self {
        id: ProfileId::Epson80mm180,
        name: "80mm · 512 dots · 180 dpi (Epson TM-T88)",
        paper_width_mm: 80.0,
        printable_dots: 512,
        dpi: 180,
        font_a: FontMetrics::FONT_A,
        font_b: FontMetrics::FONT_B,
        default_line_spacing: 30,
    };

    pub const GENERIC_80MM_203: Self = Self {
        id: ProfileId::Generic80mm203,
        name: "80mm · 576 dots · 203 dpi",
        paper_width_mm: 80.0,
        printable_dots: 576,
        dpi: 203,
        font_a: FontMetrics::FONT_A,
        font_b: FontMetrics::FONT_B,
        default_line_spacing: 30,
    };

    pub const GENERIC_58MM_203: Self = Self {
        id: ProfileId::Generic58mm203,
        name: "58mm · 384 dots · 203 dpi",
        paper_width_mm: 58.0,
        printable_dots: 384,
        dpi: 203,
        font_a: FontMetrics::FONT_A,
        font_b: FontMetrics::FONT_B,
        default_line_spacing: 30,
    };

    pub const ALL: [Self; 3] = [
        Self::EPSON_80MM_180,
        Self::GENERIC_80MM_203,
        Self::GENERIC_58MM_203,
    ];

    pub fn by_id(id: ProfileId) -> Self {
        Self::ALL
            .into_iter()
            .find(|profile| profile.id == id)
            .unwrap_or(Self::EPSON_80MM_180)
    }

    pub fn paper_dots(self) -> f32 {
        self.paper_width_mm * self.dpi as f32 / 25.4
    }

    pub fn printable_mm(self) -> f32 {
        self.printable_dots as f32 * 25.4 / self.dpi as f32
    }

    pub fn side_margin_dots(self) -> f32 {
        ((self.paper_dots() - self.printable_dots as f32) / 2.0).max(0.0)
    }

    pub fn font_a_columns(self) -> u16 {
        self.font_a.columns_per_line(self.printable_dots)
    }

    pub fn summary(self) -> String {
        format!(
            "printable {} dots ({:.1} mm) · Font A {}x{} · {} cpl · spacing {} dots",
            self.printable_dots,
            self.printable_mm(),
            self.font_a.cell_w,
            self.font_a.cell_h,
            self.font_a_columns(),
            self.default_line_spacing
        )
    }
}

impl Default for PrinterProfile {
    fn default() -> Self {
        Self::EPSON_80MM_180
    }
}
