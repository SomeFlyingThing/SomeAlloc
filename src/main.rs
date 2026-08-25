use std::ptr::NonNull;

use crate::block::{Block, allocate, map};

mod block;

fn main() {
    println!("Hello, world!");
}

fn alloc(size: usize) -> *mut u8 {
    unsafe {
        static mut START: Option<NonNull<Block>> = None;

        let Some(start) = START else {
            let block = map(size);

            START = Some(block);

            return Block::start_of_mem(block);
        };

        allocate(start, size).map_or(std::ptr::null_mut(), Block::start_of_mem)
    }
}
