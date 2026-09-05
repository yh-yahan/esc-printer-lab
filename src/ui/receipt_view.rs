use eframe::egui::{self, Align2, FontId, Pos2, Rect, Stroke, Vec2};

use crate::parser::command::{Alignment, CutMode, RasterImage, UnderlineMode};
use crate::printer::{FontMetrics, PrinterProfile, MISSING_GLYPH};
use crate::receipt::receipt::{Receipt, ReceiptEvent, ReceiptItem, ReceiptLine, ReceiptSegment};

const PAPER_COLOR: egui::Color32 = egui::Color32::from_rgb(255, 250, 240);
const PAPER_EDGE: egui::Color32 = egui::Color32::from_gray(150);
const INK: egui::Color32 = egui::Color32::from_rgb(20, 20, 20);
const INK_LIGHT: egui::Color32 = egui::Color32::from_gray(55);
const RULER: egui::Color32 = egui::Color32::from_rgb(110, 80, 40);
const RULER_LABEL: egui::Color32 = egui::Color32::from_rgb(90, 60, 25);
const TOP_PAD_DOTS: f32 = 16.0;
const RULER_PAD_DOTS: f32 = 36.0;
const BOTTOM_PAD_DOTS: f32 = 16.0;

#[derive(Debug, Clone, Copy)]
pub struct PreviewOptions {
    pub profile: PrinterProfile,
    pub px_per_dot: f32,
    pub show_ruler: bool,
}

impl PreviewOptions {
    fn paper_px(self) -> f32 {
        self.profile.paper_dots() * self.px_per_dot
    }

    fn printable_px(self) -> f32 {
        self.profile.printable_dots as f32 * self.px_per_dot
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

    fn top_pad_dots(self) -> f32 {
        if self.show_ruler {
            RULER_PAD_DOTS
        } else {
            TOP_PAD_DOTS
        }
    }
}

#[derive(Clone)]
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

