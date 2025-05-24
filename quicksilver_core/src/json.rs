use crate::{FieldTypeReflection, StructReflection};

impl<'a> StructReflection<'a> {
    pub fn to_json_string(&self) -> String {
        let mut json_parts = Vec::new();
        for field in &self.fields {
            let field_name = format!("\"{}\"", field.name);
            let field_value = match &field.ty {
                FieldTypeReflection::I32(val) => format!("{}", **val),
                FieldTypeReflection::U32(val) => format!("{}", **val),
                FieldTypeReflection::F32(val) => format!("{}", **val),
                FieldTypeReflection::String(val) => {
                    let escaped_val = val.replace(r"\", r"\\").replace(r#"""#, r#"\""#);
                    format!("\"{}\"", escaped_val)
                }
                FieldTypeReflection::Struct(s_ref) => s_ref.to_json_string(),
            };
            json_parts.push(format!("{}:{}", field_name, field_value));
        }
        format!("{{{}}}", json_parts.join(","))
    }
}

#[cfg(test)]
mod test {
    use std::mem;

    use crate::*;
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

        assert_eq!(json_string, expected_json);
    }
}
