use crate::pgmgr::{
    FrameID,
    PageManager,
    KERNEL_PAGE_SIZE,
};

use spin::mutex::Mutex;

use shared::uefi_memory::{
    MemoryMap,
    MemoryType,
    PAGE_SIZE as UEFI_PAGE_SIZE,
};

pub static PAGE_MANAGER: Mutex<PageManager> = Mutex::new(PageManager::new());

fn is_available(ty: MemoryType) -> bool {
    [
        MemoryType::CONVENTIONAL,
        MemoryType::BOOT_SERVICES_CODE,
        MemoryType::BOOT_SERVICES_DATA,
    ].contains(&ty)
}

/// Initializes memory manager.
pub fn init(mmap: &MemoryMap<'static>) {
    let mut mgr = PAGE_MANAGER.lock();

    // for (i, desc) in mmap.entries().enumerate() {
    //     if i <= 30 && is_available(desc.ty) {
    //         log::info!(
    //             "{},{:?},{:08X},{:X},{:X}",
    //             i, desc.ty, desc.phys_start, desc.page_count, desc.att.bits()
    //         );
    //     }
    // }

    let mut avail_end = 0usize;
    for desc in mmap.entries() {
        // this assumes that mmap entries are properly sorted.
        // todo: this algorithm will not work properly if actual `KERNEL_PAGE_SIZE` and `UEFI_PAGE_SIZE` were different. improve this.

        let phys_start = desc.phys_start as usize;
        let phys_end = desc.phys_start as usize + UEFI_PAGE_SIZE * desc.page_count as usize;

        // mark reserved from previous end and current start.
        mgr.mark_reserved(
            FrameID(avail_end / KERNEL_PAGE_SIZE),
            (phys_start - avail_end) / KERNEL_PAGE_SIZE
        );

        // mark current segment reserved, or extend a range.
        if is_available(desc.ty) {
            avail_end = phys_end;
        } else {
            mgr.mark_reserved(
                FrameID(phys_start / KERNEL_PAGE_SIZE),
                (phys_end - phys_start) / KERNEL_PAGE_SIZE
            );
        }
    }

    mgr.set_range(FrameID(1), FrameID(avail_end / KERNEL_PAGE_SIZE));
}