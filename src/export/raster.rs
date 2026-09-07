use crate::parser::command::{Alignment, CutMode, RasterImage, UnderlineMode};
use crate::printer::{FontMetrics, PrinterProfile, MISSING_GLYPH};
use crate::receipt::receipt::{Receipt, ReceiptEvent, ReceiptItem, ReceiptLine, ReceiptSegment};

const PAPER: [u8; 4] = [255, 250, 240, 255];
const PAPER_EDGE: [u8; 4] = [150, 150, 150, 255];
const INK: [u8; 4] = [20, 20, 20, 255];
const INK_LIGHT: [u8; 4] = [55, 55, 55, 255];
const TOP_PAD_DOTS: f32 = 16.0;
const BOTTOM_PAD_DOTS: f32 = 16.0;

#[derive(Clone, Copy)]
struct ExportOptions {
    profile: PrinterProfile,
    px_per_dot: f32,
}

impl ExportOptions {
    fn paper_px(self) -> u32 {
        (self.profile.paper_dots() * self.px_per_dot).round().max(1.0) as u32
    }

    fn margin_px(self) -> f32 {
        self.profile.side_margin_dots() * self.px_per_dot
    }

    fn dots_to_px(self, dots: f32) -> f32 {
        dots * self.px_per_dot
    }

    fn font(self) -> FontMetrics {
        self.profile.font_a
    }
}

struct VisualCell {
    ch: char,
    bold: bool,
    underline: UnderlineMode,
    width_mult: u8,
    height_mult: u8,
}

impl VisualCell {
    fn dots_w(&self, font: FontMetrics) -> u16 {
        font.cell_w as u16 * self.width_mult.max(1) as u16
    }

    fn dots_h(&self, font: FontMetrics) -> u16 {
        font.cell_h as u16 * self.height_mult.max(1) as u16
    }
}

struct VisualRow {
    cells: Vec<VisualCell>,
    dots_w: u16,
    dots_h: u16,
    alignment: Alignment,
    feed_dots: u16,
}

fn cells_from_segment(segment: &ReceiptSegment) -> impl Iterator<Item = VisualCell> + '_ {
    let bold = segment.bold;
    let underline = segment.underline;
    let width_mult = segment.char_size.width.max(1);
    let height_mult = segment.char_size.height.max(1);

    segment.text.chars().map(move |ch| VisualCell {
        ch,
        bold,
        underline,
        width_mult,
        height_mult,
    })
}

fn wrap_line(line: &ReceiptLine, profile: PrinterProfile) -> Vec<VisualRow> {
    let font = profile.font_a;
    let max_dots = profile.printable_dots;
    let mut rows = Vec::new();
    let mut current: Vec<VisualCell> = Vec::new();
    let mut used = 0u16;
    let mut row_h = 0u16;

    let flush = |rows: &mut Vec<VisualRow>,
                 current: &mut Vec<VisualCell>,
                 used: &mut u16,
                 row_h: &mut u16| {
        if current.is_empty() {
            return;
        }
        rows.push(VisualRow {
            cells: std::mem::take(current),
            dots_w: *used,
            dots_h: *row_h,
            alignment: line.alignment,
            feed_dots: line.spacing as u16,
        });
        *used = 0;
        *row_h = 0;
    };

    for segment in &line.segments {
        for cell in cells_from_segment(segment) {
            let w = cell.dots_w(font);
            let h = cell.dots_h(font);

            if w > max_dots {
                flush(&mut rows, &mut current, &mut used, &mut row_h);
                rows.push(VisualRow {
                    cells: vec![cell],
                    dots_w: w.min(max_dots),
                    dots_h: h,
                    alignment: line.alignment,
                    feed_dots: line.spacing as u16,
                });
                continue;
            }

            if used + w > max_dots {
                flush(&mut rows, &mut current, &mut used, &mut row_h);
            }

            used += w;
            row_h = row_h.max(h);
            current.push(cell);
        }
    }

    flush(&mut rows, &mut current, &mut used, &mut row_h);
    rows
}

fn row_advance_dots(row: &VisualRow) -> u16 {
    row.feed_dots.max(row.dots_h)
}

