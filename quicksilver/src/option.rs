use std::marker::PhantomData;

use crate::{
    Quicksilver, Type,
    reflections::{ValueReflection, reflect_value},
    reflections_ref::reflect_value_ref,
};

#[derive(Debug)]
pub struct OptionVtable {
    /// creates an empty option at pointer
    pub new_at: unsafe fn(ptr: *mut u8),
    /// sets value
    /// value pointer needs to be created with Box::into_raw
    pub set: unsafe fn(ptr: *mut u8, value: Option<*mut u8>),
    /// get element
    pub get_elem: unsafe fn(ptr: *mut u8) -> Option<*mut u8>,
    /// get element immutably
    pub get_elem_ref: unsafe fn(ptr: *const u8) -> Option<*const u8>,
}

pub struct OptionVtableCreator<T> {
    _phantom: PhantomData<T>,
}

impl<T> OptionVtableCreator<T>
where
    T: Quicksilver,
{
    pub const VTABLE: OptionVtable = OptionVtable {
        new_at: Self::new_at,
        set: Self::set,
        get_elem: Self::get_elem,
        get_elem_ref: Self::get_elem_ref,
    };

    unsafe fn new_at(ptr: *mut u8) {
        let o: Option<T> = None;
        let ptr = ptr as *mut Option<T>;
        unsafe {
            ptr.write(o);
        }
    }

    unsafe fn set(ptr: *mut u8, val_ptr: Option<*mut u8>) {
        let ptr = ptr as *mut Option<T>;
        if let Some(val_ptr) = val_ptr {
            let val_ptr = val_ptr as *mut T;
            unsafe {
                let o: &mut Option<T> = &mut *ptr;
                let val: T = *Box::from_raw(val_ptr);
                *o = Some(val);
            }
        } else {
            unsafe {
                let o: &mut Option<T> = &mut *ptr;
                *o = None;
            }
        }
    }

    unsafe fn get_elem(ptr: *mut u8) -> Option<*mut u8> {
        let ptr = ptr as *mut Option<T>;
        unsafe {
            let o: &mut Option<T> = &mut *ptr;
            if let Some(inner) = o {
                Some(inner as *mut T as *mut u8)
            } else {
                None
            }
        }
    }

    unsafe fn get_elem_ref(ptr: *const u8) -> Option<*const u8> {
        let ptr = ptr as *const Option<T>;
        unsafe {
            let o: &Option<T> = &*ptr;
            if let Some(inner) = o {
                Some(inner as *const T as *const u8)
            } else {
                None
            }
        }
    }
}

#[repr(C)]
pub struct OptionReflection<'a> {
    pub element: &'a Type,
    /// points to the option
    pub ptr: *mut u8,
    pub vtable: &'a OptionVtable,
    pub skip: bool,
}

impl OptionReflection<'_> {
    pub fn get(&mut self) -> Option<ValueReflection<'_>> {
        unsafe {
            let ptr = (self.vtable.get_elem)(self.ptr);
            ptr.map(|it| reflect_value(it, &self.element))
        }
    }

    pub fn get_ref(&self) -> Option<ValueReflection<'_>> {
        unsafe {
            let ptr = (self.vtable.get_elem_ref)(self.ptr);
            ptr.map(|it| reflect_value_ref(it, &self.element))
        }
    }
}

pub struct EmptyOptionVtableCreator<T> {
    _phantom: PhantomData<T>,
}

impl<T> EmptyOptionVtableCreator<T> {
    pub const VTABLE: OptionVtable = OptionVtable {
        new_at: Self::new_at,
        set: empty_set,
        get_elem: empty_get_elem,
        get_elem_ref: empty_get_elem_ref,
    };

    unsafe fn new_at(ptr: *mut u8) {
        let o: Option<T> = None;
        let ptr = ptr as *mut Option<T>;
        unsafe {
            ptr.write(o);
        }
    }
}

unsafe fn empty_set(_ptr: *mut u8, _val_ptr: Option<*mut u8>) {
    panic!("Not supported on skipped fields");
}

unsafe fn empty_get_elem(_ptr: *mut u8) -> Option<*mut u8> {
    panic!("Not supported on skipped fields");
}

unsafe fn empty_get_elem_ref(_ptr: *const u8) -> Option<*const u8> {
    panic!("Not supported on skipped fields");
}
