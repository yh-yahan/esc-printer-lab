use eframe::egui;
use egui_dock::{DockArea, DockState, Style, TabViewer};
use std::ops::Range;
use std::sync::{Arc, Mutex};

use crate::receipt::receipt::Receipt;
use crate::shared::print_session::PrintSession;
use crate::ui::inspector::{InspectorTab, InspectorViewer};
use crate::ui::receipt_view::render_receipt;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum AppTab {
    ReceiptPreview,
    Inspector(InspectorTab),
}

impl AppTab {
    fn title(&self) -> &'static str {
        match self {
            Self::ReceiptPreview => "Receipt Preview",
            Self::Inspector(InspectorTab::EscPos) => "ESC/POS",
            Self::Inspector(InspectorTab::Receipt) => "Receipt",
            Self::Inspector(InspectorTab::Hex) => "Hex",
            Self::Inspector(InspectorTab::Parser) => "Parser",
        }
    }
}

pub struct App {
    session: Arc<Mutex<PrintSession>>,
    dock_state: DockState<AppTab>,
    hovered_span: Option<Range<usize>>,
}

impl App {
    pub fn new(session: Arc<Mutex<PrintSession>>) -> Self {
        let mut dock_state = DockState::new(vec![AppTab::ReceiptPreview]);

        let surface = dock_state.main_surface_mut();

        let [main, inspector] = surface.split_left(
            egui_dock::NodeIndex::root(),
            0.30,
            vec![
                AppTab::Inspector(InspectorTab::EscPos),
                AppTab::Inspector(InspectorTab::Receipt),
                AppTab::Inspector(InspectorTab::Hex),
                AppTab::Inspector(InspectorTab::Parser),
            ],
        );

        let _ = main;
        surface.set_focused_node(inspector);

        Self {
            session,
            dock_state,
            hovered_span: None,
        }
    }
}

struct AppViewer<'a> {
    session: &'a Arc<Mutex<PrintSession>>,
    hovered_span: Option<Range<usize>>,
    next_hovered_span: &'a mut Option<Range<usize>>,
}

impl TabViewer for AppViewer<'_> {
    type Tab = AppTab;

    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        tab.title().into()
    }

    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {
        match tab {
            AppTab::ReceiptPreview => {
                self.show_receipt_preview(ui);
            }

            AppTab::Inspector(inspector_tab) => {
                let mut viewer = InspectorViewer {
                    session: self.session,
                    hovered_span: self.hovered_span.clone(),
                    next_hovered_span: self.next_hovered_span,
                };

                viewer.ui(ui, inspector_tab);
            }
        }
    }

    fn id(&mut self, tab: &mut Self::Tab) -> egui::Id {
        egui::Id::new(*tab)
    }

    fn is_closeable(&self, _tab: &Self::Tab) -> bool {
        false
    }

    fn allowed_in_windows(&self, _tab: &mut Self::Tab) -> bool {
        true
    }
}

impl AppViewer<'_> {
    fn show_receipt_preview(&self, ui: &mut egui::Ui) {
        egui::ScrollArea::vertical()
            .id_salt("receipt_preview_scroll")
            .auto_shrink([false; 2])
            .show(ui, |ui| {
                let session = self.session.lock().unwrap();
                let mut items = Vec::new();

                for receipt in &session.receipts {
                    items.extend(receipt.items.iter().cloned());
                }
                items.extend(session.current.items.iter().cloned());

                ui.vertical_centered(|ui| {
                    render_receipt(ui, &Receipt { items });
                });
            });
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
            ui.label(egui::RichText::new(format!("Receipts: {}", receipt_count)));

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let clear_button = egui::Button::new(egui::RichText::new("Clear").strong())
                    .min_size(egui::vec2(100.0, 32.0));

                if ui.add(clear_button).clicked() {
                    self.session.lock().unwrap().clear();
                    self.hovered_span = None;
                }
            });
        });

        ui.separator();

        let mut next_hovered_span = None;

        {
            let mut viewer = AppViewer {
                session: &self.session,
                hovered_span: self.hovered_span.clone(),
                next_hovered_span: &mut next_hovered_span,
            };

            let style = Style::from_egui(ui.style());

            DockArea::new(&mut self.dock_state)
                .style(style)
                .show_inside(ui, &mut viewer);
        }

        if self.hovered_span != next_hovered_span {
            ui.ctx().request_repaint();
        }

        self.hovered_span = next_hovered_span;
    }
}
