use eframe::egui;

use crate::parser::command::{Alignment, CutMode, UnderlineMode};
use crate::receipt::receipt::{Receipt, ReceiptEvent, ReceiptItem};

const PAPER_COLOR: egui::Color32 = egui::Color32::from_rgb(255, 250, 240);
const PAPER_WIDTH: f32 = 250.0;
const BASE_FONT_SIZE: f32 = 13.0;
const DOT_TO_PIXEL: f32 = 0.5;

fn line_advance_pixels(spacing: u8) -> f32 {
    spacing as f32 * DOT_TO_PIXEL
}

fn render_paper_section(ui: &mut egui::Ui, items: &[&ReceiptItem]) {
    egui::Frame::default()
        .fill(PAPER_COLOR)
        .stroke(egui::Stroke::new(1.5, egui::Color32::from_gray(150)))
        .inner_margin(egui::vec2(18.0, 14.0))
        .corner_radius(egui::CornerRadius::same(4))
        .show(ui, |ui| {
            let content_width = PAPER_WIDTH - 36.0;
            ui.set_width(content_width);

            ui.vertical(|ui| {
                ui.spacing_mut().item_spacing.y = 0.0;

                for item in items {
                    match item {
                        ReceiptItem::Line(line) => {
                            let text_width = line
                                .segments
                                .iter()
                                .map(|segment| {
                                    let font_size =
                                        BASE_FONT_SIZE * segment.char_size.height as f32;
                                    let font_id = egui::FontId::monospace(font_size);
                                    ui.fonts_mut(|fonts| {
                                        fonts
                                            .layout_no_wrap(
                                                segment.text.clone(),
                                                font_id,
                                                egui::Color32::PLACEHOLDER,
                                            )
                                            .size()
                                            .x
                                    })
                                })
                                .sum::<f32>();

                            let mut rendered_height = 0.0_f32;

                            ui.horizontal(|ui| {
                                match line.alignment {
                                    Alignment::Left => {}
                                    Alignment::Center => {
                                        ui.add_space(
                                            ((content_width - text_width) / 2.0).max(0.0),
                                        );
                                    }
                                    Alignment::Right => {
                                        ui.add_space((content_width - text_width).max(0.0));
                                    }
                                }

                                for segment in &line.segments {
                                    let font_size = BASE_FONT_SIZE * segment.char_size.height as f32;

                                    let mut text = egui::RichText::new(&segment.text)
                                        .font(egui::FontId::monospace(font_size));

                                    if segment.bold {
                                        text = text.color(egui::Color32::BLACK).strong();
                                    } else {
                                        text = text.color(egui::Color32::from_gray(70));
                                    }

                                    let response = ui.label(text);
                                    rendered_height = rendered_height.max(response.rect.height());

                                    match segment.underline {
                                        UnderlineMode::Off => {}
                                        UnderlineMode::Thin => {
                                            ui.painter().line_segment(
                                                [
                                                    egui::pos2(
                                                        response.rect.left(),
                                                        response.rect.bottom() - 1.0,
                                                    ),
                                                    egui::pos2(
                                                        response.rect.right(),
                                                        response.rect.bottom() - 1.0,
                                                    ),
                                                ],
                                                egui::Stroke::new(1.0, egui::Color32::BLACK),
                                            );
                                        }
                                        UnderlineMode::Thick => {
                                            ui.painter().line_segment(
                                                [
                                                    egui::pos2(
                                                        response.rect.left(),
                                                        response.rect.bottom() - 1.0,
                                                    ),
                                                    egui::pos2(
                                                        response.rect.right(),
                                                        response.rect.bottom() - 1.0,
                                                    ),
                                                ],
                                                egui::Stroke::new(2.0, egui::Color32::BLACK),
                                            );
                                        }
                                    }
                                }
                            });

                            let extra = (line_advance_pixels(line.spacing) - rendered_height).max(0.0);
                            ui.add_space(extra);
                        }

                        ReceiptItem::Event(ReceiptEvent::FeedLines { lines, spacing }) => {
                            ui.add_space(*lines as f32 * line_advance_pixels(*spacing));
                        }

                        ReceiptItem::Event(ReceiptEvent::FeedDots { dots }) => {
                            ui.add_space(*dots as f32 * DOT_TO_PIXEL);
                        }

                        ReceiptItem::Event(ReceiptEvent::Cut(_)) => {}
                    }
                }
            });
        });
}

pub fn render_receipt(ui: &mut egui::Ui, receipt: &Receipt) {
    ui.style_mut().override_font_id = Some(egui::FontId::monospace(BASE_FONT_SIZE));

    ui.vertical_centered(|ui| {
        let mut section: Vec<&ReceiptItem> = Vec::new();

        for item in &receipt.items {
            match item {
                ReceiptItem::Line(_)
                | ReceiptItem::Event(ReceiptEvent::FeedLines { .. })
                | ReceiptItem::Event(ReceiptEvent::FeedDots { .. }) => {
                    section.push(item);
                }

                ReceiptItem::Event(ReceiptEvent::Cut(cut_mode)) => {
                    if !section.is_empty() {
                        render_paper_section(ui, &section);
                        section.clear();
                    }

                    match cut_mode {
                        CutMode::Full => {
                            ui.add_space(24.0);
                        }
                        CutMode::Partial => {
                            let bridge_width = 70.0;
                            let bridge_height = 16.0;
                            let (rect, _) = ui.allocate_exact_size(
                                egui::vec2(PAPER_WIDTH, bridge_height),
                                egui::Sense::hover(),
                            );
                            let bridge_rect = egui::Rect::from_center_size(
                                rect.center(),
                                egui::vec2(bridge_width, bridge_height),
                            );
                            ui.painter().rect_filled(bridge_rect, 0.0, PAPER_COLOR);
                        }
                    }
                }
            }
        }

        if !section.is_empty() {
            render_paper_section(ui, &section);
        }
    });
}
