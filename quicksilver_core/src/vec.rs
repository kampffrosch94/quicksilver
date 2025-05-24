use std::marker::PhantomData;

use crate::Reflection;

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
}

pub struct VecVtableCreator<T> {
    _phantom: PhantomData<T>,
}

pub trait TableAble {}

impl<T> TableAble for T where T: Reflection {}
impl TableAble for u32 {}
impl TableAble for i32 {}
impl TableAble for f32 {}
impl TableAble for String {}
impl<T> TableAble for Vec<T> where T: TableAble {}

impl<T> VecVtableCreator<T>
where
    T: TableAble,
{
    pub const VTABLE: VecVtable = VecVtable {
        new_at: Self::new_at,
        set_len: Self::set_len,
        reserve: Self::reserve,
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
}
