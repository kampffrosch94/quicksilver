use std::collections::{HashMap, HashSet};

use quicksilver::reflections_ref::reflect_ref;
use quicksilver::{Quicksilver, json::from_json};

#[derive(Debug, PartialEq, Quicksilver, Hash, Eq)]
struct Point {
    x: i32,
    y: i32,
}

#[derive(Debug, PartialEq, Quicksilver)]
struct VecHolder {
    name: String,
    age: Box<i32>,
    alive: bool,
    values: Vec<Point>,
}

#[derive(Debug, PartialEq, Quicksilver)]
#[expect(unused)]
struct TuplePoint2f(f32, f32);

#[test]
fn vec_roundtrip() {
    let val = VecHolder {
        name: "Kampffrosch".to_string(),
        age: Box::new(31),
        alive: true,
        values: vec![
            Point { x: 1, y: 2 },
            Point { x: 2, y: 4 },
            Point { x: 3, y: 6 },
        ],
    };
    let s = reflect_ref(&val).to_json();
    let val2 = from_json::<VecHolder>(&s);
    dbg!(&val2);
    assert_eq!(val, val2);
}

#[derive(Debug, PartialEq, Quicksilver)]
struct HMHolder {
    map: HashMap<Point, String>,
}

#[test]
fn hm_roundtrip() {
    let mut val = HMHolder {
        map: HashMap::new(),
    };
    val.map
        .insert(Point { x: 1, y: 2 }, "Point of no return".to_string());
    val.map.insert(
        Point { x: 2, y: 2 },
        "Point of its really too late now".to_string(),
    );
    val.map
        .insert(Point { x: 3, y: 2 }, "Point of deep regret".to_string());
    let s = reflect_ref(&val).to_json();
    println!("{}", &s);
    let val2 = from_json::<HMHolder>(&s);
    dbg!(&val2);
    assert_eq!(val, val2);
}

#[derive(Debug, PartialEq, Quicksilver)]
struct HMHolder2 {
    name: String,
    #[quicksilver(skip)]
    map: HashMap<Point, Box<String>>,
}

#[test]
fn hm_roundtrip_skipped() {
    let mut val = HMHolder2 {
        name: "blab".to_string(),
        map: HashMap::new(),
    };
    val.map.insert(
        Point { x: 1, y: 2 },
        Box::new("Point of no return".to_string()),
    );
    val.map.insert(
        Point { x: 2, y: 2 },
        Box::new("Point of its really too late now".to_string()),
    );
    let s = reflect_ref(&val).to_json();
    println!("{}", &s);
    let val2 = from_json::<HMHolder2>(&s);
    dbg!(&val2);
    assert_ne!(val, val2);
    val.map.clear();
    assert_eq!(val, val2);
}

#[derive(Debug, PartialEq, Quicksilver)]
struct MyData {
    id: u32,
    name: String,
    value: f32,
    location: Point,
    is_active: usize, // Using i32 to demonstrate another integer type
}

#[test]
fn test_json_serialization() {
    let my_data = MyData {
        id: 789,
        name: "Another \"Test\" String with \\backslashes\\".to_string(),
        value: 123.45,
        location: Point { x: -5, y: 30 },
        is_active: 1,
    };

    let reflected_data = reflect_ref(&my_data);
    let json_string = reflected_data.to_json();

    let expected_json = r#"{"id":789,"name":"Another \"Test\" String with \\backslashes\\","value":123.45,"location":{"x":-5,"y":30},"is_active":1}"#;

    assert_eq!(json_string, expected_json);

    let deserialized = from_json::<MyData>(&json_string);
    assert_eq!(my_data, deserialized);
}

#[derive(Debug, PartialEq, Quicksilver)]
struct OptionHolder {
    val: Option<Point>,
    val2: Option<Point>,
}

#[test]
fn option_roundtrip() {
    let val = OptionHolder {
        val: Some(Point { x: 1, y: 2 }),
        val2: None,
    };
    let s = reflect_ref(&val).to_json();
    let val2 = from_json::<OptionHolder>(&s);
    dbg!(&val2);
    assert_eq!(val, val2);
}

#[derive(Debug, PartialEq, Quicksilver)]
struct OptionHolder2 {
    #[quicksilver(skip)]
    inner: Option<Point>,
    inner2: Option<Point>,
}

#[test]
fn option_roundtrip_skip() {
    let mut val = OptionHolder2 {
        inner: Some(Point { x: 1, y: 2 }),
        inner2: None,
    };
    let s = reflect_ref(&val).to_json();
    let val2 = from_json::<OptionHolder2>(&s);
    dbg!(&val2);
    assert_ne!(val, val2);
    val.inner = None;
    assert_eq!(val, val2);
}

#[derive(Debug, PartialEq, Quicksilver)]
struct HSHolder {
    inner: HashSet<Point>,
}

#[test]
fn hs_roundtrip() {
    let mut val = HSHolder {
        inner: HashSet::new(),
    };
    val.inner.insert(Point { x: 2, y: 3 });
    val.inner.insert(Point { x: 2, y: 4 });
    val.inner.insert(Point { x: 2, y: 5 });
    let s = reflect_ref(&val).to_json();
    let val2 = from_json::<HSHolder>(&s);
    dbg!(&val2);
    assert_eq!(val, val2);
}

#[derive(Debug, PartialEq, Quicksilver)]
struct HSHolder2 {
    #[quicksilver(skip)]
    inner: HashSet<Point>,
}

#[test]
fn hs_roundtrip2() {
    let mut val = HSHolder2 {
        inner: HashSet::new(),
    };
    val.inner.insert(Point { x: 2, y: 3 });
    val.inner.insert(Point { x: 2, y: 4 });
    val.inner.insert(Point { x: 2, y: 5 });
    let s = reflect_ref(&val).to_json();
    let val2 = from_json::<HSHolder2>(&s);
    dbg!(&val2);
    assert_ne!(val, val2);
    val.inner.clear();
    assert_eq!(val, val2);
}

#[derive(Debug, Quicksilver)]
#[allow(unused, private_interfaces)]
pub struct Fov(pub HashSet<Point>);

#[derive(Debug, Quicksilver)]
#[allow(unused, private_interfaces)]
pub struct Fov2(pub(crate) HashSet<Point>);

#[derive(Debug, PartialEq, Quicksilver)]
struct BoolHolder {
    val: Vec<bool>,
}

#[test]
fn bool_roundtrip() {
    let val = BoolHolder {
        val: vec![true, false, true],
    };
    let s = reflect_ref(&val).to_json();
    let val2 = from_json::<BoolHolder>(&s);
    dbg!(&val2);
    assert_eq!(val, val2);
}
