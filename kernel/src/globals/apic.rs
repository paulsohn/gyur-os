// https://wiki.osdev.org/APIC

use core::ptr::NonNull;
use core::ops::{Deref, DerefMut};
use spin::lazy::Lazy as LazyLock;

use x86_64::registers::model_specific::Msr;
use apic::ApicBase;

/// APIC Memory-mapped Access.
pub static APIC: LazyLock<Apic> = LazyLock::new(|| unsafe {
    let base_addr = Msr::new(0x1B).read() & 0xfffff000;
    Apic::new(base_addr as usize)
});

// pub static mut APIC: Apic = unsafe { Apic::new(0xfee00000) };
// #[inline]
// pub fn init(){
//     unsafe {
//         let base_addr = Msr::new(0x1B).read() & 0xfffff000;
//         APIC = Apic::new(base_addr as usize);
//     }
// }

/// An `ApicBase` wrapper just here to mark `ApicBase` sync.
pub struct Apic(ApicBase);
impl Apic {
    pub const unsafe fn new(base_addr: usize) -> Self {
        let addr = NonNull::new(base_addr as *mut _).unwrap();
        Self(ApicBase::new(addr))
    }
}
impl Deref for Apic {
    type Target = ApicBase;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl DerefMut for Apic {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
unsafe impl Send for Apic {}
unsafe impl Sync for Apic {} // quite unsafe here.