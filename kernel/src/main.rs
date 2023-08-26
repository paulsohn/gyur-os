#![allow(dead_code)]
#![no_std]
#![no_main]

extern crate shared;

use core::arch::asm;
use core::panic::PanicInfo;

use shared::{
    KernelArgs,
    // FrameBufferInfo
};
use kernel::{ ColorCode, Screen };

#[no_mangle]
pub extern "C" fn _start (
    kernel_args: KernelArgs
) -> ! {
    // a disassembly tells us that in assembly level
    // `kernel_args` is passed as a pointer in `rcx` register,
    // referencing certain position on the system stack.
    // However when we compile this kernel,
    // the kernel binary is ignorant to `rcx`
    // and attempts to find `kernel_args` from the top of the stack.
    // I couldn't unify this FFI mismatch, so here arguments are retrieved from the register manually.
    let kernel_args = unsafe {
        let mut kernel_args_ptr: *const KernelArgs;
        asm!(
            "mov {0}, rcx",
            out(reg) kernel_args_ptr
        );
        core::ptr::read(kernel_args_ptr)
    };

    // unsafe {
    //     asm!(
    //         "movq xmm0, {0}",
    //         "movq xmm2, {1}",
    //         "movq xmm4, {2}",
    //         "movq xmm6, {3}",
    //         in(reg) kernel_args.frame_buffer_info.base,
    //         in(reg) kernel_args.frame_buffer_info.stride,
    //         in(reg) kernel_args.frame_buffer_info.hor_res,
    //         in(reg) kernel_args.frame_buffer_info.ver_res,
    //     )
    // }

    let mut screen = Screen::from(kernel_args.frame_buffer_info);
    for x in 0..screen.hor_res {
        for y in 0..screen.ver_res {
            // screen.write_pixel( (x,y), ColorCode::YELLOW );
            screen.write_pixel(
                (x,y),
                ColorCode::rgb(
                    u8::try_from(x % 256).unwrap(),
                    u8::try_from(x % 256).unwrap(),
                    u8::try_from(y % 256).unwrap()
                )
            );
        }
    }

    halt();
}

#[panic_handler]
fn panic_handler(_info: &PanicInfo) -> ! {
    unsafe{
        // handy way to indicate panic, if QEMU debugger is enabled
        asm!("mov r11, 0xDEAD");
    } 
    halt()
}

fn halt() -> ! {
    loop{
        unsafe{ asm!("hlt"); }
    }
}