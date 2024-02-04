use x86_64::structures::gdt::{
    GlobalDescriptorTable,
    Descriptor,
};
use x86_64::registers::segmentation::{
    Segment, SegmentSelector,
    CS, DS, ES, FS, GS, SS
};
use x86_64::PrivilegeLevel;

static mut GDT: GlobalDescriptorTable = GlobalDescriptorTable::new();

#[inline]
pub fn init(){
    // load GDT.
    unsafe {
        GDT.add_entry(Descriptor::kernel_code_segment()); // index 1
        GDT.add_entry(Descriptor::kernel_data_segment()); // index 2, 64-bit data segment
        GDT.load();
    }

    // set segment registers.
    unsafe {
        DS::set_reg(SegmentSelector::NULL); // unused
        ES::set_reg(SegmentSelector::NULL); // unused
        FS::set_reg(SegmentSelector::NULL);
        GS::set_reg(SegmentSelector::NULL);
        
        SS::set_reg(SegmentSelector::new(2, PrivilegeLevel::Ring0));
        CS::set_reg(SegmentSelector::new(1, PrivilegeLevel::Ring0));
    }
}