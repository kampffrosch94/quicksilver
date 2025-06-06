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

pub trait Reflection {
    const MIRROR: &'static Struct;
}

/// Marks types that can be reflected
/// implement `Reflection` for making them reflectable
pub trait Reflectable {
    const TYPE: Type;
}

// macro used to implement Reflectable for primitive types
macro_rules! impl_reflectable {
    ($ty:ty, $e:expr) => {
        impl Reflectable for $ty {
            const TYPE: Type = $e;
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

impl<T> Reflectable for T
where
    T: Reflection,
{
    const TYPE: Type = Type::Struct(T::MIRROR);
}

impl<T> Reflectable for Vec<T>
where
    T: Reflectable,
{
    const TYPE: Type = Type::Vec(VecType {
        element: &T::TYPE,
        vtable: VecVtableCreator::<T>::VTABLE,
        skip: false,
    });
}

impl<Key, Value> Reflectable for HashMap<Key, Value>
where
    Key: Eq + Hash,
    Key: Reflectable,
    Value: Reflectable,
{
    const TYPE: Type = Type::HashMap(HMType {
        key: &Key::TYPE,
        value: &Value::TYPE,
        vtable: HMVtableCreator::<Key, Value>::VTABLE,
        skip: false,
    });
}

#[cfg(test)]
mod tests {
    use crate::reflections::ValueReflection;

    use super::*;
    use crate::reflections::reflect;

    #[derive(Debug, Quicksilver)]
    struct Point {
        x: i32,
        y: i32,
    }

    #[derive(Debug, Quicksilver)]
    struct MyData {
        id: u32,
        name: String,
        value: f32,
        location: Point,
        is_active: i32, // Using i32 to demonstrate another integer type
    }

    #[test]
    fn test_reflection_my_data() {
        let mut my_data = MyData {
            id: 123,
            name: "Test Data".to_string(),
            value: 42.5,
            location: Point { x: 10, y: 20 },
            is_active: 1,
        };

        let mut reflected_data = reflect(&mut my_data);

        assert_eq!(reflected_data.fields.len(), 5);

        // Test 'id' field
        let id_field = &mut reflected_data.fields[0];
        assert_eq!(id_field.name, "id");
        if let ValueReflection::U32(id_ref) = &mut id_field.value {
            assert_eq!(**id_ref, 123);
            **id_ref = 456; // Modify through reflection
        } else {
            panic!("Expected U32 for 'id' field");
        }

        // Test 'name' field
        let name_field = &mut reflected_data.fields[1];
        assert_eq!(name_field.name, "name");
        if let ValueReflection::String(name_ref) = &mut name_field.value {
            assert_eq!(**name_ref, "Test Data");
            **name_ref = "Modified Name".to_string(); // Modify through reflection
        } else {
            panic!("Expected String for 'name' field");
        }

        // Test 'value' field
        let value_field = &mut reflected_data.fields[2];
        assert_eq!(value_field.name, "value");
        if let ValueReflection::F32(value_ref) = &mut value_field.value {
            assert_eq!(**value_ref, 42.5);
            **value_ref = 99.9; // Modify through reflection
        } else {
            panic!("Expected F32 for 'value' field");
        }

        // Test 'location' field (nested struct)
        let location_field = &mut reflected_data.fields[3];
        assert_eq!(location_field.name, "location");
        if let ValueReflection::Struct(point_reflection) = &mut location_field.value {
            assert_eq!(point_reflection.fields.len(), 2);

            // Test 'location.x'
            let x_field = &mut point_reflection.fields[0];
            assert_eq!(x_field.name, "x");
            if let ValueReflection::I32(x_ref) = &mut x_field.value {
                assert_eq!(**x_ref, 10);
                **x_ref = 100; // Modify through reflection
            } else {
                panic!("Expected I32 for 'location.x' field");
            }

            // Test 'location.y'
            let y_field = &mut point_reflection.fields[1];
            assert_eq!(y_field.name, "y");
            if let ValueReflection::I32(y_ref) = &mut y_field.value {
                assert_eq!(**y_ref, 20);
                **y_ref = 200; // Modify through reflection
            } else {
                panic!("Expected I32 for 'location.y' field");
            }
        } else {
            panic!("Expected Struct for 'location' field");
        }

        // Test 'is_active' field
        let is_active_field = &mut reflected_data.fields[4];
        assert_eq!(is_active_field.name, "is_active");
        if let ValueReflection::I32(is_active_ref) = &mut is_active_field.value {
            assert_eq!(**is_active_ref, 1);
            **is_active_ref = 0; // Modify through reflection
        } else {
            panic!("Expected I32 for 'is_active' field");
        }

        // Verify that modifications through reflection are reflected in the original struct
        assert_eq!(my_data.id, 456);
        assert_eq!(my_data.name, "Modified Name");
        assert_eq!(my_data.value, 99.9);
        assert_eq!(my_data.location.x, 100);
        assert_eq!(my_data.location.y, 200);
        assert_eq!(my_data.is_active, 0);
    }

    #[derive(Quicksilver)]
    struct Point3d {
        label: String,
        x: i32,
        y: u32,
        z: f32,
    }

    #[test]
    fn test_derive() {
        let _mirror = Point3d::MIRROR;
    }
}
