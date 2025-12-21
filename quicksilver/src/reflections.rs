use crate::option::OptionReflection;
use crate::set::HSReflection;
use crate::{Quicksilver, Struct, Type, map::HMReflection, vec::VecReflection};
use std::fmt::Debug;

use std::ops::Deref;
use std::ops::DerefMut;

#[repr(C)]
pub enum ValueReflection<'a> {
    I32(RefOrMut<'a, i32>),
    U32(RefOrMut<'a, u32>),
    F32(RefOrMut<'a, f32>),
    I64(RefOrMut<'a, i64>),
    U64(RefOrMut<'a, u64>),
    F64(RefOrMut<'a, f64>),
    ISize(RefOrMut<'a, isize>),
    USize(RefOrMut<'a, usize>),
    Bool(RefOrMut<'a, bool>),
    String(RefOrMut<'a, String>),
    Struct(Box<StructReflection<'a>>),
    CEnum(Box<CEnumReflection<'a>>),
    Vec(Box<VecReflection<'a>>),
    HashMap(Box<HMReflection<'a>>),
    HashSet(Box<HSReflection<'a>>),
    Option(Box<OptionReflection<'a>>),
    RustEnum(RustEnumReflection<'a>),
    Box(Box<BoxReflection<'a>>),
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

#[repr(C)]
pub struct BoxReflection<'a> {
    pub inner: ValueReflection<'a>,
}

#[repr(C)]
pub struct CEnumReflection<'a> {
    pub name: &'a str,
    pub val: RefOrMut<'a, i32>,
    pub variants: &'a [(i32, &'a str)],
}

#[repr(C)]
pub struct RustEnumReflection<'a> {
    pub name: &'a str,
    pub variant_name: &'a str,
    pub variant_idx: usize,
    pub ty: &'a Type,
    pub fields: Vec<FieldReflection<'a>>,
}

pub fn reflect<T: Quicksilver>(val: &mut T) -> ValueReflection<'_> {
    unsafe { reflect_value(val as *mut T as *mut u8, &T::MIRROR) }
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

pub unsafe fn reflect_value(ptr: *mut u8, ty: &Type) -> ValueReflection<'_> {
    match ty {
        Type::I32 => {
            let value = unsafe { &mut *(ptr as *mut i32) };
            ValueReflection::I32(value.into())
        }
        Type::CEnum(cenum) => {
            let value = unsafe { &mut *(ptr as *mut i32) };
            ValueReflection::CEnum(Box::new(CEnumReflection {
                name: cenum.name,
                val: value.into(),
                variants: cenum.variants,
            }))
        }
        Type::U32 => {
            let value = unsafe { &mut *(ptr as *mut u32) };
            ValueReflection::U32(value.into())
        }
        Type::F32 => {
            let value = unsafe { &mut *(ptr as *mut f32) };
            ValueReflection::F32(value.into())
        }
        Type::I64 => {
            let value = unsafe { &mut *(ptr as *mut i64) };
            ValueReflection::I64(value.into())
        }
        Type::U64 => {
            let value = unsafe { &mut *(ptr as *mut u64) };
            ValueReflection::U64(value.into())
        }
        Type::F64 => {
            let value = unsafe { &mut *(ptr as *mut f64) };
            ValueReflection::F64(value.into())
        }
        Type::ISize => {
            let value = unsafe { &mut *(ptr as *mut isize) };
            ValueReflection::ISize(value.into())
        }
        Type::USize => {
            let value = unsafe { &mut *(ptr as *mut usize) };
            ValueReflection::USize(value.into())
        }
        Type::Bool => {
            let value = unsafe { &mut *(ptr as *mut bool) };
            ValueReflection::Bool(value.into())
        }
        Type::String => {
            let value = unsafe { &mut *(ptr as *mut String) };
            ValueReflection::String(value.into())
        }
        Type::Struct(s) => ValueReflection::Struct(Box::new(unsafe { reflect_struct(ptr, s) })),
        Type::Vec(v) => ValueReflection::Vec(Box::new(VecReflection {
            element: v.element,
            ptr,
            vtable: &v.vtable,
            skip: v.skip,
        })),
        Type::HashMap(hm) => ValueReflection::HashMap(Box::new(HMReflection {
            key: hm.key,
            value: hm.value,
            ptr,
            vtable: &hm.vtable,
            skip: hm.skip,
        })),
        Type::HashSet(hs) => ValueReflection::HashSet(Box::new(HSReflection {
            element: hs.element,
            ptr,
            vtable: &hs.vtable,
            skip: hs.skip,
        })),
        Type::Option(o) => ValueReflection::Option(Box::new(OptionReflection {
            element: o.element,
            ptr: ptr,
            vtable: &o.vtable,
            skip: o.skip,
        })),
        Type::RustEnum(re_mirror) => ValueReflection::RustEnum(unsafe { (re_mirror.reflect)(ptr) }),
        Type::Box(box_mirror) => unsafe {
            let inner_ptr = (box_mirror.get)(ptr);
            ValueReflection::Box(Box::new(BoxReflection {
                inner: reflect_value(inner_ptr, box_mirror.inner),
            }))
        },
    }
}

pub enum RefOrMut<'a, T> {
    Ref(&'a T),
    Mut(&'a mut T),
}

impl<T: Debug> Debug for RefOrMut<'_, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        (**self).fmt(f)
    }
}

impl<T> Deref for RefOrMut<'_, T> {
    type Target = T;
    fn deref(&self) -> &<Self as Deref>::Target {
        match self {
            RefOrMut::Ref(r) => r,
            RefOrMut::Mut(r) => r,
        }
    }
}

impl<T> DerefMut for RefOrMut<'_, T> {
    #[track_caller]
    fn deref_mut(&mut self) -> &mut <Self as Deref>::Target {
        match self {
            RefOrMut::Ref(_) => panic!("Immutable reference can't be used as mutable."),
            RefOrMut::Mut(r) => r,
        }
    }
}

impl<'a, T> From<&'a T> for RefOrMut<'a, T> {
    fn from(r: &'a T) -> Self {
        Self::Ref(r)
    }
}

impl<'a, T> From<&'a mut T> for RefOrMut<'a, T> {
    fn from(r: &'a mut T) -> Self {
        Self::Mut(r)
    }
}
