use std::collections::HashMap;

use quicksilver::*;
use quicksilver::{Quicksilver, json::from_json, reflect};

#[derive(Debug, PartialEq, Quicksilver, Hash, Eq)]
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

#[derive(Debug, PartialEq, Quicksilver)]
struct TuplePoint2f(f32, f32);

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
    let s = reflect(&mut val).to_json_string();
    let val2 = from_json::<HMHolder>(&s);
    dbg!(&val2);
    assert_eq!(val, val2);
}
