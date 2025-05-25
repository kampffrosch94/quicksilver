use quicksilver::*;
use quicksilver::{Quicksilver, json::from_json, reflect};

#[derive(Debug, PartialEq, Quicksilver)]
struct Point {
    x: i32,
    y: i32,
}

#[derive(Debug, PartialEq, Quicksilver)]
struct VecHolder {
    name: String,
    age: i32,
    alive: bool,
    values: Vec<Point>,
}

#[test]
fn vec_roundtrip() {
    let mut val = VecHolder {
        name: "Kampffrosch".to_string(),
        age: 30,
        alive: true,
        values: vec![
            Point { x: 1, y: 2 },
            Point { x: 2, y: 4 },
            Point { x: 3, y: 6 },
        ],
    };
    let s = reflect(&mut val).to_json_string();
    let val2 = from_json::<VecHolder>(&s);
    dbg!(&val2);
    assert_eq!(val, val2);
}
