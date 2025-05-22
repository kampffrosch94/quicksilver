#[non_exhaustive]
pub enum Type {
    I32,
    U32,
    F32,
    String,
    Vec(&'static Type),
    Struct(&'static Struct),
}

pub struct Field {
    name: &'static str,
    ty: Type,
    offset: usize,
}

pub struct Struct {
    fields: &'static [Field],
}

pub trait Reflection {
    const MIRROR: &'static Struct;
}

#[repr(C)]
pub enum FieldTypeReflection<'a> {
    I32(&'a mut i32),
    U32(&'a mut u32),
    F32(&'a mut f32),
    String(&'a mut String),
    Struct(Box<StructReflection<'a>>),
}

#[repr(C)]
pub struct FieldReflection<'a> {
    pub name: &'static str,
    pub ty: FieldTypeReflection<'a>,
}

#[repr(C)]
pub struct StructReflection<'a> {
    pub fields: Vec<FieldReflection<'a>>,
}

pub fn reflect<T: Reflection>(val: &mut T) -> StructReflection<'_> {
    reflect_inner(val as *mut T as *mut u8, T::MIRROR)
}

pub fn reflect_inner(val: *mut u8, mirror: &Struct) -> StructReflection<'_> {
    let mut fields: Vec<FieldReflection> = Vec::new();
    for field in mirror.fields {
        match field.ty {
            Type::I32 => {
                let value = unsafe {
                    let ptr = val.add(field.offset) as *mut i32;
                    &mut *ptr
                };
                fields.push(FieldReflection {
                    name: field.name,
                    ty: FieldTypeReflection::I32(value),
                });
            }
            Type::U32 => {
                let value = unsafe {
                    let ptr = val.add(field.offset) as *mut u32;
                    &mut *ptr
                };
                fields.push(FieldReflection {
                    name: field.name,
                    ty: FieldTypeReflection::U32(value),
                });
            }
            Type::F32 => {
                let value = unsafe {
                    let ptr = val.add(field.offset) as *mut f32;
                    &mut *ptr
                };
                fields.push(FieldReflection {
                    name: field.name,
                    ty: FieldTypeReflection::F32(value),
                });
            }
            Type::String => {
                let value = unsafe {
                    let ptr = val.add(field.offset) as *mut String;
                    &mut *ptr
                };
                fields.push(FieldReflection {
                    name: field.name,
                    ty: FieldTypeReflection::String(value),
                });
            }
            Type::Struct(s) => {
                let value = unsafe {
                    let ptr = val.add(field.offset) as *mut u8;
                    &mut *ptr
                };
                fields.push(FieldReflection {
                    name: field.name,
                    ty: FieldTypeReflection::Struct(Box::new(reflect_inner(value, s))),
                });
            }
            _ => {
                todo!();
            }
        }
    }
    StructReflection { fields }
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
        if let FieldTypeReflection::U32(id_ref) = &mut id_field.ty {
            assert_eq!(**id_ref, 123);
            **id_ref = 456; // Modify through reflection
        } else {
            panic!("Expected U32 for 'id' field");
        }

        // Test 'name' field
        let name_field = &mut reflected_data.fields[1];
        assert_eq!(name_field.name, "name");
        if let FieldTypeReflection::String(name_ref) = &mut name_field.ty {
            assert_eq!(**name_ref, "Test Data");
            **name_ref = "Modified Name".to_string(); // Modify through reflection
        } else {
            panic!("Expected String for 'name' field");
        }

        // Test 'value' field
        let value_field = &mut reflected_data.fields[2];
        assert_eq!(value_field.name, "value");
        if let FieldTypeReflection::F32(value_ref) = &mut value_field.ty {
            assert_eq!(**value_ref, 42.5);
            **value_ref = 99.9; // Modify through reflection
        } else {
            panic!("Expected F32 for 'value' field");
        }

        // Test 'location' field (nested struct)
        let location_field = &mut reflected_data.fields[3];
        assert_eq!(location_field.name, "location");
        if let FieldTypeReflection::Struct(point_reflection) = &mut location_field.ty {
            assert_eq!(point_reflection.fields.len(), 2);

            // Test 'location.x'
            let x_field = &mut point_reflection.fields[0];
            assert_eq!(x_field.name, "x");
            if let FieldTypeReflection::I32(x_ref) = &mut x_field.ty {
                assert_eq!(**x_ref, 10);
                **x_ref = 100; // Modify through reflection
            } else {
                panic!("Expected I32 for 'location.x' field");
            }

            // Test 'location.y'
            let y_field = &mut point_reflection.fields[1];
            assert_eq!(y_field.name, "y");
            if let FieldTypeReflection::I32(y_ref) = &mut y_field.ty {
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
        if let FieldTypeReflection::I32(is_active_ref) = &mut is_active_field.ty {
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

    impl<'a> StructReflection<'a> {
        /// Serializes the reflected struct data into a JSON string.
        /// This implementation manually constructs the JSON string.
        pub fn to_json_string(&self) -> String {
            let mut json_parts = Vec::new();
            for field in &self.fields {
                let field_name = format!("\"{}\"", field.name);
                let field_value = match &field.ty {
                    FieldTypeReflection::I32(val) => format!("{}", **val),
                    FieldTypeReflection::U32(val) => format!("{}", **val),
                    FieldTypeReflection::F32(val) => format!("{}", **val),
                    FieldTypeReflection::String(val) => {
                        // Escape double quotes and backslashes in the string value
                        let escaped_val = val.replace("\\", "\\\\").replace("\"", "\\\"");
                        format!("\"{}\"", escaped_val)
                    }
                    FieldTypeReflection::Struct(s_ref) => s_ref.to_json_string(), // Recursive call for nested structs
                    _ => todo!(),
                };
                json_parts.push(format!("{}:{}", field_name, field_value));
            }
            format!("{{{}}}", json_parts.join(","))
        }
    }

    #[test]
    fn test_json_serialization() {
        let mut my_data = MyData {
            id: 789,
            name: "Another \"Test\" String with \\backslashes\\".to_string(),
            value: 123.45,
            location: Point { x: -5, y: 30 },
            is_active: 1,
        };

        let reflected_data = reflect(&mut my_data);
        let json_string = reflected_data.to_json_string();

        let expected_json = r#"{"id":789,"name":"Another \"Test\" String with \\backslashes\\","value":123.45,"location":{"x":-5,"y":30},"is_active":1}"#;

        // Note: Floating point comparisons can be tricky. For exact string match,
        // ensure the float serialization matches exactly. In real apps, you'd parse
        // the JSON and compare the numerical values with a tolerance.
        assert_eq!(json_string, expected_json);

        // Test with a modified value to ensure it serializes correctly
        my_data.value = 100.0;
        let reflected_data_modified = reflect(&mut my_data);
        let json_string_modified = reflected_data_modified.to_json_string();
        let expected_json_modified = r#"{"id":789,"name":"Another \"Test\" String with \\backslashes\\","value":100,"location":{"x":-5,"y":30},"is_active":1}"#;
        assert_eq!(json_string_modified, expected_json_modified);
    }
}
