//! The allocator. Page manager is separately defined.

extern crate alloc;

use core::ptr::NonNull;
use core::ops::Deref;
use core::alloc::{GlobalAlloc, Layout};
// use alloc::alloc::{Allocator, AllocError};

use spin::mutex::Mutex;
use linked_list_allocator::Heap; // `LockedHeap` uses lock in the `spinning_top` crate. We will use `spin` instead.

// use shared::uefi_memory::PAGE_SIZE as UEFI_PAGE_SIZE;

// const KB: usize = 0x400;
// const MB: usize = KB * KB;
// const GB: usize = MB * KB;

// const KERNEL_PAGE_SIZE: usize = 4 * KB;

// #[derive(Clone, Copy, Debug)]
// #[repr(transparent)]
// struct FrameID(pub usize);

// #[derive(Clone, Copy)]
// #[repr(C, align(0x1000))]
// struct Page([u8; KERNEL_PAGE_SIZE]);
// impl Page {
//     pub const fn new() -> Self {
//         Self([0; KERNEL_PAGE_SIZE])
//     }
// }


#[repr(transparent)]
pub struct GlobalHeap(Mutex<Heap>);
impl GlobalHeap {
    pub const fn empty() -> Self {
        Self(Mutex::new(Heap::empty()))
    }

    pub unsafe fn init(&self, heap_bottom: *mut u8, heap_size: usize) {
        self.0.lock().init(heap_bottom, heap_size);
    }
}

impl Deref for GlobalHeap {
    type Target = Mutex<Heap>;

    fn deref(&self) -> &Mutex<Heap> {
        &self.0
    }
}

unsafe impl GlobalAlloc for GlobalHeap {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        self.0
            .lock()
            .allocate_first_fit(layout)
            .ok()
            .map_or(core::ptr::null_mut(), |allocation| allocation.as_ptr())
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        self.0
            .lock()
            .deallocate(NonNull::new_unchecked(ptr), layout)
    }
}