
extern crate alloc;

use crate::pgmgr::KERNEL_PAGE_SIZE;
use crate::allocator::GlobalHeap;

use super::pgmgr::PAGE_MANAGER;

#[global_allocator]
static GLOBAL_HEAP: GlobalHeap = GlobalHeap::empty();

/// Heap frame count. set to 32 * 2MB.
const HEAP_FRAME_CNT: usize = 32 * 512; // 64 * 512 failed on QEMU

pub fn init() {
    unsafe {
        let heap_bottom = PAGE_MANAGER.lock()
            .allocate(HEAP_FRAME_CNT).unwrap()
            .addr() as *mut u8;

        log::info!("Heap bottom {:?}", heap_bottom);
        
        GLOBAL_HEAP.init(heap_bottom, HEAP_FRAME_CNT * KERNEL_PAGE_SIZE);
    }
}

/// Create an instance of global allocator.
pub fn global_allocator() -> alloc::alloc::Global {
    alloc::alloc::Global
}