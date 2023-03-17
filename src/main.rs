#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
#![allow(dead_code)]

use eframe::egui;
mod syntax_highlighting;

mod metrics;
use metrics::{Metric, process_js};

fn main() -> Result<(), eframe::Error> {
    // Log to stdout (if you run with `RUST_LOG=debug`).
    tracing_subscriber::fmt::init();

    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(1280.0, 720.0)),
        ..Default::default()
    };

    eframe::run_native(
        "My egui App",
        options,
        Box::new(|_cc| Box::<MyApp>::default()),
    )
}

#[derive(Default)]
struct MyApp {
    code: String,
    metric: Metric,
    graph_window: bool,
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.set_pixels_per_point(1.5);

        egui::SidePanel::left("left_panel")
            .resizable(false)
            .min_width(400.0)
            .show(ctx, |ui| {
                use egui_extras::{Column, TableBuilder};
                if ui.button("Toggle graph window").clicked() {
                    self.graph_window = !self.graph_window;
                }

                if self.graph_window {
                    egui::Window::new("Program graph").show(ctx, |ui| {
                        let _visuals = ui.style();
                    });
                }

                ui.push_id(0, |ui| {
                    TableBuilder::new(ui)
                        .striped(true)
                        .column(Column::initial(120.0))
                        .column(Column::remainder())
                        .header(10.0, |mut header| {
                            header.col(|ui| {
                                ui.heading("Property");
                            });
                            header.col(|ui| {
                                ui.heading("value");
                            });
                        })
                        .body(|mut body| {
                            for (param, value) in &self.metric.properties {
                                body.row(30.0, |mut row| {
                                    row.col(|ui| {
                                        ui.label(param);
                                    });
                                    row.col(|ui| {
                                        ui.label(value);
                                    });
                                });
                            }
                        });
                });
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            egui::widgets::global_dark_light_mode_buttons(ui);

            if ui.button("Compute").clicked() {
                self.metric = process_js(&self.code);
                self.metric.compute_properties();

                let mut props = String::new();
                for (p, v) in &self.metric.properties {
                    props.push_str(&format!("{p}, {v}\n"));
                }
                std::fs::write("properties.csv", props).unwrap();
            }

            let mut theme = syntax_highlighting::CodeTheme::from_memory(ui.ctx());
            ui.collapsing("Theme", |ui| {
                ui.group(|ui| {
                    theme.ui(ui);
                    theme.clone().store_in_memory(ui.ctx());
                });
            });

            let mut layouter = |ui: &egui::Ui, string: &str, wrap_width: f32| {
                let mut layout_job = syntax_highlighting::highlight(ui.ctx(), &theme, string, "js");
                layout_job.wrap.max_width = wrap_width;
                ui.fonts(|f| f.layout_job(layout_job))
            };

            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.add(
                    egui::TextEdit::multiline(&mut self.code)
                        .font(egui::TextStyle::Monospace) // for cursor height
                        .code_editor()
                        .desired_rows(10)
                        .lock_focus(true)
                        .desired_width(f32::INFINITY)
                        .layouter(&mut layouter),
                );
            });
        });
    }
}
