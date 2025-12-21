use std::mem::MaybeUninit;
mod parser;

use parser::{JsonWalker, peek};

use crate::{
    Quicksilver, Struct, Type,
    reflections::{StructReflection, ValueReflection},
};

impl<'a> StructReflection<'a> {
    pub fn to_json(&self) -> String {
        let mut json_parts = Vec::new();
        for field in &self.fields {
            let field_name = format!("\"{}\"", field.name);
            let field_value = value_to_json(&field.value);
            json_parts.push(format!("{}:{}", field_name, field_value));
        }
        format!("{{{}}}", json_parts.join(","))
    }
}

impl<'a> ValueReflection<'a> {
    pub fn to_json(&self) -> String {
        value_to_json(self)
    }
}

pub fn value_to_json(vr: &ValueReflection) -> String {
    match vr {
        ValueReflection::I32(val) => format!("{}", **val),
        ValueReflection::CEnum(cenum_reflection) => format!("{}", *cenum_reflection.val),
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
        ValueReflection::Struct(s_ref) => s_ref.to_json(),
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
        ValueReflection::HashSet(hsreflection) => {
            if hsreflection.skip {
                "[]".to_string()
            } else {
                let vec_reflection = &mut hsreflection.get_elements_ref();
                let mut ret = "[".to_string();
                let mut first = true;
                for elem in vec_reflection {
                    if !first {
                        ret.push(',');
                    }
                    ret.push_str(&elem.to_json());
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
                    ret.push_str(&elem.to_json());
                    first = false;
                }
                ret.push(']');
                ret
            }
        }
        ValueReflection::Option(o_reflection) => {
            if o_reflection.skip {
                "[]".to_string()
            } else {
                let mut ret = "[".to_string();
                if let Some(inner) = o_reflection.get_ref() {
                    ret.push_str(&inner.to_json());
                }
                ret.push(']');
                ret
            }
        }
        ValueReflection::RustEnum(renum) => {
            let mut json_parts = Vec::new();
            json_parts.push(format!(
                r#""{}":"{}""#,
                "__enum_variant", renum.variant_name
            ));
            for field in &renum.fields {
                let field_name = format!("\"{}\"", field.name);
                let field_value = value_to_json(&field.value);
                json_parts.push(format!("{}:{}", field_name, field_value));
            }
            format!("{{{}}}", json_parts.join(","))
        }
        ValueReflection::Box(box_reflection) => value_to_json(&box_reflection.inner),
    }
}

pub fn from_json<T: Quicksilver>(s: &str) -> T {
    let mut result: MaybeUninit<T> = MaybeUninit::uninit();
    let ptr = result.as_mut_ptr();
    let walker = &mut JsonWalker {
        chars: s.chars(),
        buffer: String::new(),
    };
    unsafe {
        deserialize_field(walker, ptr as *mut u8, &T::MIRROR);
        result.assume_init()
    }
}

unsafe fn deserialize_struct(walker: &mut JsonWalker, base: *mut u8, mirror: &Struct) {
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
        Type::CEnum(cenum) => unsafe {
            let ptr = base as *mut i32;
            let val: i32 = walker.consume_int();
            assert!(cenum.variants.iter().any(|it| it.0 == val));
            debug_assert_eq!(cenum.size, size_of::<i32>());
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
                    cap = len + extra;
                    // because we don't set len beyond 0 until we are done
                    // reserve needs the capacity we want as parameter
                    first = (v.vtable.reserve)(base, cap);
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
                walker.consume_field("value");
                deserialize_field(walker, value, hm.value);

                walker.consume_char('}');
                (hm.vtable.fill_with)(base, key, value);
            }
            walker.consume_char(']');
        },
        Type::HashSet(hs) => unsafe {
            (hs.vtable.new_at)(base);
            walker.consume_char('[');
            while peek(&walker.chars) != ']' {
                walker.consume_maybe(',');
                let element = std::alloc::alloc(hs.element.layout());
                deserialize_field(walker, element, hs.element);
                (hs.vtable.fill_with)(base, element);
            }
            walker.consume_char(']');
        },
        Type::Option(o) => unsafe {
            (o.vtable.new_at)(base);
            walker.consume_char('[');

            if peek(&walker.chars) != ']' {
                let element = std::alloc::alloc(o.element.layout());
                deserialize_field(walker, element, o.element);
                (o.vtable.set)(base, Some(element));
            }
            walker.consume_char(']');
        },
        Type::RustEnum(mirror) => {
            walker.consume_char('{');
            // figure out which variant we a derializing
            walker.consume_field("__enum_variant");
            let name = walker.consume_string();
            let (index, variant) = mirror
                .variants
                .iter()
                .enumerate()
                .find(|(_, val)| val.name == name)
                .unwrap_or_else(|| panic!("Can't find enum variant {name} in {}", mirror.name));
            walker.consume_maybe(',');

            let mut field_ptrs = Vec::new();
            for (field_name, ty) in variant.fields {
                walker.consume_field(field_name);
                let ptr = unsafe { std::alloc::alloc(ty.layout()) };
                field_ptrs.push(ptr); // TODO protect against memory leaks on panic/error
                unsafe { deserialize_field(walker, ptr, ty) };
                walker.consume_maybe(',');
            }
            walker.consume_char('}');

            unsafe { (mirror.write)(base, index, &field_ptrs) }
        }
        Type::Box(box_type) => unsafe {
            let inner_space = std::alloc::alloc(box_type.inner.layout());
            deserialize_field(walker, inner_space, box_type.inner);
            (box_type.box_up)(base, inner_space)
        },
    }
}
