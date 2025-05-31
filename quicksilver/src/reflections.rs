use crate::{Reflection, Struct, Type, map::HMReflection, vec::VecReflection};

#[repr(C)]
pub enum ValueReflection<'a> {
    I32(&'a mut i32),
    U32(&'a mut u32),
    F32(&'a mut f32),
    I64(&'a mut i64),
    U64(&'a mut u64),
    F64(&'a mut f64),
    ISize(&'a mut isize),
    USize(&'a mut usize),
    Bool(&'a mut bool),
    String(&'a mut String),
    Struct(Box<StructReflection<'a>>),
    Vec(Box<VecReflection<'a>>),
    HashMap(Box<HMReflection<'a>>),
}

#[repr(C)]
pub struct FieldReflection<'a> {
    pub name: &'a str,
    pub value: ValueReflection<'a>,
}

#[repr(C)]
pub struct StructReflection<'a> {
    pub name: &'a str,
    pub fields: Vec<FieldReflection<'a>>,
}

pub fn reflect<T: Reflection>(val: &mut T) -> StructReflection<'_> {
    unsafe { reflect_struct(val as *mut T as *mut u8, T::MIRROR) }
}

pub unsafe fn reflect_struct(base: *mut u8, mirror: &Struct) -> StructReflection<'_> {
    let mut fields: Vec<FieldReflection> = Vec::new();
    for field in mirror.fields {
        unsafe {
            let ptr = base.add(field.offset);
            fields.push(FieldReflection {
                name: field.name,
                value: reflect_value(ptr, &field.ty),
            });
        }
    }
    StructReflection {
        name: mirror.name,
        fields,
    }
}

pub unsafe fn reflect_value(ptr: *mut u8, ty: &Type) -> ValueReflection {
    match ty {
        Type::I32 => {
            let value = unsafe { &mut *(ptr as *mut i32) };
            ValueReflection::I32(value)
        }
        Type::U32 => {
            let value = unsafe { &mut *(ptr as *mut u32) };
            ValueReflection::U32(value)
        }
        Type::F32 => {
            let value = unsafe { &mut *(ptr as *mut f32) };
            ValueReflection::F32(value)
        }
        Type::I64 => {
            let value = unsafe { &mut *(ptr as *mut i64) };
            ValueReflection::I64(value)
        }
        Type::U64 => {
            let value = unsafe { &mut *(ptr as *mut u64) };
            ValueReflection::U64(value)
        }
        Type::F64 => {
            let value = unsafe { &mut *(ptr as *mut f64) };
            ValueReflection::F64(value)
        }
        Type::ISize => {
            let value = unsafe { &mut *(ptr as *mut isize) };
            ValueReflection::ISize(value)
        }
        Type::USize => {
            let value = unsafe { &mut *(ptr as *mut usize) };
            ValueReflection::USize(value)
        }
        Type::Bool => {
            let value = unsafe { &mut *(ptr as *mut bool) };
            ValueReflection::Bool(value)
        }
        Type::String => {
            let value = unsafe { &mut *(ptr as *mut String) };
            ValueReflection::String(value)
        }
        Type::Struct(s) => ValueReflection::Struct(Box::new(unsafe { reflect_struct(ptr, s) })),
        Type::Vec(v) => ValueReflection::Vec(Box::new(VecReflection {
            element: v.element,
            ptr,
            vtable: &v.vtable,
            _phantom: std::marker::PhantomData,
        })),
        Type::HashMap(hm) => ValueReflection::HashMap(Box::new(HMReflection {
            key: hm.key,
            value: hm.value,
            ptr,
            vtable: &hm.vtable,
            _phantom: std::marker::PhantomData,
        })),
    }
}
