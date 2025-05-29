use std::{collections::HashMap, marker::PhantomData};

use crate::{FieldReflection, Reflection, StructReflection, Type, ValueReflection, reflect_value};
use std::hash::Hash;

#[derive(Debug)]
pub struct HMVtable {
    /// creates the HashMap of current Type at the pointer coordinate
    /// returns pointer to the first element
    pub new_at: unsafe fn(ptr: *mut u8),
    pub fill_with: unsafe fn(ptr: *mut u8, elements: *mut u8),
    pub get_elements: unsafe fn(ptr: *mut u8) -> *mut u8,
}

pub struct HMEntry<Key, Value> {
    pub key: Key,
    pub value: Value,
}

/// pointers to Keys and Values inside the HashMap
pub struct HMEntryView<Key, Value> {
    /// never ever mutate the key itself
    key: *const Key,
    value: *mut Value,
}

impl<Key, Value> HMEntryView<Key, Value> {
    /// unsafe because caller needs to uphold lifetime invariant
    unsafe fn reflect(self) -> StructReflection<'static> {
        StructReflection {
            name: "HMEntry",
            fields: vec![
                FieldReflection {
                    name: "key",
                    value: todo!(),
                },
                FieldReflection {
                    name: "value",
                    value: todo!(),
                },
            ],
        }
    }
}

pub struct HMVtableCreator<Key, Value> {
    _phantom: PhantomData<(Key, Value)>,
}

impl<Key, Value> HMVtableCreator<Key, Value>
where
    Key: Eq,
    Key: Hash,
{
    pub const VTABLE: HMVtable = HMVtable {
        new_at: Self::new_at,
        fill_with: Self::fill_with,
        get_elements: Self::get_elements,
    };

    unsafe fn new_at(ptr: *mut u8) {
        let v: HashMap<Key, Value> = HashMap::new();
        let ptr = ptr as *mut HashMap<Key, Value>;
        unsafe {
            ptr.write(v);
        }
    }

    unsafe fn fill_with(ptr: *mut u8, elements: *mut u8) {
        let ptr = ptr as *mut HashMap<Key, Value>;
        let elements = elements as *mut Vec<HMEntry<Key, Value>>;
        unsafe {
            let hm = &mut *ptr;
            let elements = &mut *elements;
            for element in elements.drain(..) {
                hm.insert(element.key, element.value);
            }
        }
    }

    unsafe fn get_elements(ptr: *mut u8) -> *mut u8 {
        let ptr = ptr as *mut HashMap<Key, Value>;
        let mut result: Vec<HMEntryView<Key, Value>> = Vec::new();
        unsafe {
            let val = &mut *ptr;
            for (key, value) in val.iter_mut() {
                let el = HMEntryView {
                    key: key as *const Key,
                    value: value as *mut Value,
                };
            }
        }
        // TODO: how to get values out in a type erased manner
        // maybe use a Reflected Value for the Vec?
        todo!()
    }
}
