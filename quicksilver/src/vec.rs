use std::marker::PhantomData;

use crate::{
    Quicksilver, Type,
    reflections::{ValueReflection, reflect_value},
    reflections_ref::reflect_value_ref,
};

#[derive(Debug)]
pub struct VecVtable {
    /// creates the Vec of current Type at the pointer coordinate
    /// returns pointer to the first element
    pub new_at: unsafe fn(ptr: *mut u8, capacity: usize) -> *mut u8,
    /// set len
    /// same as Vec::set_len, just type erased
    pub set_len: unsafe fn(ptr: *mut u8, len: usize),
    /// grows Vec to new capacity, like Vec::reserve
    /// returns pointer to the first element
    pub reserve: unsafe fn(ptr: *mut u8, additonal: usize) -> *mut u8,
    /// get used length
    pub get_len: unsafe fn(ptr: *mut u8) -> usize,
    /// get element at index
    pub get_elem: unsafe fn(ptr: *mut u8, index: usize) -> *mut u8,
    /// get element at index immutably
    pub get_elem_ref: unsafe fn(ptr: *const u8, index: usize) -> *const u8,
}

pub struct VecVtableCreator<T> {
    _phantom: PhantomData<T>,
}

impl<T> VecVtableCreator<T>
where
    T: Quicksilver,
{
    pub const VTABLE: VecVtable = VecVtable {
        new_at: Self::new_at,
        set_len: Self::set_len,
        reserve: Self::reserve,
        get_len: Self::get_len,
        get_elem: Self::get_elem,
        get_elem_ref: Self::get_elem_ref,
    };

    unsafe fn new_at(ptr: *mut u8, capacity: usize) -> *mut u8 {
        let mut v: Vec<T> = Vec::with_capacity(capacity);
        let ptr = ptr as *mut Vec<T>;
        unsafe {
            let out = v.as_mut_ptr();
            ptr.write(v);
            out as *mut u8
        }
    }

    unsafe fn set_len(ptr: *mut u8, len: usize) {
        let ptr = ptr as *mut Vec<T>;
        unsafe {
            let val = &mut *ptr;
            val.set_len(len);
        }
    }

    unsafe fn reserve(ptr: *mut u8, additional: usize) -> *mut u8 {
        let ptr = ptr as *mut Vec<T>;
        unsafe {
            let val = &mut *ptr;
            val.reserve(additional);
            val.as_mut_ptr() as *mut u8
        }
    }

    unsafe fn get_len(ptr: *mut u8) -> usize {
        let ptr = ptr as *mut Vec<T>;
        unsafe {
            let val = &*ptr;
            val.len()
        }
    }

    unsafe fn get_elem(ptr: *mut u8, index: usize) -> *mut u8 {
        let ptr = ptr as *mut Vec<T>;
        unsafe {
            let val = &mut *ptr;
            let el = &raw mut val[index];
            el as *mut u8
        }
    }

    unsafe fn get_elem_ref(ptr: *const u8, index: usize) -> *const u8 {
        let ptr = ptr as *const Vec<T>;
        unsafe {
            let val = &*ptr;
            let el = &raw const val[index];
            el as *const u8
        }
    }
}

#[repr(C)]
pub struct VecReflection<'a> {
    pub element: &'a Type,
    pub ptr: *mut u8,
    pub vtable: &'a VecVtable,
    pub skip: bool,
    pub _phantom: PhantomData<&'a u8>,
}

impl VecReflection<'_> {
    pub fn len(&self) -> usize {
        unsafe { (self.vtable.get_len)(self.ptr) }
    }

    pub fn get(&mut self, index: usize) -> ValueReflection {
        unsafe {
            let ptr = (self.vtable.get_elem)(self.ptr, index);
            reflect_value(ptr, &self.element)
        }
    }

    pub fn get_ref(&self, index: usize) -> ValueReflection {
        unsafe {
            let ptr = (self.vtable.get_elem_ref)(self.ptr, index);
            reflect_value_ref(ptr, &self.element)
        }
    }
}

pub struct EmptyVecVtableCreator<T> {
    _phantom: PhantomData<T>,
}

impl<T> EmptyVecVtableCreator<T> {
    pub const VTABLE: VecVtable = VecVtable {
        new_at: Self::new_at,
        set_len: empty_set_len,
        reserve: empty_reserve,
        get_len: empty_get_len,
        get_elem: empty_get_elem,
        get_elem_ref: empty_get_elem_ref,
    };

    unsafe fn new_at(ptr: *mut u8, capacity: usize) -> *mut u8 {
        let mut v: Vec<T> = Vec::with_capacity(capacity);
        let ptr = ptr as *mut Vec<T>;
        unsafe {
            let out = v.as_mut_ptr();
            ptr.write(v);
            out as *mut u8
        }
    }
}

unsafe fn empty_set_len(_ptr: *mut u8, _len: usize) {
    panic!("Not supported on skipped fields");
}

unsafe fn empty_reserve(_ptr: *mut u8, _additional: usize) -> *mut u8 {
    panic!("Not supported on skipped fields");
}

unsafe fn empty_get_len(_ptr: *mut u8) -> usize {
    panic!("Not supported on skipped fields");
}

unsafe fn empty_get_elem(_ptr: *mut u8, _index: usize) -> *mut u8 {
    panic!("Not supported on skipped fields");
}

unsafe fn empty_get_elem_ref(_ptr: *const u8, _index: usize) -> *const u8 {
    panic!("Not supported on skipped fields");
}
