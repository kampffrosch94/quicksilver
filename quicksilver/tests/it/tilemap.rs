use quicksilver::{Quicksilver, json::from_json, reflections_ref::reflect_ref};

#[derive(Debug, Quicksilver, PartialEq)]
struct Tilemap {
    tiles: Vec<Tile>,
    width: i32,
    height: i32,
}

#[derive(Debug, Quicksilver, Clone, Copy, PartialEq)]
#[repr(C)]
#[allow(unused)]
enum Tile {
    Floor,
    Wall,
}

#[test]
fn tilemap_roundtrip() {
    let val = Tilemap {
        tiles: vec![Tile::Wall; 25],
        width: 5,
        height: 5,
    };
    let s = reflect_ref(&val).to_json();
    let val2 = from_json::<Tilemap>(&s);
    dbg!(&val2);
    assert_eq!(val, val2);
}
