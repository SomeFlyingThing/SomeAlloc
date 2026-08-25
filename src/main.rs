use std::{intrinsics::black_box, ptr::NonNull};

use libc::{SCHED_FLAG_KEEP_POLICY, mmap};

use crate::block::{Block, map};

mod block;

fn main() {
    println!("Hello, world!");
}

fn alloc(size: usize) -> *mut u8 {
    unsafe {
        static mut START: Option<NonNull<Block>> = None;

        if START.is_none() {
            let block = map(size);

            START = Some(block);

            return START.unwrap().cast::<u8>().as_ptr();
        } else {
            let start = START.unwrap().read();
            
            
        }
    }

    
    
}
