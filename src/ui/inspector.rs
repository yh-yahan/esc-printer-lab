use eframe::egui;
use std::ops::Range;
use std::sync::{Arc, Mutex};

use crate::parser::command::Command;
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
    fn command_color(command: &Command) -> egui::Color32 {
        match command {
            Command::Initialize => egui::Color32::from_rgb(100, 180, 255),
            Command::LineFeed => egui::Color32::from_rgb(180, 180, 180),
            Command::Text(_) => egui::Color32::from_rgb(220, 220, 220),
            Command::Bold(_) => egui::Color32::from_rgb(255, 190, 80),
            Command::Align(_) => egui::Color32::from_rgb(180, 130, 255),
            Command::Underline(_) => egui::Color32::from_rgb(255, 120, 180),
            Command::Cut(_) => egui::Color32::from_rgb(255, 100, 100),
            Command::CharSize(_) => egui::Color32::from_rgb(100, 220, 160),
        }
    }

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
                                let text = EscPosFormatter::format_command(&parsed.command);
                                let color = Self::command_color(&parsed.command);

                                let response = ui.selectable_label(
                                    self.command_hovered(&parsed.span),
                                    egui::RichText::new(text)
                                        .monospace()
                                        .color(color),
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
