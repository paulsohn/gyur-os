//! A multi-buffer implementations of command ring, transfer ring, and event ring.

extern crate alloc;

use core::mem::ManuallyDrop;

// the allocator should not only allocate the memory,
// but should also map it into the virtual address and return it.
use alloc::alloc::{Allocator, Layout};
use alloc::boxed::Box;
use alloc::vec::Vec;

/// The ring buffer base alignment.
pub const RING_BUF_BASE_ALIGN: usize = 64;

fn alloc_buf<T, A>(size: usize, allocator: A, value: T) -> Box<[T], A>
where
    T: Copy,
    A: Allocator,
{
    unsafe {
        let ptr = allocator.allocate(
            Layout::from_size_align(size * core::mem::size_of::<T>(),
            RING_BUF_BASE_ALIGN).unwrap()
        )
        .ok()
        .map(|ptr| ptr.as_ptr() /* .as_mut_ptr() */ as *mut u8 as *mut T)
        .unwrap_or(core::ptr::null_mut());

        let buf_ptr = core::ptr::slice_from_raw_parts_mut(ptr, size);

        (&mut *buf_ptr).fill(value);

        Box::from_raw_in(
            buf_ptr,
            allocator
        )
    }
}

unsafe fn boxed_slice_from_slice_in<T, A>(slice: &mut [T], allocator: A) -> Box<[T], A>
where
    A: Allocator,
{
    let len = slice.len();
    Vec::<T, A>::from_raw_parts_in(slice.as_mut_ptr(), len, len, allocator).into_boxed_slice()
}

use xhci::ring::trb::{transfer, event, command};

#[allow(type_alias_bounds)]
type Segment<T, A: Allocator> = Box<[T], A>;

macro_rules! set_chain_bit {
    (transfer, $block:ident, $link:ident) => {
        if $block.chain_bit() {
            $link.set_chain_bit();
        }
    };
    (command, $block:ident, $link:ident) => {};
}

macro_rules! add_pushable_ring {
    ($ring_name:ident, $ring_ty:ident) => {
        paste::paste! {
            #[doc = "A " $ring_ty " ring."]
            #[derive(Debug)]
            pub struct $ring_name<A>
            where
                A: Allocator + Clone + 'static,
            {
                /// Ring segments.
                segs: Vec<Segment<$ring_ty::TRB, A>, A>,
                /// Current segment index for push.
                seg_cur: usize,
                /// Current block index for push.
                block_cur: usize,
                /// Current cycle bit.
                cycle_bit: bool,
                /// The default allocator for this ring.
                allocator: A,
            }
        }

        impl<A> $ring_name<A>
        where
            A: Allocator + Clone + 'static,
        {
            /// Create an uninitialized ring with no segments allocated.
            pub fn new_uninit(allocator: A) -> Self {
                Self {
                    segs: Vec::new_in(allocator.clone()),
                    seg_cur: 0,
                    block_cur: 0,
                    cycle_bit: true,
                    allocator,
                }
            }

            /// Returns if the ring has any buffer to use.
            pub fn is_init(&self) -> bool {
                self.segs.len() > 0
            }

            /// Create a new ring initialized with single segment of size `size`.
            pub fn new(size: usize, allocator: A) -> Self {
                let mut r = Self::new_uninit(allocator);

                // Add the first segment.
                // `seg_cur`, `block_cur` and `cycle_bit` is already ready-to-go.
                r.add_segment(size);
                r
            }

            /// Get the current pointer.
            unsafe fn get_ptr(&self) -> *mut $ring_ty::TRB {
                self.segs[self.seg_cur].as_ptr()
                    .add(self.block_cur) as _ // transmute to mut ptr
            }

            /// Write a block into current enqueue pointer, with respect to current cycle bit.
            /// Returns the pointer on which the block was written.
            fn write_block_with_cycle_bit(&mut self, mut block: $ring_ty::TRB) {
                if self.cycle_bit {
                    block.set_cycle_bit();
                } else {
                    block.clear_cycle_bit();
                }

                unsafe {
                    self.get_ptr().write_volatile(block);
                }
            }

            /// Push a block into the ring.
            /// Returns the previous enqueue pointer, as the pair of the segment and block indices the block was put.
            pub fn push(&mut self, block: $ring_ty::TRB) -> *const $ring_ty::TRB {
                assert!(self.is_init());

                let last_ptr = unsafe { self.get_ptr() };

                // push the desired block.
                self.write_block_with_cycle_bit(block);
                self.block_cur += 1;

                // if next block is the last block of the segment,
                // push a link TRB.
                if self.block_cur == self.segs[self.seg_cur].len() - 1 {
                    let seg_next = if self.seg_cur == self.segs.len() - 1 {
                        0
                    } else {
                        self.seg_cur + 1
                    };
                    let toggle_cond = (seg_next == 0);

                    let seg_next_base = self.segs[seg_next].as_mut_ptr() as usize as u64;

                    let mut link = *$ring_ty::Link::new()
                        .set_ring_segment_pointer(seg_next_base); // next segment base

                    if toggle_cond { // toggle cond
                        link.set_toggle_cycle();
                    }

                    set_chain_bit!($ring_ty, block, link);

                    self.write_block_with_cycle_bit(link.into());
                    self.seg_cur = seg_next;
                    self.block_cur = 0;
                    if toggle_cond {
                        self.cycle_bit ^= true;
                    }
                }

                last_ptr
            }

            /// Add a new segment with size `size` into the ring.
            /// Never call this on initialized ring if you want it single-segmented.
            pub fn add_segment(&mut self, size: usize) {
                self.segs.push(alloc_buf(
                    size,
                    self.allocator.clone(),
                    // Block::zero_with_cycle_bit(!self.cycle_bit),
                    // $ring_ty::TRB::new(),
                    Default::default(),
                ));
            }

            /// Get the buffer base pointer of `i`th segment.
            pub unsafe fn get_buf_ptr(&self, i: usize) -> *const $ring_ty::TRB {
                assert!(i < self.segs.len());

                self.segs[i].as_ptr()
            }
        }
    };
}

