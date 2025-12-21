use quicksilver::Quicksilver;
use quicksilver::json::from_json;
use quicksilver::reflections::{FieldReflection, RustEnumReflection, reflect, reflect_value};
use quicksilver::reflections_ref::{reflect_ref, reflect_value_ref};
use quicksilver::{RustEnum, RustEnumVariant, Type};

#[allow(unused)]
#[derive(Debug, PartialEq)]
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

impl Quicksilver for Ability {
    const MIRROR: quicksilver::Type = Type::RustEnum(&RustEnum {
        size: size_of::<Self>(),
        align: align_of::<Self>(),
        name: "Ability",
        variants: &[
            RustEnumVariant {
                name: "DoNothing",
                fields: &[],
            },
            RustEnumVariant {
                name: "Attack",
                fields: &[("who", Type::String), ("damage", Type::I32)],
            },
            RustEnumVariant {
                name: "Shout",
                fields: &[("0", Type::String)],
            },
        ],
        reflect: |ptr| {
            let enum_val: &mut Self = unsafe { &mut *(ptr as *mut Self) };
            match enum_val {
                Self::DoNothing => RustEnumReflection {
                    name: "Ability",
                    variant_name: "DoNothing",
                    variant_idx: 0,
                    ty: &Self::MIRROR,
                    fields: vec![],
                },
                Ability::Attack { who, damage } => RustEnumReflection {
                    name: "Ability",
                    variant_name: "Attack",
                    variant_idx: 1,
                    ty: &Self::MIRROR,
                    fields: vec![
                        FieldReflection {
                            name: "who",
                            value: unsafe {
                                reflect_value(&raw mut *who as *mut u8, &String::MIRROR)
                            },
                        },
                        FieldReflection {
                            name: "damage",
                            value: unsafe {
                                reflect_value(&raw mut *damage as *mut u8, &i32::MIRROR)
                            },
                        },
                    ],
                },
                Ability::Shout(val0, ) => RustEnumReflection {
                    name: "Ability",
                    variant_name: "Shout",
                    variant_idx: 2,
                    ty: &Self::MIRROR,
                    fields: vec![FieldReflection {
                        name: "0",
                        value: unsafe { reflect_value(&raw mut *val0 as *mut u8, &String::MIRROR) },
                    }],
                },
            }
        },
        reflect_ref: |ptr| {
            let enum_val: &Self = unsafe { &*(ptr as *mut Self) };
            match enum_val {
                Ability::DoNothing => RustEnumReflection {
                    name: "Ability",
                    variant_name: "DoNothing",
                    variant_idx: 0,
                    ty: &Self::MIRROR,
                    fields: vec![],
                },
                Ability::Attack { who, damage } => RustEnumReflection {
                    name: "Ability",
                    variant_name: "Attack",
                    variant_idx: 1,
                    ty: &Self::MIRROR,
                    fields: vec![
                        FieldReflection {
                            name: "who",
                            value: unsafe {
                                reflect_value_ref(&raw const *who as *const u8, &String::MIRROR)
                            },
                        },
                        FieldReflection {
                            name: "damage",
                            value: unsafe {
                                reflect_value_ref(&raw const *damage as *mut u8, &i32::MIRROR)
                            },
                        },
                    ],
                },
                Ability::Shout(val0) => RustEnumReflection {
                    name: "Ability",
                    variant_name: "Shout",
                    variant_idx: 2,
                    ty: &Self::MIRROR,
                    fields: vec![FieldReflection {
                        name: "0",
                        value: unsafe {
                            reflect_value_ref(&raw const *val0 as *mut u8, &String::MIRROR)
                        },
                    }],
                },
            }
        },
        write: |this, variant, fields| {
            let this = this as *mut Self;
            match (variant, fields) {
                (0, []) => {
                    unsafe { ::std::ptr::write(this, Self::DoNothing) };
                }
                (1, [who, damage]) => {
                    let who = unsafe { Box::from_raw(*who as *mut String) };
                    let damage = unsafe { Box::from_raw(*damage as *mut i32) };
                    unsafe {::std::ptr::write(this, Self::Attack {who: *who, damage: *damage,},)};
                }
                (2, [val0]) => {
                    let val0 = unsafe { Box::from_raw(*val0 as *mut String) };
                    unsafe { ::std::ptr::write(this, Self::Shout(*val0)) };
                }
                _ => unreachable!("Illegal operation. Setting enum variant {variant} on Ability"),
            }
        },
    });
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
    // let val2 = from_json::<Container>(&s);
    // dbg!(&val2);
    // assert_eq!(val, val2);
}

#[allow(unused)]
#[derive(Debug, PartialEq, Quicksilver)]
enum Ability2 {
    DoNothing,
    Attack { who: String, damage: i32 },
    Shout(String),
}
