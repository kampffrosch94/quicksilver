#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
#![allow(rustdoc::missing_crate_level_docs)] // it's an example

use std::sync::{Mutex, OnceLock};

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
    houses: Vec<House>,
    pos: Pos,
}

#[derive(Debug, Quicksilver)]
struct House {
    name: String,
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
            houses: vec![
                House {
                    name: "Home".to_string(),
                    pos: Pos { x: 2, y: 3 },
                },
                House {
                    name: "BeachHome".to_string(),
                    pos: Pos { x: 23, y: 3 },
                },
            ],
            pos: Pos { x: 2, y: 3 },
        }
    }
}

// we use this for grid ui ids
fn counter() -> &'static Mutex<usize> {
    static COUNTER: OnceLock<Mutex<usize>> = OnceLock::new();
    COUNTER.get_or_init(|| Mutex::new(0))
}

fn next_id() -> usize {
    let mut guard = counter().lock().unwrap();
    *guard += 1;
    *guard
}

fn reset_id() {
    let mut guard = counter().lock().unwrap();
    *guard = 0;
}

impl eframe::App for Person {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        reset_id();
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
    egui::Grid::new(next_id())
        .min_col_width(50.)
        .num_columns(2)
        .striped(true)
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
        ValueReflection::Vec(vec) => {
            ui.vertical(|ui| {
                let len = vec.len();
                for i in 0..len {
                    draw_value(ui, &mut vec.get(i));
                }
            });
        }
    }
}