add_pushable_ring!(CommandRing, command);
add_pushable_ring!(TransferRing, transfer);

use xhci::ring::erst::{
    EventRingSegmentTableEntry,
    EventRingSegmentTableEntryBlock
};
use xhci::registers::runtime::InterrupterRegisterSet;

use core::mem::MaybeUninit;
use core::ops::{Index, IndexMut};

use volatile::VolatilePtr;
// use volatile::map_field;
use volatile_field::Structural;

/// Event Ring Segment Table which inner buffer is guaranteed to be 64-byte aligned.
struct EventRingSegmentTable<A>
where
    A: Allocator + Clone + 'static,
{
    /// The ring segments, in the form of `EventRingSegmentTableEntry` rather than slices.
    table: Vec<EventRingSegmentTableEntryBlock, A>, // the length is ERSTSZ / 4
    /// The vector of allocators. Required to manually drop ring segments.
    allocators: Vec<ManuallyDrop<A>, A>, // the length is actual ERSTSZ, max 255
}
impl<A> EventRingSegmentTable<A>
where
    A: Allocator + Clone + 'static,
{
    /// Create an empty ERST.
    pub(crate) fn new_uninit(allocator: A) -> Self {
        Self {
            table: Vec::new_in(allocator.clone()),
            allocators: Vec::new_in(allocator),
        }
    }

    /// ERST Size (entry count).
    pub(crate) fn len(&self) -> usize {
        self.allocators.len()
    }

    /// ERST Base Address.
    pub(crate) fn base(&self) -> u64 {
        self.table.as_ptr() as usize as u64
    }

    pub(crate) fn push(&mut self, seg: Segment<event::TRB, A>) {
        assert!(self.len() <= u8::MAX as usize);

        let (ptr, al) = Box::into_raw_with_allocator(seg);
        let entry = unsafe {
            let buf = &*ptr;
            EventRingSegmentTableEntry::from_buf(buf)
        };

        let rem = self.len() % 4;
        if rem == 0 {
            self.table.push(
                EventRingSegmentTableEntryBlock( MaybeUninit::uninit_array())
            )
        }
        self.table.last_mut().unwrap().0[rem].write(entry);
        self.allocators.push(ManuallyDrop::new(al));
    }
}
impl<A> Index<usize> for EventRingSegmentTable<A>
where
    A: Allocator + Clone + 'static,
{
    type Output = EventRingSegmentTableEntry;

    fn index(&self, index: usize) -> &Self::Output {
        assert!(index < self.len());
        unsafe {
            self.table[index / 4].0[index % 4].assume_init_ref()
        }
    }
}
impl<A> IndexMut<usize> for EventRingSegmentTable<A>
where
    A: Allocator + Clone + 'static,
{
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        assert!(index < self.len());
        unsafe {
            self.table[index / 4].0[index % 4].assume_init_mut()
        }
    }
}
impl<A> Drop for EventRingSegmentTable<A>
where
    A: Allocator + Clone + 'static,
{
    fn drop(&mut self) {
        for i in 0..self.allocators.len() {
            unsafe {
                let mut entry = self.table[i / 4].0[i % 4].assume_init();

                let a = ManuallyDrop::take(self.allocators.get_mut(i).unwrap());

                let _b = boxed_slice_from_slice_in(entry.as_mut_slice(), a);

                // `_b` is dropped, so that the allocated slice from the erst entry is freed.
            }
        }
        // `self.table` and `self.allocators` are also dropped.
    }
}

/// A single-buffer implementation of event ring.
pub struct EventRing<'r, A>
where
    A: Allocator + Clone + 'static,
{
    /// The Interrupter associated to the event ring.
    interrupter: VolatilePtr<'r, InterrupterRegisterSet>,
    /// The Event Ring Segment Table
    erst: EventRingSegmentTable<A>,
    /// Current cycle bit.
    cycle_bit: bool,
    /// The default allocator for this ring.
    allocator: A,
}

