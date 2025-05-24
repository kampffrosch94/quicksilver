use std::mem::MaybeUninit;
mod parser;

use parser::JsonWalker;

use crate::{FieldTypeReflection, Reflection, Struct, StructReflection};

impl<'a> StructReflection<'a> {
    pub fn to_json_string(&self) -> String {
        let mut json_parts = Vec::new();
        for field in &self.fields {
            let field_name = format!("\"{}\"", field.name);
            let field_value = match &field.ty {
                FieldTypeReflection::I32(val) => format!("{}", **val),
                FieldTypeReflection::U32(val) => format!("{}", **val),
                FieldTypeReflection::F32(val) => format!("{}", **val),
                FieldTypeReflection::String(val) => {
                    let escaped_val = val.replace(r"\", r"\\").replace(r#"""#, r#"\""#);
                    format!("\"{}\"", escaped_val)
                }
                FieldTypeReflection::Struct(s_ref) => s_ref.to_json_string(),
            };
            json_parts.push(format!("{}:{}", field_name, field_value));
        }
        format!("{{{}}}", json_parts.join(","))
    }
}

pub fn from_json<T: Reflection>(s: &str) -> T {
    let mirror = T::MIRROR;
    // let layout = Layout::from_size_align(mirror.size, mirror.align).expect("Can't create layout.");

    let mut result: MaybeUninit<T> = MaybeUninit::uninit();
    let ptr = result.as_mut_ptr();
    unsafe {
        from_json_inner(s, ptr as *mut u8, mirror);
        result.assume_init()
    }
}

unsafe fn from_json_inner(s: &str, base: *mut u8, mirror: &Struct) {
    let mut walker = JsonWalker {
        chars: s.chars(),
        buffer: String::new(),
    };
    walker.consume_char('{');
    for field in mirror.fields {
        walker.consume_field(field.name);
        match field.ty {
            crate::Type::I32 => unsafe {
                let ptr = base.add(field.offset) as *mut i32;
                let val = walker.consume_i32();
                ptr.write(val);
            },
            crate::Type::U32 => unsafe {
                let ptr = base.add(field.offset) as *mut u32;
                let val = walker.consume_u32();
                ptr.write(val);
            },
            crate::Type::F32 => unsafe {
                let ptr = base.add(field.offset) as *mut f32;
                let val = walker.consume_f32();
                ptr.write(val);
            },
            crate::Type::String => unsafe {
                let ptr = base.add(field.offset) as *mut String;
                let val = walker.consume_string();
                ptr.write(val);
            },
            crate::Type::Struct(inner_mirror) => unsafe {
                let inner_ptr = base.add(field.offset);
                let inner_s = walker.chars.as_str();
                from_json_inner(inner_s, inner_ptr, inner_mirror);
            },
            crate::Type::Vec(_) => {
                todo!()
            }
        }
        walker.consume_either('}', ',');
    }
}

#[cfg(test)]
mod test {
    use std::mem;

    use crate::*;
    #[derive(Debug)]
    struct Point {
        x: i32,
        y: i32,
    }

    impl Reflection for Point {
        const MIRROR: &'static Struct = &Struct {
            size: size_of::<Self>(),
            align: size_of::<Self>(),
            fields: &[
                Field {
                    name: "x",
                    ty: Type::I32,
                    offset: mem::offset_of!(Point, x),
                },
                Field {
                    name: "y",
                    ty: Type::I32,
                    offset: mem::offset_of!(Point, y),
                },
            ],
        };
    }
    #[derive(Debug)]
    struct MyData {
        id: u32,
        name: String,
        value: f32,
        location: Point,
        is_active: i32, // Using i32 to demonstrate another integer type
    }

    impl Reflection for MyData {
        const MIRROR: &'static Struct = &Struct {
            size: size_of::<Self>(),
            align: size_of::<Self>(),
            fields: &[
                Field {
                    name: "id",
                    ty: Type::U32,
                    offset: mem::offset_of!(MyData, id),
                },
                Field {
                    name: "name",
                    ty: Type::String,
                    offset: mem::offset_of!(MyData, name),
                },
                Field {
                    name: "value",
                    ty: Type::F32,
                    offset: mem::offset_of!(MyData, value),
                },
                Field {
                    name: "location",
                    ty: Type::Struct(Point::MIRROR),
                    offset: mem::offset_of!(MyData, location),
                },
                Field {
                    name: "is_active",
                    ty: Type::I32,
                    offset: mem::offset_of!(MyData, is_active),
                },
            ],
        };
    }

    #[test]
    fn test_json_serialization() {
        let mut my_data = MyData {
            id: 789,
            name: "Another \"Test\" String with \\backslashes\\".to_string(),
            value: 123.45,
            location: Point { x: -5, y: 30 },
            is_active: 1,
        };

        let reflected_data = reflect(&mut my_data);
        let json_string = reflected_data.to_json_string();

        let expected_json = r#"{"id":789,"name":"Another \"Test\" String with \\backslashes\\","value":123.45,"location":{"x":-5,"y":30},"is_active":1}"#;

        assert_eq!(json_string, expected_json);
    }
}
