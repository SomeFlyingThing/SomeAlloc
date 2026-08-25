use std::{io, os::raw::c_void, process::Output, ptr::NonNull};

use libc::{MAP_ANONYMOUS, MAP_FAILED, MAP_PRIVATE, PROT_READ, PROT_WRITE, mmap, useconds_t};

#[derive(Clone)]
pub struct Block {
    mem: Option<NonNull<c_void>>,
    current_size: usize,
    origin_size: usize,
    is_free: bool,
    next: Option<NonNull<Block>>,
}
impl Block {
    pub const fn current_size(&self) -> usize {
        self.current_size
    }
    #[inline]
    pub const fn size(&self) -> usize {
        self.origin_size
    }
    #[inline]
    pub const fn is_free(&self) -> bool {
        self.is_free
    }
    #[inline]
    pub const fn next(&self) -> Option<NonNull<Block>> {
        self.next
    }
    #[inline]
    pub const fn start_of_mem(&self) -> *mut u8 {
        let start = self.mem.unwrap().as_ptr().cast::<u8>();

        unsafe { start.add(size_of::<Block>()) }
    }
    #[inline]
    pub fn peek(&self) -> Option<Block> {
        unsafe {
            let next = self.next?;
            let next = next.read();
            Some(next)
        }

        
    }
}

impl Block {
    fn reading(ptr: NonNull<u8>) -> Block {
        let ptr = ptr.as_ptr().cast::<Block>();

        unsafe { ptr.read() }
    }
}

const MORE: usize = 500;

pub fn map(current_size: usize) -> NonNull<Block> {
    let origin_size = current_size + MORE;
    let ptr = unsafe { mmap(std::ptr::null_mut(), origin_size, PROT_READ | PROT_WRITE, MAP_PRIVATE | MAP_ANONYMOUS, -1, 0) };

    if ptr == MAP_FAILED {
        panic!("map_failed")
    }

    let block_ptr = ptr.cast::<Block>();

    unsafe {
        block_ptr.write(Block {
            mem: NonNull::new(ptr),
            current_size,
            origin_size,

            is_free: false,
            next: None,
        });
    };

    NonNull::new(block_ptr).unwrap()
}

fn start_loop(start_block: &Block, size: usize) {
    let mut current_block = start_block;

    // or we have enouth space or we are partitioned and not partitioned we have enouth space
    while current_block.current_size < size || current_block.origin_size > size && current_block.current_size != current_block.origin_size  {
        current_block = match current_block.peek(){
            Some(value) => *value.clone(),
            None => break,
        }
    }
}

#[test]
fn block_layout_matches_exp() {
    assert_eq!(size_of::<Block>(), 32);
}
