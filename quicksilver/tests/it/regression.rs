use quicksilver::{json::from_json, Quicksilver};

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
