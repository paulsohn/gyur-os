// https://wiki.osdev.org/APIC

use core::ptr::NonNull;
use core::ops::{Deref, DerefMut};
use spin::lazy::Lazy as LazyLock;

use x86_64::registers::model_specific::Msr;
use apic::ApicBase;

/// APIC Memory-mapped Base Address.
pub static APIC_BASE: LazyLock<Apic> = LazyLock::new(|| unsafe {
    Apic::new(Msr::new(0x1B).read() as usize)
});
// /// The BootStrap Processor Local APIC ID
// pub static BSP_LAPIC_ID: LazyLock<u8> = LazyLock::new(|| {
//     APIC_BASE.id().read().id()
// });

/// An `ApicBase` wrapper just here to mark `ApicBase` sync.
pub struct Apic(ApicBase);
impl Apic {
    pub unsafe fn new(base_addr: usize) -> Self {
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