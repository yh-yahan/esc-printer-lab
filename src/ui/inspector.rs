use eframe::egui;
use std::ops::Range;
use std::sync::{Arc, Mutex};

use crate::shared::escpos_formatter::EscPosFormatter;
use crate::shared::print_session::PrintSession;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum InspectorTab {
    Receipt,
    Hex,
    Parser,
    EscPos,
}

pub struct InspectorViewer<'a> {
    pub session: &'a Arc<Mutex<PrintSession>>,
    pub hovered_span: Option<Range<usize>>,
    pub next_hovered_span: &'a mut Option<Range<usize>>,
}

impl InspectorViewer<'_> {
    fn visualize_leading_spaces(s: &str) -> String {
        s.lines()
            .map(|line| {
                let leading = line.len() - line.trim_start().len();
                format!("{}{}", "·".repeat(leading), line.trim_start())
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn command_hovered(&self, span: &Range<usize>) -> bool {
        self.hovered_span.as_ref() == Some(span)
    }

    fn byte_hovered(&self, index: usize) -> bool {
        self.hovered_span
            .as_ref()
            .is_some_and(|span| span.contains(&index))
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
                            for parsed in &session.commands {
                                let response = ui.selectable_label(
                                    self.command_hovered(&parsed.span),
                                    EscPosFormatter::format_command(&parsed.command),
                                );

                                if response.hovered() {
                                    *self.next_hovered_span = Some(parsed.span.clone());
                                }
                            }
                        }
                    }

                    InspectorTab::Parser => {
                        if session.commands.is_empty() {
                            ui.label("No data yet");
                        } else {
                            for parsed in &session.commands {
                                let response = ui.selectable_label(
                                    self.command_hovered(&parsed.span),
                                    format!("{:?}", parsed.command),
                                );

                                if response.hovered() {
                                    *self.next_hovered_span = Some(parsed.span.clone());
                                }
                            }
                        }
                    }

                    InspectorTab::Hex => {
                        if session.raw.is_empty() {
                            ui.label("No data yet");
                        } else {
                            for (chunk_index, chunk) in session.raw.chunks(16).enumerate() {
                                let chunk_start = chunk_index * 16;

                                ui.horizontal(|ui| {
                                    for (index, byte) in chunk.iter().enumerate() {
                                        let byte_index = chunk_start + index;

                                        let response = ui.selectable_label(
                                            self.byte_hovered(byte_index),
                                            format!("{:02X}", byte),
                                        );

                                        if response.hovered() {
                                            if let Some(command) = session
                                                .commands
                                                .iter()
                                                .find(|command| command.span.contains(&byte_index))
                                            {
                                                *self.next_hovered_span =
                                                    Some(command.span.clone());
                                            }
                                        }
                                    }
                                });
                            }
                        }
                    }

                    InspectorTab::Receipt => {
                        if session.receipts.is_empty() {
                            ui.label("No data yet");
                        } else {
                            for receipt in &session.receipts {
                                ui.monospace(Self::visualize_leading_spaces(&format!(
                                    "{:#?}",
                                    receipt
                                )));
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
