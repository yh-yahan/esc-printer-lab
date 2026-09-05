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
            Command::CarriageReturn => egui::Color32::from_rgb(180, 180, 180),
            Command::PrintAndFeedLines(_) | Command::PrintAndFeedDots(_) => egui::Color32::from_rgb(180, 180, 180),
            Command::SetDefaultLineSpacing | Command::SetLineSpacing(_) => egui::Color32::from_rgb(150, 180, 220),
            Command::Text(_) => egui::Color32::from_rgb(220, 220, 220),
            Command::Bold(_) => egui::Color32::from_rgb(255, 190, 80),
            Command::Align(_) => egui::Color32::from_rgb(180, 130, 255),
            Command::Underline(_) => egui::Color32::from_rgb(255, 120, 180),
            Command::Cut(_) => egui::Color32::from_rgb(255, 100, 100),
            Command::CharSize(_) => egui::Color32::from_rgb(100, 220, 160),
            Command::RasterImage(_) => egui::Color32::from_rgb(80, 200, 220),
            Command::Qr(_) => egui::Color32::from_rgb(120, 160, 255),
            Command::Unknown(_) => egui::Color32::from_rgb(255, 140, 80),
        }
    }

    fn show_command_docs(ui: &mut egui::Ui, command: &Command) {
        let spec = command.spec();

        ui.strong(spec.name);
        ui.monospace(format!("{}    {}", spec.mnemonic, spec.hex));
        ui.label(spec.summary);

        let params = command.param_lines();
        if !params.is_empty() {
            ui.add_space(4.0);
            for line in params {
                ui.monospace(line);
            }
        }

        if !spec.notes.is_empty() {
            ui.add_space(4.0);
            ui.weak(spec.notes);
        }

        if let Some(url) = spec.docs_url {
            ui.add_space(6.0);
            ui.hyperlink_to("Epson command reference", url);
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

                                response.on_hover_ui(|ui| {
                                    Self::show_command_docs(
                                        ui,
                                        &parsed.command,
                                    );
                                });
                            }
                        }
                    }

                    InspectorTab::Parser => {
                        if session.commands.is_empty() {
                            ui.label("No data yet");
                        } else {
                            for parsed in &session.commands {
                                let color = Self::command_color(&parsed.command);

                                let response = ui.selectable_label(
                                    self.command_hovered(&parsed.span),
                                    egui::RichText::new(format!(
                                        "{:?}",
                                        parsed.command
                                    ))
                                    .monospace()
                                    .color(color),
                                );

                                if response.hovered() {
                                    *self.next_hovered_span = Some(parsed.span.clone());
                                }

                                response.on_hover_ui(|ui| {
                                    Self::show_command_docs(
                                        ui,
                                        &parsed.command,
                                    );
                                });
                            }
                        }
                    }

                    InspectorTab::Hex => {
                        if session.raw.is_empty() {
                            ui.label("No data yet");
                        } else {
                            for (chunk_index, chunk) in
                                session.raw.chunks(16).enumerate()
                            {
                                let chunk_start = chunk_index * 16;

                                ui.horizontal(|ui| {
                                    for (index, byte) in
                                        chunk.iter().enumerate()
                                    {
                                        let byte_index = chunk_start + index;

                                        let parsed = session
                                            .commands
                                            .iter()
                                            .find(|command| {
                                                command.span.contains(
                                                    &byte_index,
                                                )
                                            });

                                        let color = parsed.map(|parsed| {
                                            Self::command_color(
                                                &parsed.command,
                                            )
                                        });

                                        let mut label = egui::RichText::new(format!(
                                                "{:02X}",
                                                byte
                                            ))
                                            .monospace();

                                        if let Some(color) = color {
                                            label = label.color(color);
                                        }

                                        let response = ui.selectable_label(
                                                self.byte_hovered(byte_index),
                                                label,
                                            );

                                        if response.hovered() {
                                            if let Some(parsed) = parsed {
                                                *self.next_hovered_span =
                                                    Some(
                                                        parsed.span.clone(),
                                                    );
                                            }
                                        }

                                        if let Some(parsed) = parsed {
                                            response.on_hover_ui(|ui| {
                                                Self::show_command_docs(
                                                    ui,
                                                    &parsed.command,
                                                );
                                            });
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
                                ui.monospace(
                                    Self::visualize_leading_spaces(
                                        &format!("{:#?}", receipt),
                                    ),
                                );
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
