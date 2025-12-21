use quicksilver::Quicksilver;
use quicksilver::RustEnumVariant;
use quicksilver::json::from_json;
use quicksilver::reflections::{FieldReflection, RustEnumReflection, reflect};
use quicksilver::reflections_ref::reflect_ref;

#[allow(unused)]
#[derive(Debug, PartialEq, Quicksilver)]
enum Ability {
    DoNothing,
    Attack { who: String, damage: i32 },
    Shout(String),
}

#[derive(Debug, Quicksilver, PartialEq)]
pub struct Container {
    /// This is a rust enum
    ability: Ability,
}

#[test]
fn rust_enum_roundtrip() {
    let mut val = Container {
        // ability: Ability::Shout("I am the greatest!".into()),
        ability: Ability::Attack {
            who: "Goblin".into(),
            damage: 33,
        },
    };
    let s = reflect(&mut val).to_json();
    println!("{}", &s);
    let val2 = from_json::<Container>(&s);
    dbg!(&val2);
    assert_eq!(val, val2);
}

#[test]
fn rust_enum_roundtrip_ref() {
    let val = Container {
        ability: Ability::Shout("I am the greatest!".into()),
    };
    let s = reflect_ref(&val).to_json();
    println!("{}", &s);
    let val2 = from_json::<Container>(&s);
    dbg!(&val2);
    assert_eq!(val, val2);
}

#[test]
fn rust_enum_naked_roundtrip() {
    let val = Ability::Attack {
        who: "Goblin".into(),
        damage: 33,
    };
    let s = reflect_ref(&val).to_json();
    println!("{}", &s);
    let val2 = from_json::<Ability>(&s);
    dbg!(&val2);
    assert_eq!(val, val2);
}
