use std::collections::HashMap;
use std::hash::Hash;
use std::{alloc::Layout, collections::HashSet};

use map::{HMVtable, HMVtableCreator};
use option::{OptionVtable, OptionVtableCreator};
pub use quicksilver_derive::Quicksilver;
use set::{HSVtable, HSVtableCreator};
use vec::{VecVtable, VecVtableCreator};

pub mod empty;
pub mod json;
pub mod map;
pub mod option;
pub mod reflections;
pub mod reflections_ref;
pub mod set;
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
    HashSet(HSType),
    Option(OptionType),
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
            Type::Vec(v) => unsafe { Layout::from_size_align_unchecked(v.size, v.align) },
            Type::HashMap(hm) => unsafe { Layout::from_size_align_unchecked(hm.size, hm.align) },
            Type::HashSet(hs) => unsafe { Layout::from_size_align_unchecked(hs.size, hs.align) },
            Type::Struct(s) => unsafe { Layout::from_size_align_unchecked(s.size, s.align) },
            Type::CEnum(e) => unsafe { Layout::from_size_align_unchecked(e.size, e.align) },
            Type::Option(o) => unsafe { Layout::from_size_align_unchecked(o.size, o.align) },
        }
    }
}

#[derive(Debug)]
pub struct VecType {
    pub element: &'static Type,
    pub vtable: VecVtable,
    pub skip: bool,
    pub size: usize,
    pub align: usize,
}

#[derive(Debug)]
pub struct HMType {
    pub key: &'static Type,
    pub value: &'static Type,
    pub vtable: HMVtable,
    pub skip: bool,
    pub size: usize,
    pub align: usize,
}

#[derive(Debug)]
pub struct HSType {
    pub element: &'static Type,
    pub vtable: HSVtable,
    pub skip: bool,
    pub size: usize,
    pub align: usize,
}

#[derive(Debug)]
pub struct OptionType {
    pub element: &'static Type,
    pub vtable: OptionVtable,
    pub skip: bool,
    pub size: usize,
    pub align: usize,
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
        size: size_of::<Self>(),
        align: align_of::<Self>(),
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
        size: size_of::<Self>(),
        align: align_of::<Self>(),
    });
}

impl<T> Quicksilver for HashSet<T>
where
    T: Eq + Hash,
    T: Quicksilver,
{
    const MIRROR: Type = Type::HashSet(HSType {
        element: &T::MIRROR,
        vtable: HSVtableCreator::<T>::VTABLE,
        skip: false,
        size: size_of::<Self>(),
        align: align_of::<Self>(),
    });
}

impl<T> Quicksilver for Option<T>
where
    T: Quicksilver,
{
    const MIRROR: Type = Type::Option(OptionType {
        element: &T::MIRROR,
        vtable: OptionVtableCreator::<T>::VTABLE,
        skip: false,
        size: size_of::<Self>(),
        align: align_of::<Self>(),
    });
}
