use crate::block::Block;

fn free(begin_mem: *mut u8) -> Option<()> {
    let ptr = begin_mem.cast::<Block>();

    let ptr = unsafe { ptr.sub(1) };

    let block = unsafe { &mut *ptr };

    block.free();

    
    Some(())
}
