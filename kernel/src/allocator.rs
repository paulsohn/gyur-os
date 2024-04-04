//! The allocator. Page manager is separately defined.

extern crate alloc;

// use crate::pgmgr::KERNEL_PAGE_SIZE;

use core::ptr::NonNull;
use core::ops::Deref;
use core::alloc::{GlobalAlloc, Layout};
// use alloc::alloc::{Allocator, AllocError};

use spin::mutex::Mutex;
use linked_list_allocator::Heap; // `LockedHeap` uses lock in the `spinning_top` crate. We will use `spin` instead.

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
            .map_or(core::ptr::null_mut(), |allocation| allocation.as_ptr())
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        self.0
            .lock()
            .deallocate(NonNull::new_unchecked(ptr), layout)
    }
}

// pub type Heap = BumpHeap;

// // bump allocator
// #[repr(C)]
// pub struct BumpHeap {
//     base: *mut u8,
//     limit: usize,
//     offset: usize,
// }
// impl BumpHeap {
//     pub const fn empty() -> Self {
//         Self {
//             base: core::ptr::null_mut(),
//             limit: 0,
//             offset: 0,
//         }
//     }

//     pub fn init(&mut self, base: *mut u8, limit: usize) {
//         self.base = base;
//         self.limit = limit;
//         self.offset = 0;
//     }

//     fn round_up_to(value: usize, alignment: usize) -> usize {
//         (value + alignment - 1) & !(alignment - 1)
//     }

//     pub fn allocate_first_fit(&mut self, layout: Layout) -> Result<NonNull<u8>, ()> {
//         self.offset = Self::round_up_to(self.offset, layout.align());

//         if (self.offset + layout.size()) / KERNEL_PAGE_SIZE != self.offset / KERNEL_PAGE_SIZE {
//             self.offset = Self::round_up_to(self.offset, KERNEL_PAGE_SIZE);
//         }
        
//         let result_offset = self.offset;
//         let next_offset = self.offset + layout.size();

//         if next_offset > self.limit {
//             Err(())
//         } else {
//             // log::info!("alloc {:?}", unsafe{ self.base.byte_add(result_offset) });
//             self.offset = next_offset;
//             Ok(unsafe {
//                 NonNull::new_unchecked(self.base.byte_add(result_offset))
//             })
//         }
//     }

//     pub fn deallocate(&mut self, ptr: NonNull<u8>, layout: Layout) {
//         // do nothing. this is a bump allocator
//     }
// }
// unsafe impl Sync for BumpHeap {}
// unsafe impl Send for BumpHeap {}