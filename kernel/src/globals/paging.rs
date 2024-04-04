use core::mem::MaybeUninit;
use core::ptr::addr_of;

use x86_64::structures::paging::{
    page_table::{
        PageTable,
        // PageTableEntry,
        PageTableFlags,
    },
    frame::PhysFrame,
};
use x86_64::PhysAddr;
use x86_64::registers::control::{
    Cr3, Cr3Flags
};

// an identity page mapping tables.
// we have one PML4 table, (using only index `0` -> PML3 table)
// one PML3 table, (using only index `0..ID_PD_CNT` -> each PML2 table)
// `ID_PD_CNT` times PML2 tables, each of which 512 entries represents a huge(2MB) page.
// hence, this identity mapping can serve up to `ID_PD_CNT` * 512 * 2MB memory.

const ID_PD_CNT: usize = 64;
static mut ID_PML4: PageTable = PageTable::new();
static mut ID_PML3: PageTable = PageTable::new();
static mut ID_PML2_ARR: MaybeUninit<[PageTable; ID_PD_CNT]> = MaybeUninit::zeroed();

// [PageTable::new(); ID_PD_CNT];

macro_rules! phys_addr {
    ($it:expr) => {
        PhysAddr::new(addr_of!($it) as usize as u64)
    }
}

#[inline]
pub fn init() {
    unsafe {
        // prepare the identity mapping
        // configure PML4
        ID_PML4[0].set_addr(
            phys_addr!(ID_PML3),
            PageTableFlags::PRESENT | PageTableFlags::WRITABLE
        );
        // configure PML3
        for i in 0..ID_PD_CNT {
            ID_PML3[i].set_addr(
                phys_addr!(ID_PML2_ARR.assume_init_ref()[i]),
                PageTableFlags::PRESENT | PageTableFlags::WRITABLE
            );
            // configure PML2
            for j in 0..512usize {
                ID_PML2_ARR.assume_init_mut()[i][j].set_addr(
                    PhysAddr::new(0x200000 * (512 * i + j) as u64),
                    PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::HUGE_PAGE
                );
            }
        }

        // set CR3 register.
        Cr3::write(
            PhysFrame::from_start_address(phys_addr!(ID_PML4)).unwrap(),
            Cr3Flags::empty()
        );
    }
}

