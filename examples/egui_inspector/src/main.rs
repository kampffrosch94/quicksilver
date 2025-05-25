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
        Box::new(|_cc| Ok(Box::<Person>::default())),
    )
}

#[derive(Debug, Quicksilver)]
struct Person {
    name: String,
    age: u32,
    kids: u32,
    pos: Pos,
}

#[derive(Debug, Quicksilver)]
struct Pos {
    x: i32,
    y: i32,
}

impl Default for Person {
    fn default() -> Self {
        Self {
            name: "Arthur".to_owned(),
            age: 42,
            kids: 2,
            pos: Pos { x: 2, y: 3 },
        }
    }
}

impl eframe::App for Person {
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
        .num_columns(2)
        .show(ui, |ui| {
            for field in &mut r.fields {
                ui.label(field.name);
                draw_value(ui, &mut field.value);
                ui.end_row();
            }
        });
}

fn draw_value(ui: &mut egui::Ui, value: &mut ValueReflection) {
    match value {
        ValueReflection::I32(it) => {
            ui.add(egui::DragValue::new(*it));
        }
        ValueReflection::U32(it) => {
            ui.add(egui::DragValue::new(*it));
        }
        ValueReflection::F32(it) => {
            ui.add(egui::DragValue::new(*it));
        }
        ValueReflection::String(s) => {
            ui.text_edit_singleline(*s);
        }
        ValueReflection::Struct(s) => {
            ui.vertical(|ui| {
                draw_reflection(ui, s);
            });
        }
        ValueReflection::Vec(_vec_reflection) => todo!(),
    }
}
