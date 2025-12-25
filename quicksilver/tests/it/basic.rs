use quicksilver::Quicksilver;
use quicksilver::reflections::ValueReflection;
use quicksilver::reflections::reflect;

#[derive(Debug, Quicksilver)]
struct Point {
    x: i32,
    y: i32,
}

#[derive(Debug, Quicksilver)]
/// Multi line
/// Doc string
struct MyData {
    // fasdjfl
    id: u32,
    /// Docs
    name: String,
    /// Multi
    /// Line Docs
    value: f32,
    location: Point,
    is_active: i32, // Using i32 to demonstrate another integer type
}

#[derive(Debug, Quicksilver)]
#[expect(unused)]
pub struct Player {
    pub pulse: f32,
    pub last_action: i32,
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

    let ValueReflection::Struct(ref mut reflected_data) = reflect(&mut my_data) else {
        panic!()
    };

    assert_eq!(reflected_data.fields.len(), 5);

    let id_field = &mut reflected_data.fields[0];
    assert_eq!(id_field.name, "id");
    if let ValueReflection::U32(id_ref) = &mut id_field.value {
        assert_eq!(**id_ref, 123);
        **id_ref = 456; // Modify through reflection
    } else {
        panic!("Expected U32 for 'id' field");
    }

    let name_field = &mut reflected_data.fields[1];
    assert_eq!(name_field.name, "name");
    if let ValueReflection::String(name_ref) = &mut name_field.value {
        assert_eq!(**name_ref, "Test Data");
        **name_ref = "Modified Name".to_string(); // Modify through reflection
    } else {
        panic!("Expected String for 'name' field");
    }

    let value_field = &mut reflected_data.fields[2];
    assert_eq!(value_field.name, "value");
    if let ValueReflection::F32(value_ref) = &mut value_field.value {
        assert_eq!(**value_ref, 42.5);
        **value_ref = 99.9; // Modify through reflection
    } else {
        panic!("Expected F32 for 'value' field");
    }

    let location_field = &mut reflected_data.fields[3];
    assert_eq!(location_field.name, "location");
    if let ValueReflection::Struct(point_reflection) = &mut location_field.value {
        assert_eq!(point_reflection.fields.len(), 2);

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

    let is_active_field = &mut reflected_data.fields[4];
    assert_eq!(is_active_field.name, "is_active");
    if let ValueReflection::I32(is_active_ref) = &mut is_active_field.value {
        assert_eq!(**is_active_ref, 1);
        **is_active_ref = 0; // Modify through reflection
    } else {
        panic!("Expected I32 for 'is_active' field");
    }

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
