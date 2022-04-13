use std::alloc::{alloc, dealloc, Layout};
use std::ptr;

pub struct Heaped<T> {
    pub raw_ptr: *mut u8,
    pub const_ptr: *const T,
    pub mut_ptr: *mut T,
    is_allocated: bool,
    is_filled: bool
}

impl<T> Heaped::<T> {
    pub fn new_with_value(value: T) -> Heaped::<T> {
        let mut heaped = Heaped {
            raw_ptr: ptr::null_mut(),
            const_ptr: ptr::null(),
            mut_ptr: ptr::null_mut(),
            is_allocated: false,
            is_filled: false
        };

        heaped.alloc();
        heaped.fill(value);

        return heaped;
    }

    fn alloc(&mut self){
        if self.is_allocated {
            panic!("Trying to allocate an already allocated Heaped!");
        }

        unsafe {
            let layout = Layout::new::<T>();

            self.raw_ptr = alloc(layout);
            self.const_ptr = self.raw_ptr as *const T;
            self.mut_ptr = self.raw_ptr as *mut T;
        }

        self.is_allocated = true;
    }

    pub fn fill(&mut self, value: T){
        if !self.is_allocated {
            panic!("Trying to fill a non-allocated Heaped!");
        }

        if self.is_filled {
            panic!("Trying to fill an already filled Heaped!");
        }

        unsafe {
            ptr::write(self.mut_ptr, value);
        }

        self.is_filled = true;
    }

    pub fn dealloc(&mut self){
        if !self.is_allocated {
            panic!("Trying to destroy a non-allocated Heaped!");
        }

        unsafe {
            let layout = Layout::new::<T>();
            dealloc(self.raw_ptr, layout);

            self.raw_ptr = ptr::null_mut();
            self.const_ptr = ptr::null();
            self.mut_ptr = ptr::null_mut();
        }

        self.is_allocated = false;
        self.is_filled = false;
    }
}

impl<T> Copy for Heaped<T> { }
impl<T> Clone for Heaped<T> {
    fn clone(&self) -> Heaped::<T>{
        Heaped::<T>{
            raw_ptr: self.raw_ptr,
            const_ptr: self.const_ptr,
            mut_ptr: self.mut_ptr,
            is_allocated: self.is_allocated,
            is_filled: self.is_filled
        }
    }
}

impl<T> PartialEq for Heaped<T> {
    fn eq(&self, other: &Self) -> bool {
        self.raw_ptr == other.raw_ptr
    }
}