fn section_height_dots(items: &[&ReceiptItem], options: ExportOptions) -> f32 {
    let mut height = TOP_PAD_DOTS + BOTTOM_PAD_DOTS;

    if items.is_empty() {
        return height + options.profile.font_a.cell_h as f32 * 4.0;
    }

    for item in items {
        match item {
            ReceiptItem::Line(line) => {
                for row in wrap_line(line, options.profile) {
                    height += row_advance_dots(&row) as f32;
                }
            }
            ReceiptItem::Event(ReceiptEvent::FeedLines { lines, spacing }) => {
                height += *lines as f32 * *spacing as f32;
            }
            ReceiptItem::Event(ReceiptEvent::FeedDots { dots }) => {
                height += *dots as f32;
            }
            ReceiptItem::Event(ReceiptEvent::Cut(_)) => {}
            ReceiptItem::Event(ReceiptEvent::RasterImage { image, .. }) => {
                height += image.printed_height_dots() as f32;
            }
        }
    }

    height
}

fn align_offset_dots(alignment: Alignment, line_dots: u16, printable_dots: u16) -> f32 {
    let leftover = printable_dots.saturating_sub(line_dots) as f32;
    match alignment {
        Alignment::Left => 0.0,
        Alignment::Center => leftover / 2.0,
        Alignment::Right => leftover,
    }
}

struct Canvas {
    width: u32,
    height: u32,
    pixels: Vec<u8>,
}

impl Canvas {
    fn new(width: u32, height: u32) -> Self {
        let width = width.max(1);
        let height = height.max(1);
        let mut pixels = vec![0u8; width as usize * height as usize * 4];
        for px in pixels.chunks_exact_mut(4) {
            px.copy_from_slice(&PAPER);
        }
        Self {
            width,
            height,
            pixels,
        }
    }

    fn put(&mut self, x: i32, y: i32, color: [u8; 4]) {
        if x < 0 || y < 0 {
            return;
        }
        let x = x as u32;
        let y = y as u32;
        if x >= self.width || y >= self.height {
            return;
        }
        let i = ((y * self.width + x) * 4) as usize;
        self.pixels[i..i + 4].copy_from_slice(&color);
    }

    fn fill_rect(&mut self, x: i32, y: i32, w: i32, h: i32, color: [u8; 4]) {
        let x1 = x.max(0);
        let y1 = y.max(0);
        let x2 = (x + w).min(self.width as i32);
        let y2 = (y + h).min(self.height as i32);
        for yy in y1..y2 {
            for xx in x1..x2 {
                self.put(xx, yy, color);
            }
        }
    }

    fn hline(&mut self, x: i32, y: i32, w: i32, thickness: i32, color: [u8; 4]) {
        self.fill_rect(x, y, w, thickness.max(1), color);
    }

    fn into_rgba_image(self) -> image::RgbaImage {
        image::RgbaImage::from_raw(self.width, self.height, self.pixels)
            .expect("canvas size matches pixel buffer")
    }
}

fn glyph_bitmap(ch: char) -> Option<[u8; 8]> {
    use font8x8::UnicodeFonts;
    font8x8::BASIC_FONTS
        .get(ch)
        .or_else(|| font8x8::LATIN_FONTS.get(ch))
        .or_else(|| font8x8::GREEK_FONTS.get(ch))
        .or_else(|| font8x8::BLOCK_FONTS.get(ch))
        .or_else(|| font8x8::BOX_FONTS.get(ch))
        .or_else(|| font8x8::MISC_FONTS.get(ch))
}

fn paint_glyph(
    canvas: &mut Canvas,
    ch: char,
    x: i32,
    y: i32,
    cell_w: i32,
    cell_h: i32,
    bold: bool,
    color: [u8; 4],
) {
    if ch == MISSING_GLYPH || ch == '█' {
        canvas.fill_rect(x + 1, y + 1, (cell_w - 2).max(1), (cell_h - 2).max(1), color);
        return;
    }

    let Some(bitmap) = glyph_bitmap(ch) else {
        canvas.fill_rect(x + 1, y + 1, (cell_w - 2).max(1), (cell_h - 2).max(1), color);
        return;
    };

    let shifts = if bold { [0, 1] } else { [0, 0] };
    for row in 0i32..8 {
        let bits = bitmap[row as usize];
        for col in 0i32..8 {
            if bits & (1 << col) == 0 {
                continue;
            }
            let px0 = x + col * cell_w / 8;
            let py0 = y + row * cell_h / 8;
            let px1 = x + (col + 1) * cell_w / 8;
            let py1 = y + (row + 1) * cell_h / 8;
            for (i, shift) in shifts.iter().enumerate() {
                if i == 1 && *shift == 0 {
                    continue;
                }
                canvas.fill_rect(px0 + *shift, py0, (px1 - px0).max(1), (py1 - py0).max(1), color);
            }
        }
    }
}

