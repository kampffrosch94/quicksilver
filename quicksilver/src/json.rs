use std::mem::MaybeUninit;
mod parser;

use parser::{JsonWalker, peek};

use crate::{
    Reflection, Struct, Type,
    reflections::{StructReflection, ValueReflection},
};

impl<'a> StructReflection<'a> {
    pub fn to_json_string(&self) -> String {
        let mut json_parts = Vec::new();
        for field in &self.fields {
            let field_name = format!("\"{}\"", field.name);
            let field_value = value_to_json(&field.value);
            json_parts.push(format!("{}:{}", field_name, field_value));
        }
        format!("{{{}}}", json_parts.join(","))
    }
}

pub fn value_to_json(vr: &ValueReflection) -> String {
    match vr {
        ValueReflection::I32(val) => format!("{}", **val),
        ValueReflection::U32(val) => format!("{}", **val),
        ValueReflection::F32(val) => format!("{}", **val),
        ValueReflection::I64(val) => format!("{}", **val),
        ValueReflection::U64(val) => format!("{}", **val),
        ValueReflection::F64(val) => format!("{}", **val),
        ValueReflection::ISize(val) => format!("{}", **val),
        ValueReflection::USize(val) => format!("{}", **val),
        ValueReflection::Bool(val) => format!("{}", **val),
        ValueReflection::String(val) => {
            let escaped_val = val.replace(r"\", r"\\").replace(r#"""#, r#"\""#);
            format!("\"{}\"", escaped_val)
        }
        ValueReflection::Struct(s_ref) => s_ref.to_json_string(),
        ValueReflection::Vec(vec_reflection) => {
            if vec_reflection.skip {
                "[]".to_string()
            } else {
                let mut ret = "[".to_string();
                let len = vec_reflection.len();
                let mut first = true;
                for i in 0..len {
                    if !first {
                        ret.push(',');
                    }
                    ret.push_str(&value_to_json(&vec_reflection.get_ref(i)));
                    first = false;
                }
                ret.push(']');
                ret
            }
        }
        ValueReflection::HashMap(hmreflection) => {
            if hmreflection.skip {
                "[]".to_string()
            } else {
                let vec_reflection = &mut hmreflection.get_elements_ref();
                let mut ret = "[".to_string();
                let mut first = true;
                for elem in vec_reflection {
                    if !first {
                        ret.push(',');
                    }
                    ret.push_str(&elem.to_json_string());
                    first = false;
                }
                ret.push(']');
                ret
            }
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
        deserialize_struct(&mut walker, ptr as *mut u8, mirror);
        result.assume_init()
    }
}

unsafe fn deserialize_struct(walker: &mut JsonWalker, base: *mut u8, mirror: &Struct) {
    println!("{}", walker.chars.as_str());
    walker.consume_char('{');
    for field in mirror.fields {
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
            let val = walker.consume_int();
            ptr.write(val);
        },
        Type::U32 => unsafe {
            let ptr = base as *mut u32;
            let val = walker.consume_int();
            ptr.write(val);
        },
        Type::F32 => unsafe {
            let ptr = base as *mut f32;
            let val = walker.consume_float();
            ptr.write(val);
        },
        Type::I64 => unsafe {
            let ptr = base as *mut i64;
            let val = walker.consume_int();
            ptr.write(val);
        },
        Type::U64 => unsafe {
            let ptr = base as *mut u64;
            let val = walker.consume_int();
            ptr.write(val);
        },
        Type::F64 => unsafe {
            let ptr = base as *mut f64;
            let val = walker.consume_int();
            ptr.write(val);
        },
        Type::ISize => unsafe {
            let ptr = base as *mut isize;
            let val = walker.consume_int();
            ptr.write(val);
        },
        Type::USize => unsafe {
            let ptr = base as *mut usize;
            let val = walker.consume_int();
            ptr.write(val);
        },
        Type::Bool => unsafe {
            let ptr = base as *mut bool;
            let val = walker.consume_bool();
            ptr.write(val);
        },
        Type::String => unsafe {
            let ptr = base as *mut String;
            let val = walker.consume_string();
            ptr.write(val);
        },
        Type::Struct(inner_mirror) => unsafe {
            deserialize_struct(walker, base, inner_mirror);
        },
        Type::Vec(v) => unsafe {
            walker.consume_char('[');
            let mut cap = 4;
            let mut first = (v.vtable.new_at)(base, cap);
            let mut len = 0;
            let stride = v.element.layout().size();

            let mut is_first = true;
            while peek(&walker.chars) != ']' {
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
        Type::HashMap(hm) => unsafe {
            (hm.vtable.new_at)(base);
            walker.consume_char('[');
            while peek(&walker.chars) != ']' {
                walker.consume_maybe(',');
                let key = std::alloc::alloc(hm.key.layout());
                let value = std::alloc::alloc(hm.value.layout());
                walker.consume_char('{');
                walker.consume_field("key");
                deserialize_field(walker, key, hm.key);
                walker.consume_char(',');
                println!("Key Ok");
                walker.consume_field("value");
                dbg!(hm);
                deserialize_field(walker, value, hm.value);

                println!("Value Ok");
                walker.consume_char('}');
                (hm.vtable.fill_with)(base, key, value);
            }
            walker.consume_char(']');
        },
    }
}

#[cfg(test)]
mod test {
    use crate::{json::from_json, reflections_ref::reflect_ref, *};
    #[derive(Debug, PartialEq, Quicksilver)]
    struct Point {
        x: i32,
        y: i32,
    }

    #[derive(Debug, PartialEq, Quicksilver)]
    struct MyData {
        id: u32,
        name: String,
        value: f32,
        location: Point,
        is_active: usize, // Using i32 to demonstrate another integer type
    }

    #[test]
    fn test_json_serialization() {
        let my_data = MyData {
            id: 789,
            name: "Another \"Test\" String with \\backslashes\\".to_string(),
            value: 123.45,
            location: Point { x: -5, y: 30 },
            is_active: 1,
        };

        let reflected_data = reflect_ref(&my_data);
        let json_string = reflected_data.to_json_string();

        let expected_json = r#"{"id":789,"name":"Another \"Test\" String with \\backslashes\\","value":123.45,"location":{"x":-5,"y":30},"is_active":1}"#;

        assert_eq!(json_string, expected_json);

        let deserialized = from_json::<MyData>(&json_string);
        assert_eq!(my_data, deserialized);
    }
}
