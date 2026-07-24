//! The eframe shell. Thin and human-validated (not unit-tested): it holds the
//! derived read-only `Snarl` and the cached generated Rust, and lays out the
//! top menu, the canvas, and the live "Generated Rust" panel per the spec.

use eframe::egui;
use egui_snarl::ui::{PinInfo, SnarlPin, SnarlStyle, SnarlViewer};
use egui_snarl::{InPin, OutPin, Snarl};

use crate::codegen::generate_source;
use crate::seed::seed_graph;
use crate::view::{to_snarl, InputRow, NodeView, OutputRow};

/// The whole editor: seed graph -> read-only `Snarl` + cached generated source.
/// The `Graph` is the source of truth; here it is consumed once at startup since
/// the skeleton is read-only (no canvas -> model adapter yet, per ADR-0009).
pub struct EditorApp {
    snarl: Snarl<NodeView>,
    generated: String,
    style: SnarlStyle,
}

impl Default for EditorApp {
    fn default() -> Self {
        let graph = seed_graph();
        let snarl = to_snarl(&graph);
        // generate_source never panics; on error it returns the message string,
        // which the panel shows verbatim.
        let generated = generate_source(&graph).unwrap_or_else(|e| e);
        Self {
            snarl,
            generated,
            style: SnarlStyle::new(),
        }
    }
}

impl eframe::App for EditorApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::Panel::top("menu_bar").show(ui, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                ui.menu_button("File", |_ui| {});
                ui.menu_button("Help", |_ui| {});
            });
        });

        egui::Panel::right("generated_rust")
            .default_size(360.0)
            .show(ui, |ui| {
                ui.heading("Generated Rust");
                egui::ScrollArea::vertical().show(ui, |ui| {
                    // Read-only display: interactive(false) discards edits, so a
                    // per-frame clone as the buffer is fine for the skeleton.
                    let mut buf = self.generated.clone();
                    ui.add(
                        egui::TextEdit::multiline(&mut buf)
                            .font(egui::TextStyle::Monospace)
                            .desired_width(f32::INFINITY)
                            .interactive(false),
                    );
                });
            });

        egui::CentralPanel::default().show(ui, |ui| {
            self.snarl
                .show(&mut SkeletonViewer, &self.style, "vr_canvas", ui);
        });
    }
}

/// Read-only viewer: draws titles and pins from `NodeView`. All mutation hooks
/// are no-ops so the canvas cannot be edited (the model stays the source of
/// truth; editing arrives in a later phase).
struct SkeletonViewer;

impl SnarlViewer<NodeView> for SkeletonViewer {
    fn title(&mut self, node: &NodeView) -> String {
        node.title.clone()
    }

    fn inputs(&mut self, node: &NodeView) -> usize {
        node.inputs.len()
    }

    fn outputs(&mut self, node: &NodeView) -> usize {
        node.outputs.len()
    }

    fn show_input(
        &mut self,
        pin: &InPin,
        ui: &mut egui::Ui,
        snarl: &mut Snarl<NodeView>,
    ) -> impl SnarlPin + 'static {
        // White triangle = execution pin (Blueprint style); circle = data pin.
        match &snarl[pin.id.node].inputs[pin.id.input] {
            InputRow::Exec => PinInfo::triangle().with_fill(egui::Color32::WHITE),
            InputRow::Wired { label } => {
                ui.label(label);
                PinInfo::circle()
            }
            InputRow::Inline { text } => {
                ui.label(egui::RichText::new(text).monospace());
                PinInfo::circle()
            }
        }
    }

    fn show_output(
        &mut self,
        pin: &OutPin,
        _ui: &mut egui::Ui,
        snarl: &mut Snarl<NodeView>,
    ) -> impl SnarlPin + 'static {
        match &snarl[pin.id.node].outputs[pin.id.output] {
            OutputRow::Exec => PinInfo::triangle().with_fill(egui::Color32::WHITE),
            OutputRow::Data => PinInfo::circle(),
        }
    }

    // Read-only: refuse every mutation.
    fn connect(&mut self, _from: &OutPin, _to: &InPin, _snarl: &mut Snarl<NodeView>) {}
    fn disconnect(&mut self, _from: &OutPin, _to: &InPin, _snarl: &mut Snarl<NodeView>) {}
}

/// Boot the eframe window hosting the editor.
pub fn run() -> eframe::Result<()> {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "Visual Rust",
        options,
        Box::new(|_cc| Ok(Box::new(EditorApp::default()))),
    )
}