fn paint_row(canvas: &mut Canvas, origin_x: f32, y_dots: f32, row: &VisualRow, options: ExportOptions) {
    let font = options.font();
    let content_left = origin_x + options.margin_px();
    let x0_dots = align_offset_dots(row.alignment, row.dots_w, options.profile.printable_dots);
    let mut cursor_dots = x0_dots;
    let top = options.dots_to_px(y_dots);

    for cell in &row.cells {
        let cell_w_dots = cell.dots_w(font) as f32;
        let cell_h_dots = cell.dots_h(font) as f32;
        let cell_w_px = options.dots_to_px(cell_w_dots).round() as i32;
        let cell_h_px = options.dots_to_px(cell_h_dots).round() as i32;
        let x = (content_left + options.dots_to_px(cursor_dots)).round() as i32;
        let y = top.round() as i32;
        let color = if cell.bold { INK } else { INK_LIGHT };

        paint_glyph(
            canvas,
            cell.ch,
            x,
            y,
            cell_w_px.max(1),
            cell_h_px.max(1),
            cell.bold,
            color,
        );

        match cell.underline {
            UnderlineMode::Off => {}
            UnderlineMode::Thin => {
                let uy = y + cell_h_px - options.dots_to_px(2.0).round() as i32;
                canvas.hline(x, uy, cell_w_px, 1, INK);
            }
            UnderlineMode::Thick => {
                let uy = y + cell_h_px - options.dots_to_px(2.0).round() as i32;
                canvas.hline(x, uy, cell_w_px, 2, INK);
            }
        }

        cursor_dots += cell_w_dots;
    }
}

fn paint_raster(
    canvas: &mut Canvas,
    origin_x: f32,
    y_dots: f32,
    alignment: Alignment,
    image: &RasterImage,
    options: ExportOptions,
) {
    let src_w = image.width_dots() as i32;
    let src_h = image.height as i32;
    if src_w <= 0 || src_h <= 0 || image.data.is_empty() {
        return;
    }

    let printed_w = image.printed_width_dots() as u16;
    let printed_h = image.printed_height_dots() as f32;
    let x0 = align_offset_dots(
        alignment,
        printed_w.min(options.profile.printable_dots),
        options.profile.printable_dots,
    );
    let dest_x = (origin_x + options.margin_px() + options.dots_to_px(x0)).round() as i32;
    let dest_y = options.dots_to_px(y_dots).round() as i32;
    let dest_w = options.dots_to_px(printed_w as f32).round().max(1.0) as i32;
    let dest_h = options.dots_to_px(printed_h).round().max(1.0) as i32;
    let x_mult = image.scale.width_mult().max(1) as i32;
    let y_mult = image.scale.height_mult().max(1) as i32;
    let row_bytes = image.width_bytes as usize;

    for dy in 0..dest_h {
        let src_y = ((dy * src_h * y_mult) / dest_h).min(src_h * y_mult - 1) / y_mult;
        for dx in 0..dest_w {
            let src_x = ((dx * src_w * x_mult) / dest_w).min(src_w * x_mult - 1) / x_mult;
            let xb = src_x as usize / 8;
            let bit = src_x as usize % 8;
            let byte = image
                .data
                .get(src_y as usize * row_bytes + xb)
                .copied()
                .unwrap_or(0);
            if byte & (0x80 >> bit) != 0 {
                canvas.put(dest_x + dx, dest_y + dy, INK);
            }
        }
    }
}

