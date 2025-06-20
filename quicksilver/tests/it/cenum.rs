use quicksilver::reflections_ref::reflect_ref;
use quicksilver::{Quicksilver, json::from_json};

#[repr(C)]
#[derive(Debug, Quicksilver, PartialEq)]
#[allow(unused)]
enum Number {
    #[allow(unused)]
    Zero,
    #[allow(unused)]
    One,
    Two,
    #[allow(unused)]
    Four,
}

#[derive(Debug, Quicksilver, PartialEq)]
pub struct Container {
    /// This is an enum
    number: Number,
}

#[test]
fn enum_roundtrip() {
    let val = Container {
        number: Number::Two,
    };
    let s = reflect_ref(&val).to_json();
    println!("{}", &s);
    let val2 = from_json::<Container>(&s);
    dbg!(&val2);
    assert_eq!(val, val2);
}
