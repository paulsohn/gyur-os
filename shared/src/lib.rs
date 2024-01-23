#![no_std]

pub mod frame_buffer;
pub mod memory_map; // some re-exports of uefi

// /// kernel argument type
// #[derive(Copy, Clone, Debug, PartialEq, Eq)]
// #[repr(C)]
// pub struct KernelArgs {
//     pub frame_buffer: FrameBuffer
// }
