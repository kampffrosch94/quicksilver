#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
#![allow(rustdoc::missing_crate_level_docs)] // it's an example

use eframe::egui;
use quicksilver::*;
use winit::platform::x11::EventLoopBuilderExtX11;

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([320.0, 240.0]),
        event_loop_builder: Some(Box::new(|el| {
            el.with_x11();
        })),
        ..Default::default()
    };
    eframe::run_native(
        "My egui App",
        options,
        Box::new(|_cc| Ok(Box::<MyApp>::default())),
    )
}

#[derive(Debug, Quicksilver)]
struct MyApp {
    name: String,
    age: u32,
    kids: u32,
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            name: "Arthur".to_owned(),
            age: 42,
            kids: 2,
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            draw_reflection(ui, &mut reflect(self));
            ui.separator();
            if ui.button("Print Debug").clicked() {
                dbg!(&self);
            }
        });
    }
}

fn draw_reflection(ui: &mut egui::Ui, r: &mut StructReflection) {
    ui.heading(r.name);
    egui::Grid::new("Grid")
        .min_col_width(50.)
        .show(ui, |ui| {
        for field in &mut r.fields {
            let field_label = ui.label(field.name);
            match &mut field.value {
                ValueReflection::I32(_) => todo!(),
                ValueReflection::U32(it) => {
                    ui.add(egui::DragValue::new(*it))
                        .labelled_by(field_label.id);
                }
                ValueReflection::F32(_) => todo!(),
                ValueReflection::String(s) => {
                    ui.text_edit_singleline(*s).labelled_by(field_label.id);
                }
                ValueReflection::Struct(_struct_reflection) => todo!(),
                ValueReflection::Vec(_vec_reflection) => todo!(),
            }
            ui.end_row();
        }
    });
    ui.columns(2, |columns| {
        for field in &mut r.fields {
            let field_label = columns[0].label(field.name);
            match &mut field.value {
                ValueReflection::I32(_) => todo!(),
                ValueReflection::U32(it) => {
                    columns[1]
                        .add(egui::DragValue::new(*it))
                        .labelled_by(field_label.id);
                }
                ValueReflection::F32(_) => todo!(),
                ValueReflection::String(s) => {
                    columns[1]
                        .text_edit_singleline(*s)
                        .labelled_by(field_label.id);
                }
                ValueReflection::Struct(_struct_reflection) => todo!(),
                ValueReflection::Vec(_vec_reflection) => todo!(),
            }
        }
    });
}
