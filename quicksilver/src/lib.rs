use std::alloc::Layout;

pub use quicksilver_derive::Quicksilver;
use vec::{VecReflection, VecVtable};

pub mod json;
pub mod vec;

#[derive(Debug)]
pub enum Type {
    I32,
    U32,
    F32,
    String,
    Vec(VecType),
    Struct(&'static Struct),
}

impl Type {
    pub const fn layout(&self) -> Layout {
        match self {
            Type::I32 => Layout::new::<i32>(),
            Type::U32 => Layout::new::<u32>(),
            Type::F32 => Layout::new::<f32>(),
            Type::String => Layout::new::<String>(),
            Type::Vec(_) => Layout::new::<Vec<i32>>(),
            Type::Struct(s) => unsafe { Layout::from_size_align_unchecked(s.size, s.align) },
        }
    }
}

#[derive(Debug)]
pub struct VecType {
    pub element: &'static Type,
    pub vtable: VecVtable,
}

#[derive(Debug)]
pub struct Field {
    name: &'static str,
    ty: Type,
    offset: usize,
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

#[repr(C)]
pub enum ValueReflection<'a> {
    I32(&'a mut i32),
    U32(&'a mut u32),
    F32(&'a mut f32),
    String(&'a mut String),
    Struct(Box<StructReflection<'a>>),
    Vec(Box<VecReflection<'a>>),
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
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::mem;
    #[derive(Debug)]
    struct Point {
        x: i32,
        y: i32,
    }

    impl Reflection for Point {
        const MIRROR: &'static Struct = &Struct {
            name: "Point",
            size: size_of::<Self>(),
            align: align_of::<Self>(),
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

    #[derive(Debug)]
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
            align: align_of::<Self>(),
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
        let mirror = Point3d::MIRROR;
    }
}
