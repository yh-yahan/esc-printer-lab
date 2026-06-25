use eframe::egui;
use std::sync::mpsc::Receiver;
use crate::receipt::receipt::Receipt;
use crate::ui::receipt_view::render_receipt;

pub struct App {
    rx: Receiver<Receipt>,
    receipts: Vec<Receipt>,
}

impl App {
    pub fn new(rx: Receiver<Receipt>) -> Self {
        Self {
            rx,
            receipts: Vec::new(),
        }
    }
}

impl eframe::App for App {
    fn logic(&mut self, _ctx: &egui::Context, _frame: &mut eframe::Frame) {
        while let Ok(receipt) = self.rx.try_recv() {
            self.receipts.push(receipt);
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new(format!("Receipts: {}", self.receipts.len()))
                    .color(egui::Color32::from_gray(230))
            );

            ui.with_layout(
                egui::Layout::right_to_left(egui::Align::Center),
                |ui| {
                    let clear_button = egui::Button::new(
                        egui::RichText::new("Clear")
                            .strong()
                    )
                    .min_size(egui::vec2(100.0, 32.0));

                    if ui.add(clear_button).clicked() {
                        self.receipts.clear();
                    }
                },
            );
        });

        ui.separator();
        
        egui::ScrollArea::vertical()
            .auto_shrink([false; 2])
            .show(ui, |ui| {
                if self.receipts.is_empty() {
                    let empty_receipt = Receipt { lines: vec![] };
                    render_receipt(ui, &empty_receipt);
                } else {
                    for receipt in &self.receipts {
                        render_receipt(ui, receipt);
                    }
                }
            });
    }
}
