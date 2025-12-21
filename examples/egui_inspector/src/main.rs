#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
#![allow(rustdoc::missing_crate_level_docs)] // it's an example

use std::collections::{HashMap, HashSet};
use std::sync::{Mutex, OnceLock};

use eframe::{egui, emath};
use quicksilver::reflections::*;
use quicksilver::*;
#[cfg(target_os = "linux")]
use winit::platform::x11::EventLoopBuilderExtX11;

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([800.0, 800.0]),
        #[cfg(target_os = "linux")]
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
#[repr(C)]
#[allow(unused)]
enum Occupation {
    None,
    DemonKing = 5,
    Layperson,
}

#[derive(Debug, Quicksilver)]
#[allow(unused)]
enum Ideology {
    None,
    CatPerson(usize),
    DogPerson(usize),
    CatDogPerson { cats: usize, dogs: usize },
}

#[derive(Debug, Quicksilver)]
struct Person {
    name: String,
    ideology: Ideology,
    job: Occupation,
    alive: bool,
    age: u32,
    houses: Vec<House>,
    pos: Pos,
    house_map: HashMap<Pos, House>,
    maybe_nickname: Option<String>,
    setting: HashSet<String>,
}

#[derive(Debug, Quicksilver, Clone)]
struct House {
    name: String,
    pos: Pos,
}

#[derive(Debug, Quicksilver, Clone, Hash, PartialEq, Eq)]
struct Pos {
    x: i32,
    y: i32,
}

impl Default for Person {
    fn default() -> Self {
        let houses = vec![
            House {
                name: "Home".to_string(),
                pos: Pos { x: 2, y: 3 },
            },
            House {
                name: "BeachHome".to_string(),
                pos: Pos { x: 23, y: 3 },
            },
        ];
        let house_map = houses
            .iter()
            .map(|it| (it.pos.clone(), it.clone()))
            .collect();
        Self {
            name: "Arthur".to_owned(),
            ideology: Ideology::CatDogPerson { cats: 5, dogs: 2 },
            age: 42,
            alive: true,
            houses,
            pos: Pos { x: 2, y: 3 },
            house_map,
            job: Occupation::None,
            maybe_nickname: Some("TODO".into()),
            setting: {
                let mut hs = HashSet::new();
                hs.insert("SciFi".into());
                hs.insert("Comedy".into());
                hs
            },
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
            draw_value(ui, &mut reflect(self));
            ui.separator();
            if ui.button("Print Debug").clicked() {
                dbg!(&self);
            }
        });
    }
}