    let flush = |rows: &mut Vec<VisualRow>, current: &mut Vec<VisualCell>, used: &mut u16, row_h: &mut u16| {
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

fn section_height_dots(items: &[&ReceiptItem], options: PreviewOptions) -> f32 {
    let mut height = options.top_pad_dots() + BOTTOM_PAD_DOTS;

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

fn paint_ruler(painter: &egui::Painter, origin: Pos2, options: PreviewOptions) {
    let font = options.font();
    let content_left = origin.x + options.margin_px();
    let content_right = content_left + options.printable_px();
    let pad_px = options.dots_to_px(options.top_pad_dots());
    let baseline = origin.y + pad_px - options.dots_to_px(4.0);
    let minor_top = baseline - 6.0_f32;
    let major_top = baseline - 12.0_f32;
    let columns = options.profile.font_a_columns();

    painter.line_segment(
        [Pos2::new(content_left, baseline), Pos2::new(content_right, baseline)],
        Stroke::new(1.0_f32, RULER),
    );

    for col in 0..=columns {
        let x = content_left + options.dots_to_px((col * font.cell_w as u16) as f32);
        let major = col % 8 == 0;
        painter.line_segment(
            [
                Pos2::new(x, if major { major_top } else { minor_top }),
                Pos2::new(x, baseline),
            ],
            Stroke::new(if major { 1.2_f32 } else { 1.0_f32 }, RULER),
        );
        if major {
            painter.text(
                Pos2::new(x, major_top - 1.0_f32),
                Align2::CENTER_BOTTOM,
                col.to_string(),
                FontId::monospace(10.0_f32),
                RULER_LABEL,
            );
        }
    }
}

fn paint_row(
    painter: &egui::Painter,
    origin: Pos2,
    y_dots: f32,
    row: &VisualRow,
    options: PreviewOptions,
) {
    let font = options.font();
    let content_left = origin.x + options.margin_px();
    let x0_dots = align_offset_dots(row.alignment, row.dots_w, options.profile.printable_dots);
    let mut cursor_dots = x0_dots;
    let top = origin.y + options.dots_to_px(y_dots);

    for cell in &row.cells {
        let cell_w_dots = cell.dots_w(font) as f32;
        let cell_h_dots = cell.dots_h(font) as f32;
        let cell_w_px = options.dots_to_px(cell_w_dots);
        let cell_h_px = options.dots_to_px(cell_h_dots);
        let x = content_left + options.dots_to_px(cursor_dots);

        let font_size = (cell_h_px * 0.92).max(1.0);
        let color = if cell.bold { INK } else { INK_LIGHT };

        if cell.ch == MISSING_GLYPH {
            let pad = options.dots_to_px(2.0);
            painter.rect_filled(
                Rect::from_min_size(
                    Pos2::new(x + pad, top + pad),
                    Vec2::new((cell_w_px - pad * 2.0).max(1.0), (cell_h_px - pad * 2.0).max(1.0)),
                ),
                0.0,
                color,
            );
        } else {
            let font_id = if cell.bold {
                FontId::new(font_size, egui::FontFamily::Monospace)
            } else {
                FontId::monospace(font_size)
            };

            painter.text(
                Pos2::new(x, top),
                Align2::LEFT_TOP,
                cell.ch.to_string(),
                font_id,
                color,
            );
        }

        match cell.underline {
            UnderlineMode::Off => {}
            UnderlineMode::Thin => {
                let y = top + cell_h_px - options.dots_to_px(2.0);
                painter.line_segment(
                    [Pos2::new(x, y), Pos2::new(x + cell_w_px, y)],
                    Stroke::new(1.0_f32, INK),
                );
            }
            UnderlineMode::Thick => {
                let y = top + cell_h_px - options.dots_to_px(2.0);
                painter.line_segment(
                    [Pos2::new(x, y), Pos2::new(x + cell_w_px, y)],
                    Stroke::new(2.0_f32, INK),
                );
            }
        }

        cursor_dots += cell_w_dots;
    }
}

fn raster_to_color_image(image: &RasterImage) -> egui::ColorImage {
    let width = image.width_dots() as usize;
    let height = image.height as usize;
    let mut rgba = vec![0u8; width.saturating_mul(height).saturating_mul(4)];

    for px in rgba.chunks_exact_mut(4) {
        px[0] = 255;
        px[1] = 250;
        px[2] = 240;
        px[3] = 255;
    }

    let row_bytes = image.width_bytes as usize;
    for y in 0..height {
        for xb in 0..row_bytes {
            let byte = image
                .data
                .get(y.saturating_mul(row_bytes) + xb)
                .copied()
                .unwrap_or(0);
            for bit in 0..8 {
                if byte & (0x80 >> bit) == 0 {
                    continue;
                }
                let x = xb * 8 + bit;
                if x >= width {
                    continue;
                }
                let i = (y * width + x) * 4;
                if i + 3 < rgba.len() {
                    rgba[i] = 20;
                    rgba[i + 1] = 20;
                    rgba[i + 2] = 20;
                    rgba[i + 3] = 255;
                }
            }
        }
    }

    egui::ColorImage::from_rgba_unmultiplied([width, height], &rgba)
}

fn paint_raster(
    ui: &egui::Ui,
    painter: &egui::Painter,
    origin: Pos2,
    y_dots: f32,
    alignment: Alignment,
    image: &RasterImage,
    options: PreviewOptions,
) {
    let src_w = image.width_dots();
    let src_h = image.height as u32;
    if src_w == 0 || src_h == 0 || image.data.is_empty() {
        return;
    }

    let printed_w = image.printed_width_dots() as u16;
    let printed_h = image.printed_height_dots() as f32;
    let x0 = align_offset_dots(
        alignment,
        printed_w.min(options.profile.printable_dots),
        options.profile.printable_dots,
    );
    let dest = Rect::from_min_size(
        Pos2::new(
            origin.x + options.margin_px() + options.dots_to_px(x0),
            origin.y + options.dots_to_px(y_dots),
        ),
        Vec2::new(options.dots_to_px(printed_w as f32), options.dots_to_px(printed_h)),
    );

    let hash = image
        .data
        .iter()
        .fold(0u32, |acc, byte| acc.wrapping_mul(16777619) ^ u32::from(*byte));
    let texture = ui.ctx().load_texture(
        format!("gs-v0-{src_w}x{src_h}-{hash}"),
        raster_to_color_image(image),
        egui::TextureOptions::NEAREST,
    );

    painter.image(
        texture.id(),
        dest,
        Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
        egui::Color32::WHITE,
    );
}

fn render_paper_section(ui: &mut egui::Ui, items: &[&ReceiptItem], options: PreviewOptions) {
    let paper_w = options.paper_px();
    let paper_h = options
        .dots_to_px(section_height_dots(items, options))
        .max(options.dots_to_px(48.0));
    let (rect, _) = ui.allocate_exact_size(Vec2::new(paper_w, paper_h), egui::Sense::hover());

    let painter = ui.painter_at(rect);
    painter.rect(
        rect,
        3.0,
        PAPER_COLOR,
        Stroke::new(1.5_f32, PAPER_EDGE),
        egui::StrokeKind::Inside,
    );

    let printable = Rect::from_min_size(
        Pos2::new(rect.left() + options.margin_px(), rect.top()),
        Vec2::new(options.printable_px(), rect.height()),
    );
    painter.rect_stroke(
        printable,
        0.0,
        Stroke::new(
            1.0_f32,
            egui::Color32::from_rgba_premultiplied(180, 160, 120, 40),
        ),
        egui::StrokeKind::Inside,
    );

    if options.show_ruler {
        paint_ruler(&painter, rect.min, options);
    }

    let content_painter = painter.with_clip_rect(printable);
    let mut y_dots = options.top_pad_dots();

    for item in items {
        match item {
            ReceiptItem::Line(line) => {
                for row in wrap_line(line, options.profile) {
                    paint_row(&content_painter, rect.min, y_dots, &row, options);
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
                    ui,
                    &content_painter,
                    rect.min,
                    y_dots,
                    *alignment,
                    image,
                    options,
                );
                y_dots += image.printed_height_dots() as f32;
            }
        }
    }
}

pub fn render_receipt(ui: &mut egui::Ui, receipt: &Receipt, options: PreviewOptions) {
    ui.spacing_mut().item_spacing.y = 0.0;

    ui.vertical_centered(|ui| {
        let mut section: Vec<&ReceiptItem> = Vec::new();

        for item in &receipt.items {
            match item {
                ReceiptItem::Line(_)
                | ReceiptItem::Event(ReceiptEvent::FeedLines { .. })
                | ReceiptItem::Event(ReceiptEvent::FeedDots { .. })
                | ReceiptItem::Event(ReceiptEvent::RasterImage { .. }) => {
                    section.push(item);
                }

                ReceiptItem::Event(ReceiptEvent::Cut(cut_mode)) => {
                    if !section.is_empty() {
                        render_paper_section(ui, &section, options);
                        section.clear();
                    } else {
                        render_paper_section(ui, &[], options);
                    }

                    match cut_mode {
                        CutMode::Full => {
                            ui.add_space(options.dots_to_px(28.0));
                        }
                        CutMode::Partial => {
                            let gap = options.dots_to_px(18.0);
                            let (rect, _) = ui.allocate_exact_size(
                                Vec2::new(options.paper_px(), gap),
                                egui::Sense::hover(),
                            );
                            let bridge = Rect::from_center_size(
                                rect.center(),
                                Vec2::new(options.paper_px() * 0.28, gap),
                            );
                            ui.painter().rect_filled(bridge, 0.0, PAPER_COLOR);
                        }
                    }
                }
            }
        }

        if section.is_empty() && receipt.items.is_empty() {
            render_paper_section(ui, &[], options);
        } else if !section.is_empty() {
            render_paper_section(ui, &section, options);
        }
    });
}
