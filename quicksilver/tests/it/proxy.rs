use std::collections::HashMap;

use quicksilver::Quicksilver;

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
}

#[test]
fn subst_roundtrip_enum() {
}