fn draw_struct_reflection(ui: &mut egui::Ui, r: &mut StructReflection) {
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

fn draw_enum_reflection(ui: &mut egui::Ui, r: &mut RustEnumReflection) {
    ui.heading(format!("{}::{}", r.name, r.variant_name));
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

fn draw_struct_reflection_ref(ui: &mut egui::Ui, r: &StructReflection) {
    ui.heading(r.name);
    egui::Grid::new(next_id())
        .min_col_width(50.)
        .num_columns(2)
        .striped(true)
        .show(ui, |ui| {
            for field in &r.fields {
                ui.label(field.name);
                draw_value_ref(ui, &field.value);
                ui.end_row();
            }
        });
}

fn draw_enum_reflection_ref(ui: &mut egui::Ui, r: &RustEnumReflection) {
    ui.heading(format!("{}::{}", r.name, r.variant_name));
    egui::Grid::new(next_id())
        .min_col_width(50.)
        .num_columns(2)
        .striped(true)
        .show(ui, |ui| {
            for field in &r.fields {
                ui.label(field.name);
                draw_value_ref(ui, &field.value);
                ui.end_row();
            }
        });
}

fn draw_numeric<Num: emath::Numeric>(ui: &mut egui::Ui, value: &mut RefOrMut<Num>) {
    match value {
        RefOrMut::Ref(_val) => {
            unreachable!();
        }
        RefOrMut::Mut(val) => {
            ui.add(egui::DragValue::new(*val));
        }
    }
}

fn draw_value(ui: &mut egui::Ui, value: &mut ValueReflection) {
    match value {
        ValueReflection::I32(it) => {
            draw_numeric(ui, it);
        }
        ValueReflection::U32(it) => {
            draw_numeric(ui, it);
        }
        ValueReflection::F32(it) => {
            draw_numeric(ui, it);
        }
        ValueReflection::I64(it) => {
            draw_numeric(ui, it);
        }
        ValueReflection::U64(it) => {
            draw_numeric(ui, it);
        }
        ValueReflection::F64(it) => {
            draw_numeric(ui, it);
        }
        ValueReflection::ISize(it) => {
            draw_numeric(ui, it);
        }
        ValueReflection::USize(it) => {
            draw_numeric(ui, it);
        }
        ValueReflection::Bool(it) => {
            ui.checkbox(it, "");
        }
        ValueReflection::String(s) => {
            ui.text_edit_singleline(&mut **s);
        }
        ValueReflection::Struct(s) => {
            ui.vertical(|ui| {
                draw_struct_reflection(ui, s);
            });
        }
        ValueReflection::RustEnum(r) => {
            ui.vertical(|ui| {
                draw_enum_reflection(ui, r);
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
        ValueReflection::HashMap(hmreflection) => {
            egui::Grid::new(next_id())
                .min_col_width(50.)
                .num_columns(2)
                .striped(true)
                .show(ui, |ui| {
                    ui.label("Key");
                    ui.label("Value");
                    ui.end_row();
                    for mut element in hmreflection.get_elements() {
                        let [key, value] = element.fields.get_disjoint_mut([0, 1]).unwrap();
                        draw_value_ref(ui, &key.value);
                        draw_value(ui, &mut value.value);
                        ui.end_row();
                    }
                });
        }
        ValueReflection::CEnum(e) => {
            let name = e.variants.iter().find(|it| it.0 == *e.val).unwrap().1;
            egui::ComboBox::from_id_salt(next_id())
                .selected_text(format!("{name}"))
                .show_ui(ui, |ui| {
                    for (i, name) in e.variants {
                        ui.selectable_value(&mut *e.val, *i, *name);
                    }
                });
        }
        ValueReflection::Option(o) => {
            if let Some(ref mut inner) = o.get() {
                ui.vertical(|ui| {
                    ui.label("Some:");
                    draw_value(ui, inner);
                });
            } else {
                ui.label("None");
            }
        }
        ref outer @ ValueReflection::HashSet(_) => {
            draw_value_ref(ui, outer);
        }
    }
}

fn draw_value_ref(ui: &mut egui::Ui, value: &ValueReflection) {
    match value {
        ValueReflection::I32(it) => {
            draw_numeric_ref(ui, it);
        }
        ValueReflection::U32(it) => {
            draw_numeric_ref(ui, it);
        }
        ValueReflection::F32(it) => {
            draw_numeric_ref(ui, it);
        }
        ValueReflection::I64(it) => {
            draw_numeric_ref(ui, it);
        }
        ValueReflection::U64(it) => {
            draw_numeric_ref(ui, it);
        }
        ValueReflection::F64(it) => {
            draw_numeric_ref(ui, it);
        }
        ValueReflection::ISize(it) => {
            draw_numeric_ref(ui, it);
        }
        ValueReflection::USize(it) => {
            draw_numeric_ref(ui, it);
        }
        ValueReflection::Bool(it) => {
            let mut copy: bool = **it;
            ui.add_enabled(false, egui::Checkbox::new(&mut copy, ""));
        }
        ValueReflection::String(s) => {
            ui.label(&**s);
        }
        ValueReflection::Struct(s) => {
            ui.vertical(|ui| {
                draw_struct_reflection_ref(ui, s);
            });
        }
        ValueReflection::RustEnum(r) => {
            ui.vertical(|ui| {
                draw_enum_reflection_ref(ui, r);
            });
        }
        ValueReflection::Vec(vec) => {
            ui.vertical(|ui| {
                let len = vec.len();
                for i in 0..len {
                    draw_value_ref(ui, &vec.get_ref(i));
                }
            });
        }
        ValueReflection::HashSet(hsreflection) => {
            egui::Grid::new(next_id())
                .min_col_width(50.)
                .num_columns(2)
                .striped(true)
                .show(ui, |ui| {
                    for element in hsreflection.get_elements_ref() {
                        draw_value_ref(ui, &element);
                        ui.end_row();
                    }
                });
        }
        ValueReflection::HashMap(hmreflection) => {
            egui::Grid::new(next_id())
                .min_col_width(50.)
                .num_columns(2)
                .striped(true)
                .show(ui, |ui| {
                    for element in hmreflection.get_elements_ref() {
                        let key = &element.fields[0].value;
                        let value = &element.fields[1].value;
                        draw_value_ref(ui, key);
                        draw_value_ref(ui, value);
                        ui.end_row();
                    }
                });
        }
        ValueReflection::CEnum(e) => {
            let name = e.variants.iter().find(|it| it.0 == *e.val).unwrap().1;
            ui.label(&format!("{}", name));
        }
        ValueReflection::Option(o) => {
            if let Some(ref inner) = o.get_ref() {
                ui.vertical(|ui| {
                    ui.label("Some:");
                    draw_value_ref(ui, inner);
                });
            } else {
                ui.label("None");
            }
        }
    }
}

fn draw_numeric_ref<Num: emath::Numeric + std::fmt::Display>(
    ui: &mut egui::Ui,
    value: &RefOrMut<Num>,
) {
    match value {
        RefOrMut::Ref(val) => {
            ui.label(&format!("{}", *val));
        }
        RefOrMut::Mut(_val) => {
            unreachable!();
        }
    }
}
