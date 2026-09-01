use eframe::egui;
use std::sync::{Arc, Mutex};

use crate::shared::escpos_formatter::EscPosFormatter;
use crate::shared::print_session::PrintSession;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum InspectorTab {
    Receipt,
    Hex,
    Parser,
    Raw,
    EscPos,
}

pub struct InspectorViewer<'a> {
    pub session: &'a Arc<Mutex<PrintSession>>,
}

impl InspectorViewer<'_> {
    fn visualize_leading_spaces(s: &str) -> String {
        s.lines()
            .map(|line| {
                let leading = line.len() - line.trim_start().len();
                let visible_spaces = "·".repeat(leading);

                format!("{}{}", visible_spaces, line.trim_start())
            })
            .collect::<Vec<_>>()
            .join("\n")
    }
}

impl egui_dock::TabViewer for InspectorViewer<'_> {
    type Tab = InspectorTab;

    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        match tab {
            InspectorTab::EscPos => "ESC/POS".into(),
            InspectorTab::Receipt => "Receipt".into(),
            InspectorTab::Hex => "Hex".into(),
            InspectorTab::Parser => "Parser".into(),
            InspectorTab::Raw => "Raw".into(),
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {
        let session = self.session.lock().unwrap();

        egui::ScrollArea::vertical()
            .id_salt("inspector_content_scroll")
            .auto_shrink([false; 2])
            .show(ui, |ui| {
                match tab {
                    InspectorTab::EscPos => {
                        if session.commands.is_empty() {
                            ui.label("No data yet");
                        } else {
                            for command in &session.commands {
                                ui.monospace(
                                    EscPosFormatter::format_command(command),
                                );
                            }
                        }
                    }

                    InspectorTab::Receipt => {
                        if session.receipts.is_empty() {
                            ui.label("No data yet");
                        } else {
                            for receipt in &session.receipts {
                                ui.monospace(Self::visualize_leading_spaces(
                                    &format!("{:#?}", receipt),
                                ));
                            }
                        }
                    }

                    InspectorTab::Hex => {
                        if session.raw.is_empty() {
                            ui.label("No data yet");
                        } else {
                            for chunk in session.raw.chunks(16) {
                                let hex: String = chunk
                                    .iter()
                                    .map(|b| format!("{:02X} ", b))
                                    .collect();

                                ui.monospace(hex);
                            }
                        }
                    }

                    InspectorTab::Parser => {
                        if session.commands.is_empty() {
                            ui.label("No data yet");
                        } else {
                            for command in &session.commands {
                                ui.monospace(format!("{:?}", command));
                            }
                        }
                    }

                    InspectorTab::Raw => {
                        if session.raw.is_empty() {
                            ui.label("No data yet");
                        } else {
                            for chunk in session.raw.chunks(16) {
                                ui.monospace(format!("{:?}", chunk));
                            }
                        }
                    }
                }
            });
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
