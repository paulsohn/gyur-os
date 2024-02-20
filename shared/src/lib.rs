#![no_std]

// TODO: specify FFI-safe layout for Memory Map.

/// subset of re-exports of `uefi`
pub mod uefi_memory {
    pub use uefi::table::boot::{
        MemoryMap,
        MemoryMapKey,
        MemoryDescriptor,
        MemoryType,
        MemoryAttribute,
        AllocateType,
        PAGE_SIZE,
    };
}
pub mod uefi_gop {
    pub use uefi::proto::console::gop::{
        FrameBuffer,
        ModeInfo,
        PixelFormat,
    };
}

/// The kernel argument type, which can be provided from bootloading process.
/// This doesn't include memory map.
#[derive(Debug, /* Copy, Clone, PartialEq, Eq */)]
#[repr(C)]
pub struct KernelArgs {
    pub gop_frame_buffer: uefi_gop::FrameBuffer<'static>,
    pub gop_mode_info: uefi_gop::ModeInfo,
}