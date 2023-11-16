extern crate alloc;

use core::ptr::NonNull;
use core::alloc::{GlobalAlloc, Layout};
use alloc::alloc::{Allocator, AllocError};

use spin::mutex::Mutex;

const PAGE_BYTES: usize = 0x1000;

#[derive(Clone, Copy)]
#[repr(C, align(0x1000))]
struct Page([u8; PAGE_BYTES]);
impl Page {
    const fn new() -> Self {
        Self([0; PAGE_BYTES])
    }
}

// bump allocator

#[repr(C)]
struct BumpArena<const N: usize> {
    arena: [Page; N],
    offset: Mutex<[usize; N]>,
} // only `offset` is subjected to modify.
impl<const N: usize> BumpArena<N> {
    const fn new() -> Self {
        Self {
            arena: [Page::new(); N],
            offset: Mutex::new([0; N]),
        }
    }

    fn round_up_to(value: usize, alignment: usize) -> usize {
        (value + alignment - 1) & !(alignment - 1)
    }

    unsafe fn alloc_mem(&self, size: usize, alignment: usize) -> *mut u8 {
        let mut offset = self.offset.lock();

        (0..N).find_map(|i| {
            let result_offset = Self::round_up_to(offset[i], alignment);

            let next_offset = result_offset + size;
            if next_offset < PAGE_BYTES {
                None
            } else {
                offset[i] = next_offset;
                let base = &self.arena[i]
                    as *const _
                    as *const u8 as *mut u8;
                
                Some(base.byte_add(result_offset))

                // let result = unsafe {
                //     core::slice::from_raw_parts_mut(
                //         base.byte_add(result_offset),
                //         size
                //     )
                // };

                // NonNull::new(result)
            }
        }).unwrap_or(core::ptr::null_mut())
    }
}

unsafe impl<const N: usize> GlobalAlloc for BumpArena<N> {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let size = layout.size();
        let alignment = layout.align();

        self.alloc_mem(size, alignment)
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        // Do nothing. This is bump allocator
    }
}

#[global_allocator]
static BUMP_ARENA: BumpArena<32> = BumpArena::<32>::new();

#[derive(Clone, Copy)]
pub struct BumpStatic;

unsafe impl Allocator for BumpStatic {
    fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        Ok(unsafe {
            NonNull::slice_from_raw_parts(
                NonNull::new(BUMP_ARENA.alloc(layout)).ok_or(AllocError)?,
                layout.size()
            )
        })
    }

    unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: Layout) {
        BUMP_ARENA.dealloc(ptr.as_ptr(), layout);
    }
}