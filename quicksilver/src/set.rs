use std::collections::HashSet;
use std::marker::PhantomData;

use std::hash::Hash;

use crate::reflections::ValueReflection;
use crate::reflections_ref::reflect_value_ref;
use crate::{Quicksilver, Type};

#[derive(Debug)]
pub struct HSVtable {
    /// creates the HashSet of current Type at the pointer coordinate
    pub new_at: unsafe fn(ptr: *mut u8),
    /// adds element to hashmap
    /// element pointers need to be created with Box::into_raw
    pub fill_with: unsafe fn(ptr: *mut u8, element_ptr: *mut u8),
    /// returns all elements in HashSet in whatever iteration order it sees fit
    pub get_elements_ref: unsafe fn(ptr: *const u8) -> Vec<ValueReflection<'static>>,
}

pub struct HSVtableCreator<T> {
    _phantom: PhantomData<T>,
}

impl<T> HSVtableCreator<T>
where
    T: Eq,
    T: Hash,
    T: Quicksilver,
{
    pub const VTABLE: HSVtable = HSVtable {
        new_at: Self::new_at,
        fill_with: Self::fill_with,
        get_elements_ref: Self::get_elements_ref,
    };

    unsafe fn new_at(ptr: *mut u8) {
        let v: HashSet<T> = HashSet::new();
        let ptr = ptr as *mut HashSet<T>;
        unsafe {
            ptr.write(v);
        }
    }

    unsafe fn fill_with(ptr: *mut u8, element_ptr: *mut u8) {
        let ptr = ptr as *mut HashSet<T>;
        let element_ptr = element_ptr as *mut T;
        unsafe {
            let val = &mut *ptr;
            let e: T = *Box::from_raw(element_ptr);
            val.insert(e);
        }
    }

    unsafe fn get_elements_ref(ptr: *const u8) -> Vec<ValueReflection<'static>> {
        let ptr = ptr as *mut HashSet<T>;
        let mut result = Vec::new();
        unsafe {
            let val = &*ptr;
            for el in val.iter() {
                result.push(reflect_value_ref(el as *const T as *const u8, &T::MIRROR))
            }
        }
        result
    }
}

#[repr(C)]
pub struct HSReflection<'a> {
    pub element: &'a Type,
    pub ptr: *mut u8,
    pub vtable: &'a HSVtable,
    pub skip: bool,
}

impl HSReflection<'_> {
    pub fn get_elements_ref(&self) -> Vec<ValueReflection<'_>> {
        unsafe { (self.vtable.get_elements_ref)(self.ptr) }
    }
}