fn paint_section(canvas: &mut Canvas, y0: u32, items: &[&ReceiptItem], options: ExportOptions) {
    let paper_w = options.paper_px();
    let paper_h = options
        .dots_to_px(section_height_dots(items, options))
        .max(options.dots_to_px(48.0))
        .round()
        .max(1.0) as u32;

    canvas.fill_rect(0, y0 as i32, paper_w as i32, paper_h as i32, PAPER);
    canvas.fill_rect(0, y0 as i32, paper_w as i32, 1, PAPER_EDGE);
    canvas.fill_rect(0, y0 as i32 + paper_h as i32 - 1, paper_w as i32, 1, PAPER_EDGE);
    canvas.fill_rect(0, y0 as i32, 1, paper_h as i32, PAPER_EDGE);
    canvas.fill_rect(paper_w as i32 - 1, y0 as i32, 1, paper_h as i32, PAPER_EDGE);

    let mut y_dots = TOP_PAD_DOTS;
    for item in items {
        match item {
            ReceiptItem::Line(line) => {
                for row in wrap_line(line, options.profile) {
                    paint_row(canvas, 0.0, y0 as f32 / options.px_per_dot + y_dots, &row, options);
                    y_dots += row_advance_dots(&row) as f32;
                }
            }
            ReceiptItem::Event(ReceiptEvent::FeedLines { lines, spacing }) => {
                y_dots += *lines as f32 * *spacing as f32;
            }
            ReceiptItem::Event(ReceiptEvent::FeedDots { dots }) => {
                y_dots += *dots as f32;
            }
            ReceiptItem::Event(ReceiptEvent::Cut(_)) => {}
            ReceiptItem::Event(ReceiptEvent::RasterImage { alignment, image }) => {
                paint_raster(
                    canvas,
                    0.0,
                    y0 as f32 / options.px_per_dot + y_dots,
                    *alignment,
                    image,
                    options,
                );
                y_dots += image.printed_height_dots() as f32;
            }
        }
    }
}

fn split_sections(receipt: &Receipt) -> (Vec<Vec<&ReceiptItem>>, Vec<CutMode>) {
    let mut sections: Vec<Vec<&ReceiptItem>> = Vec::new();
    let mut cuts: Vec<CutMode> = Vec::new();
    let mut current: Vec<&ReceiptItem> = Vec::new();

    for item in &receipt.items {
        match item {
            ReceiptItem::Line(_)
            | ReceiptItem::Event(ReceiptEvent::FeedLines { .. })
            | ReceiptItem::Event(ReceiptEvent::FeedDots { .. })
            | ReceiptItem::Event(ReceiptEvent::RasterImage { .. }) => {
                current.push(item);
            }
            ReceiptItem::Event(ReceiptEvent::Cut(cut_mode)) => {
                sections.push(std::mem::take(&mut current));
                cuts.push(*cut_mode);
            }
        }
    }

    if current.is_empty() && sections.is_empty() && receipt.items.is_empty() {
        sections.push(Vec::new());
    } else if !current.is_empty() {
        sections.push(current);
    }

    (sections, cuts)
}

pub fn render_receipt_rgba(receipt: &Receipt, profile: PrinterProfile, px_per_dot: f32) -> image::RgbaImage {
    let options = ExportOptions {
        profile,
        px_per_dot: px_per_dot.max(0.5),
    };
    let paper_w = options.paper_px();
    let (sections, cuts) = split_sections(receipt);

    let mut heights = Vec::with_capacity(sections.len());
    let mut total_h = 0u32;
    for (index, section) in sections.iter().enumerate() {
        let h = options
            .dots_to_px(section_height_dots(section, options))
            .max(options.dots_to_px(48.0))
            .round()
            .max(1.0) as u32;
        heights.push(h);
        total_h += h;
        if index < cuts.len() {
            let gap = match cuts[index] {
                CutMode::Full => options.dots_to_px(28.0),
                CutMode::Partial => options.dots_to_px(18.0),
            };
            total_h += gap.round().max(1.0) as u32;
        }
    }

    let mut canvas = Canvas::new(paper_w, total_h.max(1));
    for px in canvas.pixels.chunks_exact_mut(4) {
        px.copy_from_slice(&[236, 228, 214, 255]);
    }

    let mut y = 0u32;
    for (index, section) in sections.iter().enumerate() {
        paint_section(&mut canvas, y, section, options);
        y += heights[index];
        if index < cuts.len() {
            let gap = match cuts[index] {
                CutMode::Full => options.dots_to_px(28.0).round().max(1.0) as u32,
                CutMode::Partial => options.dots_to_px(18.0).round().max(1.0) as u32,
            };
            if matches!(cuts[index], CutMode::Partial) {
                let bridge_w = ((paper_w as f32) * 0.28).round() as i32;
                let x = (paper_w as i32 - bridge_w) / 2;
                canvas.fill_rect(x, y as i32, bridge_w, gap as i32, PAPER);
            }
            y += gap;
        }
    }

    canvas.into_rgba_image()
}
