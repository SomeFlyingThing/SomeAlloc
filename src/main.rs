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

#[cfg(test)]
mod test {
    use std::ptr::{self, copy_nonoverlapping, slice_from_raw_parts};

    use super::*;

    #[test]
    fn test_alloc() {
        let mem = alloc(40);

        let mut read = [1u8; 40];

        unsafe {
            ptr::write_bytes(mem, 3, 40);

            copy_nonoverlapping(mem, read.as_mut_ptr(), 40);
        }

        let result = [3u8; 40];

        assert!(result == read);
    }

    #[test]
    fn test_calloc() {
        let mem = calloc(40);

        let read: &[u8] = unsafe { std::slice::from_raw_parts(mem, 40) };

        assert_eq!(read, &[0u8; 40]);
    }
}
