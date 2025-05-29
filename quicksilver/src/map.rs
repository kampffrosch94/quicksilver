use std::{collections::HashMap, marker::PhantomData};

use crate::{
    FieldReflection, Reflectable, Reflection, StructReflection, Type, ValueReflection,
    reflect_value,
};
use std::hash::Hash;

#[derive(Debug)]
pub struct HMVtable {
    /// creates the HashMap of current Type at the pointer coordinate
    pub new_at: unsafe fn(ptr: *mut u8),
    /// adds elements to hashmap
    /// elements needs to be a pointer to a `Vec<HashMapEntry<K,V>>`
    pub fill_with: unsafe fn(ptr: *mut u8, elements: *mut u8),
    /// returns all elements in HashMap in whatever iteration order it sees fit
    pub get_elements: unsafe fn(ptr: *mut u8) -> Vec<StructReflection<'static>>,
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

impl<Key, Value> HMEntryView<Key, Value>
where
    Key: Reflectable,
    Value: Reflectable,
{
    /// unsafe because caller needs to uphold lifetime invariant
    unsafe fn reflect(self) -> StructReflection<'static> {
        StructReflection {
            name: "HMEntry",
            fields: unsafe {
                vec![
                    FieldReflection {
                        name: "key",
                        value: reflect_value(self.key as *mut u8, &Key::TYPE),
                    },
                    FieldReflection {
                        name: "value",
                        value: reflect_value(self.value as *mut u8, &Key::TYPE),
                    },
                ]
            },
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
    Key: Reflection,
    Value: Reflection,
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

    unsafe fn get_elements(ptr: *mut u8) -> Vec<StructReflection<'static>> {
        let ptr = ptr as *mut HashMap<Key, Value>;
        let mut result = Vec::new();
        unsafe {
            let val = &mut *ptr;
            for (key, value) in val.iter_mut() {
                let el = HMEntryView {
                    key: key as *const Key,
                    value: value as *mut Value,
                };
                result.push(el.reflect());
            }
        }
        result
    }
}
