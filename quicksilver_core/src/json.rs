use std::mem::MaybeUninit;
mod parser;

use parser::JsonWalker;

use crate::{Reflection, Struct, StructReflection, Type, ValueReflection};

impl<'a> StructReflection<'a> {
    pub fn to_json_string(&mut self) -> String {
        let mut json_parts = Vec::new();
        for field in &mut self.fields {
            let field_name = format!("\"{}\"", field.name);
            let field_value = value_to_json(&mut field.value);
            json_parts.push(format!("{}:{}", field_name, field_value));
        }
        format!("{{{}}}", json_parts.join(","))
    }
}

pub fn value_to_json(vr: &mut ValueReflection) -> String {
    match vr {
        ValueReflection::I32(val) => format!("{}", **val),
        ValueReflection::U32(val) => format!("{}", **val),
        ValueReflection::F32(val) => format!("{}", **val),
        ValueReflection::String(val) => {
            let escaped_val = val.replace(r"\", r"\\").replace(r#"""#, r#"\""#);
            format!("\"{}\"", escaped_val)
        }
        ValueReflection::Struct(s_ref) => s_ref.to_json_string(),
        ValueReflection::Vec(vec_reflection) => {
            let mut ret = "[".to_string();
            let len = vec_reflection.len();
            let mut first = true;
            for i in 0..len {
                if !first {
                    ret.push(',');
                }
                ret.push_str(&value_to_json(&mut vec_reflection.get(i)));
                first = false;
            }
            ret.push(']');
            ret
        }
    }
}

pub fn from_json<T: Reflection>(s: &str) -> T {
    let mirror = T::MIRROR;
    let mut result: MaybeUninit<T> = MaybeUninit::uninit();
    let ptr = result.as_mut_ptr();
    let mut walker = JsonWalker {
        chars: s.chars(),
        buffer: String::new(),
    };
    unsafe {
        from_json_inner(&mut walker, ptr as *mut u8, mirror);
        result.assume_init()
    }
}

unsafe fn from_json_inner(walker: &mut JsonWalker, base: *mut u8, mirror: &Struct) {
    walker.consume_char('{');
    for field in mirror.fields {
        dbg!(field);
        dbg!(walker.chars.as_str());
        walker.consume_field(field.name);
        unsafe { deserialize_field(walker, base.add(field.offset), &field.ty) };
        walker.consume_maybe(',');
    }
    walker.consume_char('}');
}

unsafe fn deserialize_field(walker: &mut JsonWalker, base: *mut u8, ty: &Type) {
    match ty {
        Type::I32 => unsafe {
            let ptr = base as *mut i32;
            let val = walker.consume_i32();
            ptr.write(val);
        },
        Type::U32 => unsafe {
            let ptr = base as *mut u32;
            let val = walker.consume_u32();
            ptr.write(val);
        },
        Type::F32 => unsafe {
            let ptr = base as *mut f32;
            let val = walker.consume_f32();
            ptr.write(val);
        },
        Type::String => unsafe {
            let ptr = base as *mut String;
            let val = walker.consume_string();
            ptr.write(val);
        },
        Type::Struct(inner_mirror) => unsafe {
            from_json_inner(walker, base, inner_mirror);
        },
        Type::Vec(v) => unsafe {
            walker.consume_char('[');
            let mut cap = 4;
            let mut first = (v.vtable.new_at)(base, cap);
            let mut len = 0;
            let stride = v.element.layout().align();

            let mut is_first = true;
            while !walker.chars.as_str().starts_with(']') {
                if !is_first {
                    walker.consume_char(',');
                }
                is_first = false;

                debug_assert!(len <= cap); // sanity check
                if len == cap {
                    let extra = 8;
                    first = (v.vtable.reserve)(base, extra);
                    cap += extra;
                }

                let ptr = first.add(stride * len);
                deserialize_field(walker, ptr, v.element);
                len += 1;
            }
            (v.vtable.set_len)(base, len);
            walker.consume_char(']');
        },
    }
}

#[cfg(test)]
mod test {
    use std::mem;

    use crate::{json::from_json, vec::VecVtableCreator, *};
    #[derive(Debug, PartialEq)]
    struct Point {
        x: i32,
        y: i32,
    }

    impl Reflection for Point {
        const MIRROR: &'static Struct = &Struct {
            name: "Point",
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
    #[derive(Debug, PartialEq)]
    struct MyData {
        id: u32,
        name: String,
        value: f32,
        location: Point,
        is_active: i32, // Using i32 to demonstrate another integer type
    }

    impl Reflection for MyData {
        const MIRROR: &'static Struct = &Struct {
            name: "MyData",
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

        let mut reflected_data = reflect(&mut my_data);
        let json_string = reflected_data.to_json_string();

        let expected_json = r#"{"id":789,"name":"Another \"Test\" String with \\backslashes\\","value":123.45,"location":{"x":-5,"y":30},"is_active":1}"#;

        assert_eq!(json_string, expected_json);

        let deserialized = from_json::<MyData>(&json_string);
        assert_eq!(my_data, deserialized);
    }

    #[derive(Debug, PartialEq)]
    struct VecHolder {
        name: String,
        age: i32,
        values: Vec<i32>,
    }

    impl Reflection for VecHolder {
        const MIRROR: &'static Struct = &Struct {
            name: "VecHolder",
            size: size_of::<Self>(),
            align: size_of::<Self>(),
            fields: &[
                Field {
                    name: "name",
                    ty: Type::String,
                    offset: mem::offset_of!(Self, name),
                },
                Field {
                    name: "age",
                    ty: Type::I32,
                    offset: mem::offset_of!(Self, age),
                },
                Field {
                    name: "values",
                    ty: Type::Vec(VecType {
                        element: &Type::I32,
                        vtable: VecVtableCreator::<i32>::VTABLE,
                    }),
                    offset: mem::offset_of!(Self, values),
                },
            ],
        };
    }

    #[test]
    fn vec_roundtrip() {
        let mut val = VecHolder {
            name: "Kampffrosch".to_string(),
            age: 30,
            values: vec![1, 2, 3, 4, 5],
        };
        let s = reflect(&mut val).to_json_string();
        let val2 = from_json::<VecHolder>(&s);
        assert_eq!(val, val2);
    }
}
