use std::{collections::HashMap, marker::PhantomData};

use std::hash::Hash;

use crate::reflections::{FieldReflection, StructReflection, reflect_value};
use crate::reflections_ref::reflect_value_ref;
use crate::{Reflectable, Type};

#[derive(Debug)]
pub struct HMVtable {
    /// creates the HashMap of current Type at the pointer coordinate
    pub new_at: unsafe fn(ptr: *mut u8),
    /// adds element to hashmap
    /// element pointers need to be created with Box::into_raw
    pub fill_with: unsafe fn(ptr: *mut u8, key_ptr: *mut u8, value_ptr: *mut u8),
    /// returns all elements in HashMap in whatever iteration order it sees fit
    pub get_elements: unsafe fn(ptr: *mut u8) -> Vec<StructReflection<'static>>,
    /// returns all elements in HashMap in whatever iteration order it sees fit
    pub get_elements_ref: unsafe fn(ptr: *const u8) -> Vec<StructReflection<'static>>,
}

/// pointers to Keys and Values inside the HashMap
pub struct HMEntryView {
    /// never ever mutate the key itself
    pub key: *mut u8,
    pub value: *mut u8,
    pub key_t: &'static Type,
    pub value_t: &'static Type,
}

impl HMEntryView {
    /// unsafe because caller needs to uphold lifetime invariant
    unsafe fn reflect(self) -> StructReflection<'static> {
        StructReflection {
            name: "HMEntry",
            fields: unsafe {
                vec![
                    FieldReflection {
                        name: "key",
                        value: reflect_value_ref(self.key, self.key_t),
                    },
                    FieldReflection {
                        name: "value",
                        value: reflect_value(self.value, self.value_t),
                    },
                ]
            },
        }
    }

    unsafe fn reflect_ref(self) -> StructReflection<'static> {
        StructReflection {
            name: "HMEntry",
            fields: unsafe {
                vec![
                    FieldReflection {
                        name: "key",
                        value: reflect_value_ref(self.key, self.key_t),
                    },
                    FieldReflection {
                        name: "value",
                        value: reflect_value_ref(self.value, self.value_t),
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
    Key: Reflectable,
    Value: Reflectable,
{
    pub const VTABLE: HMVtable = HMVtable {
        new_at: Self::new_at,
        fill_with: Self::fill_with,
        get_elements: Self::get_elements,
        get_elements_ref: Self::get_elements_ref,
    };

    unsafe fn new_at(ptr: *mut u8) {
        let v: HashMap<Key, Value> = HashMap::new();
        let ptr = ptr as *mut HashMap<Key, Value>;
        unsafe {
            ptr.write(v);
        }
    }

    unsafe fn fill_with(ptr: *mut u8, key_ptr: *mut u8, value_ptr: *mut u8) {
        let ptr = ptr as *mut HashMap<Key, Value>;
        let key_ptr = key_ptr as *mut Key;
        let value_ptr = value_ptr as *mut Value;
        unsafe {
            let hm = &mut *ptr;
            let key: Key = *Box::from_raw(key_ptr);
            let value = *Box::from_raw(value_ptr);
            hm.insert(key, value);
        }
    }

    unsafe fn get_elements(ptr: *mut u8) -> Vec<StructReflection<'static>> {
        let ptr = ptr as *mut HashMap<Key, Value>;
        let mut result = Vec::new();
        unsafe {
            let val = &mut *ptr;
            for (key, value) in val.iter_mut() {
                let el = HMEntryView {
                    key: key as *const Key as *mut u8,
                    value: value as *mut Value as *mut u8,
                    key_t: &Key::TYPE,
                    value_t: &Value::TYPE,
                };
                result.push(el.reflect());
            }
        }
        result
    }

    unsafe fn get_elements_ref(ptr: *const u8) -> Vec<StructReflection<'static>> {
        let ptr = ptr as *const HashMap<Key, Value>;
        let mut result = Vec::new();
        unsafe {
            let val = &*ptr;
            for (key, value) in val.iter() {
                let el = HMEntryView {
                    key: key as *const Key as *mut u8,
                    value: value as *const Value as *mut u8,
                    key_t: &Key::TYPE,
                    value_t: &Value::TYPE,
                };
                result.push(el.reflect_ref());
            }
        }
        result
    }
}

#[repr(C)]
pub struct HMReflection<'a> {
    pub key: &'a Type,
    pub value: &'a Type,
    pub ptr: *mut u8,
    pub vtable: &'a HMVtable,
    pub _phantom: PhantomData<&'a u8>,
}

impl HMReflection<'_> {
    pub fn get_elements(&self) -> Vec<StructReflection<'_>> {
        unsafe { (self.vtable.get_elements)(self.ptr) }
    }

    pub fn get_elements_ref(&self) -> Vec<StructReflection<'_>> {
        unsafe { (self.vtable.get_elements_ref)(self.ptr) }
    }
}
