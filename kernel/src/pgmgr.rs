use bit_field::BitField;
// use shared::uefi_memory::PAGE_SIZE as UEFI_PAGE_SIZE;

pub const KB: usize = 0x400;
pub const MB: usize = KB * KB;
pub const GB: usize = MB * KB;

pub const KERNEL_PAGE_SIZE: usize = 4 * KB; // 0x1000

#[derive(Clone, Copy)]
#[repr(C, align(0x1000))]
pub struct Page([u8; KERNEL_PAGE_SIZE]);
impl Page {
    pub const fn new() -> Self {
        Self([0; KERNEL_PAGE_SIZE])
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct FrameID(pub usize);
impl FrameID {
    /// Translate the frame ID into the physical address.
    pub fn addr(&self) -> usize {
        self.0 * KERNEL_PAGE_SIZE
    }
}

/// Page Status of either vacant or using.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PageStat {
    Vacant = 0,
    Using, // allocated by this page manager
    Reserved, // using and not allocated by this page manager. currently unused
}

/// The maximum memory address (exclusive)
pub const MAX_MEMORY: usize = 128 * GB;
/// The maximum page frame number (exclusive)
pub const MAX_PAGES: usize = MAX_MEMORY / KERNEL_PAGE_SIZE;

type PgMgrLine = u32;
const PGMGR_LINE_BITS: usize = 32;

pub struct PageManager {
    bitmap: [PgMgrLine; MAX_PAGES / PGMGR_LINE_BITS],
    begin: FrameID,
    end: FrameID, // represents [begin, end) range
}

impl PageManager {
    /// Create a new page manager with default frame id range.
    /// Initially all pages are marked as vacant.
    pub const fn new() -> Self {
        Self {
            bitmap: [0; _],
            begin: FrameID(0),
            end: FrameID(MAX_PAGES),
        }
    }

    /// Set the frame id range of which this manager is responsible.
    pub fn set_range(&mut self, begin: FrameID, end: FrameID) {
        self.begin = begin;
        self.end = end;
    }

    /// Get the status of the page of the given frame id.
    /// This only guarantees that free memory gives `PageStat::Vacant`.
    pub fn get_stat(&self, id: FrameID) -> PageStat {
        assert!(self.begin <= id, "FrameID range bound error");
        assert!(id < self.end, "FrameID range bound error");

        let line_no = id.0 / PGMGR_LINE_BITS;
        let bit_no = id.0 % PGMGR_LINE_BITS;

        if self.bitmap[line_no].get_bit(bit_no) {
            PageStat::Using
        } else {
            PageStat::Vacant
        }
    }

    /// Set the status of the page of the given frame id.
    pub fn set_stat(&mut self, id: FrameID, stat: PageStat) {
        assert!(self.begin <= id, "FrameID range bound error");
        assert!(id < self.end, "FrameID range bound error");

        let line_no = id.0 / PGMGR_LINE_BITS;
        let bit_no = id.0 % PGMGR_LINE_BITS;

        self.bitmap[line_no].set_bit(bit_no, stat != PageStat::Vacant);
    }

    fn set_range_stat(&mut self, begin: FrameID, page_cnt: usize, stat: PageStat) {
        // let value = stat == PageStat::Using;
        for id in (begin.0)..(begin.0 + page_cnt) {
            self.set_stat(FrameID(id), stat);
        }
    }
}

impl PageManager { // allocation and freeing
    /// Mark specified frame range as reserved.
    /// Mainly used in initialization.
    pub fn mark_reserved(&mut self, begin: FrameID, page_cnt: usize) {
        self.set_range_stat(begin, page_cnt, PageStat::Reserved);
    }

    /// Allocate pages of the given page count.
    /// This includes zeroing the allocated region.
    pub fn allocate(&mut self, page_cnt: usize) -> Result<FrameID> {
        // implement first-fit.
        let mut acc = 0usize;
        for i in (self.begin.0)..(self.end.0) {
            match self.get_stat(FrameID(i)) {
                PageStat::Vacant => { acc += 1; },
                _ => { acc = 0; },
            };
            if acc == page_cnt {
                let frame_id = FrameID(i - page_cnt + 1); // start frame id.

                self.set_range_stat(frame_id, page_cnt, PageStat::Using);

                // // zeroing the page
                // unsafe {
                //     core::slice::from_raw_parts_mut(
                //         (frame_id.0 * KERNEL_PAGE_SIZE) as *mut u8,
                //         page_cnt * KERNEL_PAGE_SIZE
                //     ).fill(0);
                // }

                return Ok(frame_id);
            }
        }
        Err(PageAllocationError::NotEnoughMemory)
    }

    /// Free pages of the given range.
    /// In this simple page manager, we need both start frame id and page count.
    /// Memoizing the list might be the future improvement.
    pub fn free(&mut self, begin: FrameID, page_cnt: usize) -> Result<()> {
        self.set_range_stat(begin, page_cnt, PageStat::Vacant);
        // todo: prevent attempting to free unusing pages?

        Ok(())
    }
}

impl PageManager { // statistics - part
    /// Returns total number of frames of which this manager is responsible.
    /// 
    /// This is a function for memory-managing statistics.
    pub const fn total_frame_count(&self) -> usize {
        self.end.0 - self.begin.0
    }

    /// Returns available number of frames of which this manager is responsible.
    /// 
    /// This is a function for memory-managing statistics.
    pub fn available_frame_count(&self) -> usize {
        let mut cnt = 0;
        for i in (self.begin.0)..(self.end.0) {
            if self.get_stat(FrameID(i)) == PageStat::Vacant {
                cnt += 1;
            }
        }
        cnt
    }
}

pub type Result<T> = core::result::Result<T, PageAllocationError>;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PageAllocationError {
    NotEnoughMemory
}