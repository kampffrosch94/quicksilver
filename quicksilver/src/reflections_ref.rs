use crate::{Reflection, Struct, Type, map::HMReflectionRef, vec::VecReflectionRef};

#[repr(C)]
pub enum ValueReflectionRef<'a> {
    I32(&'a i32),
    U32(&'a u32),
    F32(&'a f32),
    I64(&'a i64),
    U64(&'a u64),
    F64(&'a f64),
    ISize(&'a isize),
    USize(&'a usize),
    Bool(&'a bool),
    String(&'a String),
    Struct(Box<StructReflectionRef<'a>>),
    Vec(Box<VecReflectionRef<'a>>),
    HashMap(Box<HMReflectionRef<'a>>),
}

#[repr(C)]
pub struct FieldReflectionRef<'a> {
    pub name: &'a str,
    pub value: ValueReflectionRef<'a>,
}

#[repr(C)]
pub struct StructReflectionRef<'a> {
    pub name: &'a str,
    pub fields: Vec<FieldReflectionRef<'a>>,
}

pub fn reflect_ref<T: Reflection>(val: &T) -> StructReflectionRef<'_> {
    unsafe { reflect_struct_ref(val as *const T as *const u8, T::MIRROR) }
}

pub unsafe fn reflect_struct_ref(base: *const u8, mirror: &Struct) -> StructReflectionRef<'_> {
    let mut fields: Vec<FieldReflectionRef> = Vec::new();
    for field in mirror.fields {
        unsafe {
            let ptr = base.add(field.offset);
            fields.push(FieldReflectionRef {
                name: field.name,
                value: reflect_value_ref(ptr, &field.ty),
            });
        }
    }
    StructReflectionRef {
        name: mirror.name,
        fields,
    }
}

pub unsafe fn reflect_value_ref(ptr: *const u8, ty: &Type) -> ValueReflectionRef {
    match ty {
        Type::I32 => {
            let value = unsafe { &*(ptr as *const i32) };
            ValueReflectionRef::I32(value)
        }
        Type::U32 => {
            let value = unsafe { &*(ptr as *const u32) };
            ValueReflectionRef::U32(value)
        }
        Type::F32 => {
            let value = unsafe { &*(ptr as *const f32) };
            ValueReflectionRef::F32(value)
        }
        Type::I64 => {
            let value = unsafe { &*(ptr as *const i64) };
            ValueReflectionRef::I64(value)
        }
        Type::U64 => {
            let value = unsafe { &*(ptr as *const u64) };
            ValueReflectionRef::U64(value)
        }
        Type::F64 => {
            let value = unsafe { &*(ptr as *const f64) };
            ValueReflectionRef::F64(value)
        }
        Type::ISize => {
            let value = unsafe { &*(ptr as *const isize) };
            ValueReflectionRef::ISize(value)
        }
        Type::USize => {
            let value = unsafe { &*(ptr as *const usize) };
            ValueReflectionRef::USize(value)
        }
        Type::Bool => {
            let value = unsafe { &*(ptr as *const bool) };
            ValueReflectionRef::Bool(value)
        }
        Type::String => {
            let value = unsafe { &*(ptr as *const String) };
            ValueReflectionRef::String(value)
        }
        Type::Struct(s) => {
            ValueReflectionRef::Struct(Box::new(unsafe { reflect_struct_ref(ptr, s) }))
        }
        Type::Vec(v) => ValueReflectionRef::Vec(Box::new(VecReflectionRef {
            element: v.element,
            ptr: ptr as *mut u8,
            vtable: &v.vtable,
            _phantom: std::marker::PhantomData,
        })),
        Type::HashMap(hm) => ValueReflectionRef::HashMap(Box::new(HMReflectionRef {
            key: hm.key,
            value: hm.value,
            ptr: ptr as *mut u8,
            vtable: &hm.vtable,
            _phantom: std::marker::PhantomData,
        })),
    }
}
