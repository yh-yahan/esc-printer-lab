use eframe::egui;
use std::sync::mpsc::Receiver;
use crate::receipt::receipt::Receipt;
use crate::ui::receipt_view::render_receipt;

pub struct MainWindow {
    rx: Receiver<Receipt>,
    receipts: Vec<Receipt>,
}

impl MainWindow {
    pub fn new(rx: Receiver<Receipt>) -> Self {
        Self {
            rx,
            receipts: Vec::new(),
        }
    }
}

impl eframe::App for MainWindow {
    fn logic(&mut self, _ctx: &egui::Context, _frame: &mut eframe::Frame) {
        while let Ok(receipt) = self.rx.try_recv() {
            self.receipts.push(receipt);
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
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
