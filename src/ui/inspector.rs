use eframe::egui;
use std::sync::{Arc, Mutex};

use crate::shared::print_session::PrintSession;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum InspectorTab {
    Receipt,
    Hex,
    Parser,
    Raw,
    EscPos,
}

pub struct Inspector;

impl Inspector {
    pub fn tab_button(ui: &mut egui::Ui, current: &mut InspectorTab, tab: InspectorTab, title: &str) {
        if ui.selectable_label(*current == tab, title).clicked() {
            *current = tab;
        }
    }

    pub fn show(ui: &mut egui::Ui, selected_tab: &mut InspectorTab, session: &Arc<Mutex<PrintSession>>) {
        ui.horizontal(|ui| {
            Self::tab_button(ui, selected_tab, InspectorTab::EscPos, "ESC/POS");
            Self::tab_button(ui, selected_tab, InspectorTab::Receipt, "Receipt");
            Self::tab_button(ui, selected_tab, InspectorTab::Hex, "Hex");
            Self::tab_button(ui, selected_tab, InspectorTab::Parser, "Parser");
            Self::tab_button(ui, selected_tab, InspectorTab::Raw, "Raw");
        });

        ui.separator();

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

        egui::ScrollArea::vertical()
            .id_source("inspector_scroll")
            .auto_shrink([false; 2])
            .show(ui, |ui| {
                let session = session.lock().unwrap();

                match selected_tab {
                    InspectorTab::EscPos => {
                        if session.escpos_output.is_empty() {
                            ui.label("No data yet");
                        } else {
                            for line in &session.escpos_output {
                                ui.label(line);
                            }
                        }
                    }

                    InspectorTab::Receipt => {
                        if session.receipts.is_empty() {
                            ui.label("No data yet");
                        } else {
                            for receipt in &session.receipts {
                                ui.label(visualize_leading_spaces(&format!("{:#?}", receipt)));
                            }
                        }
                    }

                    InspectorTab::Parser => {
                        if session.parser_output.is_empty() {
                            ui.label("No parser data yet");
                        } else {
                            for line in &session.parser_output {
                                ui.label(line);
                            }
                        }
                    }

                    InspectorTab::Raw => {
                        if session.raw_chunks.is_empty() {
                            ui.label("No data yet");
                        } else {
                            for chunk in &session.raw_chunks {
                                ui.label(format!("{:?}", chunk));
                            }
                        }
                    }

                    InspectorTab::Hex => {
                        if session.raw_chunks.is_empty() {
                            ui.label("No data yet");
                        } else {
                            for chunk in &session.raw_chunks {
                                let hex: String = chunk
                                    .iter()
                                    .map(|b| format!("{:02X} ", b))
                                    .collect();

                                ui.monospace(hex);
                            }
                        }
                    }
                }
            });
    }
}
