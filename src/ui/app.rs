use eframe::egui;
use std::sync::{Arc, Mutex};

use crate::receipt::receipt::Receipt;
use crate::shared::print_session::PrintSession;
use crate::ui::inspector::{Inspector, InspectorDock, InspectorTab};
use crate::ui::receipt_view::render_receipt;

pub struct App {
    session: Arc<Mutex<PrintSession>>,
    selected_tab: InspectorTab,
    dock: InspectorDock,
}

impl App {
    pub fn new(session: Arc<Mutex<PrintSession>>) -> Self {
        Self {
            session,
            selected_tab: InspectorTab::EscPos,
            dock: InspectorDock::Right,
        }
    }

    fn show_preview(&self, ui: &mut egui::Ui) {
        egui::ScrollArea::vertical()
            .id_salt("receipt_preview_scroll")
            .auto_shrink([false; 2])
            .show(ui, |ui| {
                let session = self.session.lock().unwrap();

                ui.vertical_centered(|ui| {
                    if session.receipts.is_empty() {
                        render_receipt(
                            ui,
                            &Receipt {
                                items: vec![],
                            },
                        );
                    } else {
                        let mut combined_items = Vec::new();

                        for receipt in &session.receipts {
                            combined_items.extend(
                                receipt.items.iter().cloned(),
                            );
                        }

                        let combined = Receipt {
                            items: combined_items,
                        };

                        render_receipt(ui, &combined);
                    }
                });
            });
    }

    fn show_inspector(&mut self, ui: &mut egui::Ui) {
        Inspector::show(
            ui,
            &mut self.selected_tab,
            &mut self.dock,
            &self.session,
        );
    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        ui.set_visuals(egui::Visuals::dark());

        let receipt_count = {
            let session = self.session.lock().unwrap();
            session.receipts.len()
        };

        ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new(
                    format!("Receipts: {}", receipt_count),
                ),
            );

            ui.with_layout(
                egui::Layout::right_to_left(egui::Align::Center),
                |ui| {
                    let clear_button = egui::Button::new(
                        egui::RichText::new("Clear").strong(),
                    )
                    .min_size(egui::vec2(100.0, 32.0));

                    if ui.add(clear_button).clicked() {
                        self.session.lock().unwrap().clear();
                    }
                },
            );
        });

        ui.separator();

        match self.dock {
            InspectorDock::Left => {
                egui::SidePanel::left("inspector_panel_left")
                    .resizable(true)
                    .default_width(ui.available_width() * 0.5)
                    .show_inside(ui, |ui| {
                        self.show_inspector(ui);
                    });

                self.show_preview(ui);
            }

            InspectorDock::Right => {
                egui::SidePanel::right("inspector_panel_right")
                    .resizable(true)
                    .default_width(ui.available_width() * 0.5)
                    .show_inside(ui, |ui| {
                        self.show_inspector(ui);
                    });

                self.show_preview(ui);
            }

            InspectorDock::Top => {
                egui::TopBottomPanel::top("inspector_panel_top")
                    .resizable(true)
                    .default_height(ui.available_height() * 0.5)
                    .show_inside(ui, |ui| {
                        self.show_inspector(ui);
                    });

                self.show_preview(ui);
            }

            InspectorDock::Bottom => {
                egui::TopBottomPanel::bottom("inspector_panel_bottom")
                    .resizable(true)
                    .default_height(ui.available_height() * 0.5)
                    .show_inside(ui, |ui| {
                        self.show_inspector(ui);
                    });

                self.show_preview(ui);
            }
        }
    }
}
