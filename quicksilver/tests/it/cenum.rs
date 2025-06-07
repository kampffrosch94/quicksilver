use quicksilver::reflections_ref::reflect_ref;
use quicksilver::{Quicksilver, json::from_json};

#[repr(C)]
#[derive(Debug, Quicksilver, PartialEq)]
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
struct Container {
    number: Number,
}

#[test]
fn enum_roundtrip() {
    let val = Container {
        number: Number::Two,
    };
    let s = reflect_ref(&val).struct_to_json();
    println!("{}", &s);
    let val2 = from_json::<Container>(&s);
    dbg!(&val2);
    assert_eq!(val, val2);
}
