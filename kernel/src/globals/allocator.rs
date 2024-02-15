
extern crate alloc;

use crate::allocator::GlobalHeap;

#[global_allocator]
static GLOBAL_HEAP: GlobalHeap = GlobalHeap::empty();

pub fn init(heap_bottom: *mut u8, heap_size: usize) {
    unsafe {
        GLOBAL_HEAP.init(heap_bottom, heap_size);
    }
}

/// Create an instance of global allocator.
pub fn global_allocator() -> alloc::alloc::Global {
    alloc::alloc::Global
}