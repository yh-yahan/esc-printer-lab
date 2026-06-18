use eframe::egui;
use crate::receipt::receipt::Receipt;

pub fn render_receipt(ui: &mut egui::Ui, receipt: &Receipt) {
    ui.style_mut().override_font_id = Some(egui::FontId::monospace(13.0));

    ui.horizontal(|ui| {
        let receipt_width = 260.0; // 58mm

        egui::Frame::default()
            .fill(egui::Color32::from_rgb(255, 250, 240))
            .stroke(egui::Stroke::new(1.5, egui::Color32::from_gray(150)))
            .inner_margin(egui::vec2(18.0, 14.0))
            .rounding(egui::CornerRadius::same(4))
            .show(ui, |ui| {
                ui.set_width(receipt_width);

                ui.vertical(|ui| {
                    ui.add_space(8.0);

                    for line in &receipt.lines {
                        let text = if line.bold {
                            egui::RichText::new(&line.text).strong()
                        } else {
                            egui::RichText::new(&line.text)
                        };

                        match line.alignment {
                            crate::parser::command::Alignment::Left => {
                                ui.label(text);
                            }

                            crate::parser::command::Alignment::Center => {
                                let font_id = egui::FontId::monospace(13.0);
                                let galley = ui.fonts_mut(|fonts| {
                                    fonts.layout_no_wrap(
                                        line.text.clone(),
                                        font_id.clone(),
                                        egui::Color32::PLACEHOLDER,
                                    )
                                });
                                let text_width = galley.size().x;
                                let paper_width = receipt_width - 36.0;

                                ui.horizontal(|ui| {
                                    if paper_width > text_width {
                                        ui.add_space((paper_width - text_width) / 2.0);
                                    }
                                    ui.label(text);
                                });
                            }

                            crate::parser::command::Alignment::Right => {
                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Center),
                                    |ui| {
                                        ui.label(text);
                                    },
                                );
                            }
                        }
                    }

                    ui.add_space(8.0);
                });
            });
    });
}