impl<'r, A> EventRing<'r, A>
where
    A: Allocator + Clone + 'static,
{
    /// Create an uninitialized Event Ring from interrupter and the allocator.
    /// No buffers are allocated. This method exists only for completeness.
    #[deprecated]
    pub fn new_uninit(
        interrupter: VolatilePtr<'r, InterrupterRegisterSet>,
        allocator: A,
    ) -> Self {
        Self {
            interrupter,
            erst: EventRingSegmentTable::new_uninit(allocator.clone()),
            cycle_bit: true,
            allocator,
        }
    }

    /// Create a new Event Ring from interrupter, first buffer size, and the allocator.
    pub fn new(
        interrupter: VolatilePtr<'r, InterrupterRegisterSet>,
        size: usize,
        allocator: A,
    ) -> Self {
        #[allow(deprecated)]
        let mut er = Self::new_uninit(interrupter, allocator);

        // Add the first segment and initialize dequeue pointer.
        er.add_segment(size);

        er
    }
}

impl<A> EventRing<'_, A>
where
    A: Allocator + Clone + 'static,
{
    /// Returns if the ring has any buffer to use.
    #[deprecated]
    pub fn is_init(&self) -> bool {
        self.erst.len() > 0
    }

    /// Enable interrupt from the interrupter.
    pub fn enable_interrupt(&mut self) {
        self.interrupter.fields().iman().update(|mut iman| {
            *iman.clear_interrupt_pending() // RW1C, this writes 1 to clear
                .set_interrupt_enable()
        });
    }

    /// Set event ring segment index and dequeue pointer.
    fn set_erdp(&mut self, seg_idx: u8, dequeue_pointer: u64) {
        self.interrupter.fields().erdp().update(|mut erdp| {
            *erdp
                // .clear_event_handler_busy()
                .set_dequeue_erst_segment_index(seg_idx)
                .set_event_ring_dequeue_pointer(dequeue_pointer)
        });
    }

    /// Add a new segment with size `size` into the ring.
    /// Never call this if you want a single-segmented ring.
    /// 
    /// If this is the first segment, initialize the dequeue pointer also.
    ///
    /// # Panics
    ///
    /// This method panics if the ring already has 255 segments.
    pub fn add_segment(&mut self, size: usize) {
        // assert!(self.erst.len() <= u8::MAX as usize);

        // allocate a new segment.
        let seg: Segment<event::TRB, A> = alloc_buf(
            size,
            self.allocator.clone(),
            // Block::zero_with_cycle_bit(!self.cycle_bit),
            Default::default()
        );

        self.erst.push(seg);

        // update interrupter registers.
        let base = self.erst.base();
        let len = self.erst.len() as u16;

        if len == 1 { // init. set ERDP here.
            self.set_erdp(0, self.erst[0].ring_segment_base_address());
        }

        // event ring is enabled by erstba write. erstba should be updated last.
        self.interrupter.fields().erstsz().update(|mut erstsz| *erstsz.set(len));
        self.interrupter.fields().erstba().update(|mut erstba| *erstba.set(base));
    }

    /// Performs dequeue operation, and returns the block if not empty.
    /// 
    /// # Panics
    /// 
    /// This method panics if there are no buffers available.
    pub fn pop(&mut self) -> Option<event::TRB> {
        assert!({
            #[allow(deprecated)]
            self.is_init()
        });

        // Get the dequeue pointer.
        let (seg_cur, dq_ptr) = {
            let erdp = self.interrupter.fields().erdp().read();
            (
                erdp.dequeue_erst_segment_index() as usize,
                erdp.event_ring_dequeue_pointer() as usize as *const event::TRB,
            )
        };

        // Get the front block.
        let front = unsafe { dq_ptr.read_volatile() };

        // Check whether the block should be consumed.
        if front.cycle_bit() == self.cycle_bit {
            // Increment the current dequeue pointer.
            let incremented = unsafe { dq_ptr.add(1) } as usize as u64;
            let bound = self.erst[seg_cur].ring_segment_bound_address();

            // Determine the new segment index and dequeue pointer.
            let (seg_next, new_dq_pos) = if incremented == bound {
                // Incremented ptr has reached the bound.
                // Flip the cycle bit and move to the next (or front) segment.
                let seg_next = if seg_cur == self.erst.len() - 1 {
                    self.cycle_bit ^= true;
                    0
                } else {
                    seg_cur + 1
                };
                let seg_next_base = self.erst[seg_next].ring_segment_base_address();

                (seg_next, seg_next_base)
            } else {
                (seg_cur, incremented)
            };

            // Update dequeue pointer register.
            self.set_erdp(seg_next as u8, new_dq_pos);

            Some(front)
        } else {
            None
        }
    }
}
