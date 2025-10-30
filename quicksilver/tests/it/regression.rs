use std::collections::HashMap;

use quicksilver::{Quicksilver, json::from_json, reflections_ref::reflect_ref};

#[derive(Debug, Clone, Copy, PartialEq, Quicksilver)]
pub struct FPos {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Quicksilver)]
pub struct DrawPos(pub FPos);

#[test]
fn drawpos_deser() {
    let s = r#"{"0":{"x":384,"y":192}}"#;
    let val2 = from_json::<DrawPos>(&s);
    dbg!(&val2);
}

#[repr(C)]
#[derive(Debug, PartialEq, Quicksilver)]
#[allow(unused)]
enum MyEnum {
    A,
    B,
    C,
}

#[derive(Default, Debug, Quicksilver, PartialEq)]
pub struct Pair(pub u32, pub String);

#[derive(Debug, PartialEq, Quicksilver)]
struct HMHolder {
    map: HashMap<String, Vec<Pair>>,
}

// this wasn't actually a bug, but I'll leave the test in to see that this case is working
#[test]
fn vec_in_hashmap() {
    let mut val = HMHolder {
        map: HashMap::new(),
    };
    let payload = reflect_ref(&MyEnum::A).to_json();
    val.map.insert("MyKey".into(), vec![Pair(0, payload)]);
    let s = reflect_ref(&val).to_json();
    println!("{}", &s);
    let val2 = from_json::<HMHolder>(&s);
    dbg!(&val2);
    assert_eq!(val, val2);
}
