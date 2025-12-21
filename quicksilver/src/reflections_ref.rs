use crate::{
    Quicksilver, Struct, Type,
    map::HMReflection,
    option::OptionReflection,
    reflections::{CEnumReflection, FieldReflection, StructReflection, ValueReflection},
    set::HSReflection,
    vec::VecReflection,
};

pub fn reflect_ref<T: Quicksilver>(val: &T) -> ValueReflection<'_> {
    unsafe { reflect_value_ref(val as *const T as *const u8, &T::MIRROR) }
}

pub unsafe fn reflect_struct_ref(base: *const u8, mirror: &Struct) -> StructReflection<'_> {
    let mut fields: Vec<FieldReflection> = Vec::new();
    for field in mirror.fields {
        unsafe {
            let ptr = base.add(field.offset);
            fields.push(FieldReflection {
                name: field.name,
                value: reflect_value_ref(ptr, &field.ty),
            });
        }
    }
    StructReflection {
        name: mirror.name,
        fields,
    }
}

pub unsafe fn reflect_value_ref(ptr: *const u8, ty: &Type) -> ValueReflection<'_> {
    match ty {
        Type::I32 => {
            let value = unsafe { &*(ptr as *const i32) };
            ValueReflection::I32(value.into())
        }
        Type::U32 => {
            let value = unsafe { &*(ptr as *const u32) };
            ValueReflection::U32(value.into())
        }
        Type::F32 => {
            let value = unsafe { &*(ptr as *const f32) };
            ValueReflection::F32(value.into())
        }
        Type::I64 => {
            let value = unsafe { &*(ptr as *const i64) };
            ValueReflection::I64(value.into())
        }
        Type::U64 => {
            let value = unsafe { &*(ptr as *const u64) };
            ValueReflection::U64(value.into())
        }
        Type::F64 => {
            let value = unsafe { &*(ptr as *const f64) };
            ValueReflection::F64(value.into())
        }
        Type::ISize => {
            let value = unsafe { &*(ptr as *const isize) };
            ValueReflection::ISize(value.into())
        }
        Type::USize => {
            let value = unsafe { &*(ptr as *const usize) };
            ValueReflection::USize(value.into())
        }
        Type::Bool => {
            let value = unsafe { &*(ptr as *const bool) };
            ValueReflection::Bool(value.into())
        }
        Type::String => {
            let value = unsafe { &*(ptr as *const String) };
            ValueReflection::String(value.into())
        }
        Type::Struct(s) => ValueReflection::Struct(Box::new(unsafe { reflect_struct_ref(ptr, s) })),
        Type::CEnum(cenum) => {
            let value = unsafe { &*(ptr as *const i32) };
            ValueReflection::CEnum(Box::new(CEnumReflection {
                name: cenum.name,
                val: value.into(),
                variants: cenum.variants,
            }))
        }
        Type::Vec(v) => ValueReflection::Vec(Box::new(VecReflection {
            element: v.element,
            ptr: ptr as *mut u8,
            vtable: &v.vtable,
            skip: v.skip,
        })),
        Type::HashMap(hm) => ValueReflection::HashMap(Box::new(HMReflection {
            key: hm.key,
            value: hm.value,
            ptr: ptr as *mut u8,
            vtable: &hm.vtable,
            skip: hm.skip,
        })),
        Type::HashSet(hs) => ValueReflection::HashSet(Box::new(HSReflection {
            element: hs.element,
            ptr: ptr as *mut u8,
            vtable: &hs.vtable,
            skip: hs.skip,
        })),
        Type::Option(o) => ValueReflection::Option(Box::new(OptionReflection {
            element: o.element,
            ptr: ptr as *mut u8,
            vtable: &o.vtable,
            skip: o.skip,
        })),
        Type::RustEnum(renum) => ValueReflection::RustEnum(unsafe { (renum.reflect_ref)(ptr) }),
    }
}
