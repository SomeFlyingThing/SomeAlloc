use std::{
    ffi::c_char,
    mem::{align_of, size_of},
    os::raw::c_void,
    ptr::NonNull,
};

use libc::{MAP_ANONYMOUS, MAP_FAILED, MAP_PRIVATE, PROT_READ, PROT_WRITE, mmap};

pub struct Block {
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
    pub const fn origin_size(&self) -> usize {
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
    pub const fn start_of_mem(block: NonNull<Self>) -> *mut u8 {
        unsafe { block.as_ptr().add(1).cast::<u8>() }
    }

    #[inline]
    pub fn peek(&self) -> Option<&Block> {
        unsafe {
            let next = self.next?;
            Some(next.as_ref())
        }
    }
    #[inline]
    pub fn mut_peek(&mut self) -> Option<&mut Block> {
        unsafe {
            let mut next = self.next?;
            Some(next.as_mut())
        }
    }

    #[inline]
    pub fn free(&mut self) {
        self.is_free = true;
    }
}

const MORE: usize = 500;

pub fn map(current_size: usize) -> NonNull<Block> {
    let current_size = align_size(current_size);
    let origin_size = current_size.checked_add(MORE).expect("allocation size overflow");
    let mapped_size = size_of::<Block>().checked_add(origin_size).expect("allocation size overflow");
    let ptr = unsafe { mmap(std::ptr::null_mut(), mapped_size, PROT_READ | PROT_WRITE, MAP_PRIVATE | MAP_ANONYMOUS, -1, 0) };

    if ptr == MAP_FAILED {
        panic!("map_failed")
    }

    let block_ptr = NonNull::new(ptr.cast::<Block>()).unwrap();
    unsafe {
        block_ptr.as_ptr().write(Block {
            current_size: origin_size,
            origin_size,

            is_free: true,
            next: None,
        });

        use_block(block_ptr, current_size);
    };

    block_ptr
}

const fn align_size(size: usize) -> usize {
    let alignment = align_of::<Block>();
    let remainder = size % alignment;

    if remainder == 0 {
        size
    } else {
        size.checked_add(alignment - remainder).expect("allocation size overflow")
    }
}

unsafe fn separate(wanted_size: usize, mut block_ptr: NonNull<Block>) -> bool {
    let metadata_size = size_of::<Block>();
    let wanted_size = align_size(wanted_size);
    let minimum_rest = align_of::<Block>();
    let Some(size_with_metadata) = wanted_size.checked_add(metadata_size) else {
        return false;
    };
    let Some(minimum_size) = size_with_metadata.checked_add(minimum_rest) else {
        return false;
    };

    let block = unsafe { block_ptr.as_mut() };

    if block.current_size < minimum_size {
        return false;
    }

    let rest = block.current_size - size_with_metadata;

    let old_next = block.next;

    // Put the next header immediately after this block's usable memory.
    let new_block_ptr = unsafe { Block::start_of_mem(block_ptr).add(wanted_size).cast::<Block>() };

    unsafe {
        new_block_ptr.write(Block {
            current_size: rest,
            origin_size: rest,
            is_free: true,
            next: old_next,
        });
    }

    block.current_size = wanted_size;
    block.next = NonNull::new(new_block_ptr);

    true
}

unsafe fn use_block(mut block_ptr: NonNull<Block>, wanted_size: usize) {
    unsafe {
        separate(wanted_size, block_ptr);
        block_ptr.as_mut().is_free = false;
    }
}

unsafe fn return_origin(mut block_ptr: NonNull<Block>) {
    let block = unsafe { block_ptr.as_mut() };

    if !block.is_free {
        return;
    }

    while block.current_size < block.origin_size {
        let Some(mut next_ptr) = block.next else {
            break;
        };
        let next = unsafe { next_ptr.as_mut() };
        let expected_next = Block::start_of_mem(block_ptr).wrapping_add(block.current_size).cast::<Block>();

        if !next.is_free || next_ptr.as_ptr() != expected_next {
            break;
        }

        let Some(joined_size) = block.current_size.checked_add(size_of::<Block>()).and_then(|size| size.checked_add(next.current_size)) else {
            break;
        };

        if joined_size > block.origin_size {
            break;
        }

        block.current_size = joined_size;
        block.next = next.next;
    }
}

unsafe fn start_loop(start_block: NonNull<Block>, size: usize) -> Option<NonNull<Block>> {
    let wanted_size = align_size(size);
    let mut current_block = start_block;

    loop {
        let block = unsafe { current_block.as_ref() };

        if block.is_free && block.current_size >= wanted_size {
            unsafe { use_block(current_block, wanted_size) };
            return Some(current_block);
        }

        current_block = block.next?;
    }
}
pub fn map_n_zero(current_size: usize) -> NonNull<Block> {
    let current_size = align_size(current_size);
    let origin_size = current_size.checked_add(MORE).expect("allocation size overflow");
    let mapped_size = size_of::<Block>().checked_add(origin_size).expect("allocation size overflow");
    let ptr = unsafe { mmap(std::ptr::null_mut(), mapped_size, PROT_READ | PROT_WRITE, MAP_PRIVATE | MAP_ANONYMOUS, -1, 0) };

    if ptr == MAP_FAILED {
        panic!("map_failed")
    }

    let block_ptr = NonNull::new(ptr.cast::<Block>()).unwrap();
    unsafe {
        block_ptr.as_ptr().write(Block {
            current_size: origin_size,
            origin_size,

            is_free: true,
            next: None,
        });

        use_block(block_ptr, current_size);
    };

    zerod_block(block_ptr, current_size);

    block_ptr
}

fn zerod_block(block: NonNull<Block>, size: usize) {
    let mem = unsafe { block.add(1).cast::<u8>().as_ptr() };

    unsafe {
        std::ptr::write_bytes(mem, 0, size);
    }
}
/// Searches a mapped block list for reusable memory.
///
/// Safety
///
/// start_block must point to a live list created by map, and the caller
/// must have exclusive access to that list for the duration of this call.
pub unsafe fn allocate(start_block: NonNull<Block>, size: usize) -> Option<NonNull<Block>> {
    unsafe { start_loop(start_block, size) }
}

#[test]
fn block_layout_matches_exp() {
    assert_eq!(size_of::<Block>(), 32);
}

#[test]
fn map_keeps_the_header_outside_the_payload() {
    let block_ptr = map(13);
    let block = unsafe { block_ptr.as_ref() };

    assert_eq!(Block::start_of_mem(block_ptr), unsafe { block_ptr.as_ptr().add(1).cast::<u8>() });
    assert_eq!(block.current_size(), 16);
    assert!(!block.is_free());
    assert_eq!(Block::start_of_mem(block.next().unwrap()) as usize % align_of::<Block>(), 0);
}

#[test]
fn allocation_uses_a_small_remainder_without_splitting_it() {
    let mut block_ptr = map(1);
    let first_rest = unsafe { block_ptr.as_mut().mut_peek().unwrap() };
    first_rest.current_size = size_of::<Block>() + align_of::<Block>() - 1;
    let first_rest_size = first_rest.current_size;

    let allocated = unsafe { start_loop(NonNull::from(&mut *first_rest), 1).unwrap().as_ref() };

    assert_eq!(allocated.current_size, first_rest_size);
    assert!(allocated.next.is_none());
}

#[test]
fn allocation_skips_blocks_that_are_not_free() {
    let mut block_ptr = map(8);
    let root = unsafe { block_ptr.as_mut() };
    let expected = root.next.unwrap();

    assert_eq!(unsafe { start_loop(NonNull::from(root), 8) }, Some(expected));
}

#[test]
fn return_origin_only_joins_adjacent_free_blocks() {
    let mut block_ptr = map(8);
    let root = unsafe { block_ptr.as_mut() };

    root.is_free = true;
    root.mut_peek().unwrap().is_free = false;
    unsafe { return_origin(NonNull::from(&mut *root)) };
    assert!(root.next.is_some());

    root.mut_peek().unwrap().is_free = true;
    unsafe { return_origin(NonNull::from(&mut *root)) };
    assert_eq!(root.current_size, root.origin_size);
    assert!(root.next.is_none());
}
