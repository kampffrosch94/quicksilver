use std::alloc::Layout;
use std::collections::HashMap;
use std::hash::Hash;

use map::{HMVtable, HMVtableCreator};
pub use quicksilver_derive::Quicksilver;
use vec::{VecVtable, VecVtableCreator};

pub mod empty;
pub mod json;
pub mod map;
pub mod reflections;
pub mod reflections_ref;
pub mod vec;

#[derive(Debug)]
pub enum Type {
    I32,
    U32,
    F32,
    I64,
    U64,
    F64,
    ISize,
    USize,
    Bool,
    String,
    Vec(VecType),
    HashMap(HMType),
    Struct(&'static Struct),
    CEnum(&'static CEnum),
}

impl Type {
    pub const fn layout(&self) -> Layout {
        match self {
            Type::I32 => Layout::new::<i32>(),
            Type::U32 => Layout::new::<u32>(),
            Type::F32 => Layout::new::<f32>(),
            Type::I64 => Layout::new::<i64>(),
            Type::U64 => Layout::new::<u64>(),
            Type::F64 => Layout::new::<f64>(),
            Type::ISize => Layout::new::<isize>(),
            Type::USize => Layout::new::<usize>(),
            Type::Bool => Layout::new::<bool>(),
            Type::String => Layout::new::<String>(),
            Type::Vec(_) => Layout::new::<Vec<i32>>(),
            Type::HashMap(_) => Layout::new::<HashMap<i32, i32>>(),
            Type::Struct(s) => unsafe { Layout::from_size_align_unchecked(s.size, s.align) },
            Type::CEnum(e) => unsafe { Layout::from_size_align_unchecked(e.size, e.align) },
        }
    }
}

#[derive(Debug)]
pub struct VecType {
    pub element: &'static Type,
    pub vtable: VecVtable,
    pub skip: bool,
}

#[derive(Debug)]
pub struct HMType {
    pub key: &'static Type,
    pub value: &'static Type,
    pub vtable: HMVtable,
    pub skip: bool,
}

#[derive(Debug)]
pub struct Field {
    pub name: &'static str,
    pub ty: Type,
    pub offset: usize,
}

#[derive(Debug)]
pub struct Struct {
    pub size: usize,
    pub align: usize,
    pub name: &'static str,
    pub fields: &'static [Field],
}

#[derive(Debug)]
pub struct CEnum {
    pub size: usize,
    pub align: usize,
    pub name: &'static str,
    pub variants: &'static [(i32, &'static str)],
}

pub trait Quicksilver {
    const MIRROR: Type;
}

// macro used to implement Reflectable for primitive types
macro_rules! impl_reflectable {
    ($ty:ty, $e:expr) => {
        impl Quicksilver for $ty {
            const MIRROR: Type = $e;
        }
    };
}

impl_reflectable!(bool, Type::Bool);
impl_reflectable!(u32, Type::U32);
impl_reflectable!(i32, Type::I32);
impl_reflectable!(f32, Type::F32);
impl_reflectable!(u64, Type::U64);
impl_reflectable!(i64, Type::I64);
impl_reflectable!(f64, Type::F64);
impl_reflectable!(usize, Type::USize);
impl_reflectable!(isize, Type::ISize);
impl_reflectable!(String, Type::String);

impl<T> Quicksilver for Vec<T>
where
    T: Quicksilver,
{
    const MIRROR: Type = Type::Vec(VecType {
        element: &T::MIRROR,
        vtable: VecVtableCreator::<T>::VTABLE,
        skip: false,
    });
}

impl<Key, Value> Quicksilver for HashMap<Key, Value>
where
    Key: Eq + Hash,
    Key: Quicksilver,
    Value: Quicksilver,
{
    const MIRROR: Type = Type::HashMap(HMType {
        key: &Key::MIRROR,
        value: &Value::MIRROR,
        vtable: HMVtableCreator::<Key, Value>::VTABLE,
        skip: false,
    });
}
