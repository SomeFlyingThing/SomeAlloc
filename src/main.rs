use std::ptr::NonNull;

use crate::block::{Block, allocate, map, map_n_zero};

mod block;
mod free;

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

fn calloc(size: usize) -> *mut u8 {
    unsafe {
        static mut START: Option<NonNull<Block>> = None;

        let Some(start) = START else {
            let block = map_n_zero(size);

            START = Some(block);

            return Block::start_of_mem(block);
        };

        allocate(start, size).map_or(std::ptr::null_mut(), Block::start_of_mem)
    }
}
