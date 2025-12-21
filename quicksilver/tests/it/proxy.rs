use std::collections::HashMap;

use quicksilver::{Quicksilver, json::from_json, reflections_ref::reflect_ref};

#[derive(PartialEq, Debug)]
struct Entity {
    id: u32,
    generation: u32,
}

#[derive(PartialEq, Debug)]
#[repr(transparent)]
struct EntityWrapper(Entity);

impl ::quicksilver::Quicksilver for EntityWrapper {
    const MIRROR: ::quicksilver::Type = ::quicksilver::Type::Struct(&::quicksilver::Struct {
        name: "EntityWrapper",
        size: ::std::mem::size_of::<Self>(),
        align: align_of::<Self>(),
        fields: &[
            ::quicksilver::Field {
                name: "id",
                ty: u32::MIRROR,
                offset: ::std::mem::offset_of!(Self, 0.id),
            },
            ::quicksilver::Field {
                name: "generation",
                ty: u32::MIRROR,
                offset: ::std::mem::offset_of!(Self, 0.generation),
            },
        ],
    });
}

#[derive(PartialEq, Debug, Quicksilver)]
struct Container {
    #[quicksilver(proxy(Entity, EntityWrapper))]
    leader: Entity,
    #[quicksilver(proxy(Entity, EntityWrapper))]
    followers: Vec<Entity>,
    #[quicksilver(proxy(Entity, EntityWrapper))]
    mappings: HashMap<u32, Entity>,
}

#[test]
fn subst_roundtrip() {
    let mut val = Container {
        leader: Entity {
            id: 1,
            generation: 1,
        },
        followers: vec![
            Entity {
                id: 2,
                generation: 1,
            },
            Entity {
                id: 3,
                generation: 701,
            },
            Entity {
                id: 4,
                generation: 1,
            },
        ],
        mappings: vec![(
            23,
            Entity {
                id: 23,
                generation: 12,
            },
        )]
        .into_iter()
        .collect(),
    };
    let s = reflect_ref(&mut val).to_json();
    println!("{}", &s);
    let val2 = from_json::<Container>(&s);
    dbg!(&val2);
    assert_eq!(val, val2);
}

#[derive(PartialEq, Debug, Quicksilver)]
enum ContainerEnum {
    One(#[quicksilver(proxy(Entity, EntityWrapper))] Entity),
    More {
        #[quicksilver(proxy(Entity, EntityWrapper))]
        followers: Vec<Entity>,
    },
}

#[test]
fn subst_roundtrip_enum_one() {
    let mut val = ContainerEnum::One(Entity {
        id: 1,
        generation: 1,
    });
    let s = reflect_ref(&mut val).to_json();
    println!("{}", &s);
    let val2 = from_json::<ContainerEnum>(&s);
    dbg!(&val2);
    assert_eq!(val, val2);
}

#[test]
fn subst_roundtrip_enum_more() {
    let mut val = ContainerEnum::More {
        followers: vec![
            Entity {
                id: 2,
                generation: 1,
            },
            Entity {
                id: 3,
                generation: 701,
            },
            Entity {
                id: 4,
                generation: 1,
            },
        ],
    };
    let s = reflect_ref(&mut val).to_json();
    println!("{}", &s);
    let val2 = from_json::<ContainerEnum>(&s);
    dbg!(&val2);
    assert_eq!(val, val2);
}
