use eframe::egui;
use std::sync::{Arc, Mutex};

use crate::receipt::receipt::Receipt;
use crate::ui::receipt_view::render_receipt;
use crate::ui::inspector::{Inspector, InspectorTab};
use crate::shared::print_session::PrintSession;

pub struct App {
    session: Arc<Mutex<PrintSession>>,
    selected_tab: InspectorTab,
}

impl App {
    pub fn new(session: Arc<Mutex<PrintSession>>) -> Self {
        Self {
            session,
            selected_tab: InspectorTab::EscPos,
        }
    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        ui.set_visuals(egui::Visuals::dark());

        let session = self.session.lock().unwrap();

        ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new(format!(
                    "Receipts: {}",
                    session.receipts.len()
                ))
            );

            ui.with_layout(
                egui::Layout::right_to_left(egui::Align::Center),
                |ui| {
                    let clear_button = egui::Button::new(
                        egui::RichText::new("Clear").strong(),
                    )
                    .min_size(egui::vec2(100.0, 32.0));

                    if ui.add(clear_button).clicked() {
                        drop(session);
                        self.session.lock().unwrap().clear();
                        return;
                    }
                },
            );
        });

        ui.separator();

        ui.columns(2, |columns| {
            egui::ScrollArea::vertical()
                .id_source("receipt_preview_scroll")
                .auto_shrink([false; 2])
                .show(&mut columns[0], |ui| {
                    let session = self.session.lock().unwrap();

                    ui.vertical_centered(|ui| {
                        if session.receipts.is_empty() {
                            render_receipt(ui, &Receipt { items: vec![] });
                        } else {
                            let mut combined_items = Vec::new();

                            for receipt in &session.receipts {
                                combined_items.extend(receipt.items.iter().cloned());
                            }

                            let combined = Receipt {
                                items: combined_items,
                            };

                            render_receipt(ui, &combined);
                        }
                    });
                });

            columns[1].separator();

            Inspector::show(
                &mut columns[1],
                &mut self.selected_tab,
                &self.session,
            );
        });
    }
}
