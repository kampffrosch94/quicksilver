use crate::option::EmptyOptionVtableCreator;
use crate::set::EmptyHSVtableCreator;
use crate::{HMType, Struct, Type, VecType, map::EmptyHMVtableCreator, vec::EmptyVecVtableCreator};
use crate::{HSType, OptionType};
use std::collections::{HashMap, HashSet};
use std::hash::Hash;

pub trait EmptyContainer {
    const EMPTY: Type;
}

const EMPTY_STRUCT: Type = Type::Struct(&Struct {
    size: 0,
    align: 0,
    name: "skipped",
    fields: &[],
});

impl<T> EmptyContainer for Vec<T> {
    const EMPTY: Type = Type::Vec(VecType {
        element: &EMPTY_STRUCT,
        vtable: EmptyVecVtableCreator::<T>::VTABLE,
        skip: true,
        size: size_of::<Self>(),
        align: align_of::<Self>(),
    });
}

impl<K, V> EmptyContainer for HashMap<K, V>
where
    K: Eq,
    K: Hash,
{
    const EMPTY: Type = Type::HashMap(HMType {
        vtable: EmptyHMVtableCreator::<K, V>::VTABLE,
        skip: true,
        key: &EMPTY_STRUCT,
        value: &EMPTY_STRUCT,
        size: size_of::<Self>(),
        align: align_of::<Self>(),
    });
}

impl<T> EmptyContainer for Option<T> {
    const EMPTY: Type = Type::Option(OptionType {
        element: &EMPTY_STRUCT,
        vtable: EmptyOptionVtableCreator::<T>::VTABLE,
        skip: true,
        size: size_of::<Self>(),
        align: align_of::<Self>(),
    });
}

impl<K> EmptyContainer for HashSet<K>
where
    K: Eq,
    K: Hash,
{
    const EMPTY: Type = Type::HashSet(HSType {
        vtable: EmptyHSVtableCreator::<K>::VTABLE,
        skip: true,
        element: &EMPTY_STRUCT,
        size: size_of::<Self>(),
        align: align_of::<Self>(),
    });
}
