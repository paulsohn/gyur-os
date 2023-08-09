#![no_std]
#![no_main]
// #![feature(asm)]
// #![feature(abi_efiapi)]

extern crate uefi;
extern crate uefi_services;

use uefi::prelude::*;
// use uefi::CString16;
use uefi_macros::cstr16;

use core::fmt::Write;
use mikanos_rust::BufferWriter;

#[inline]
fn uefi_boot(image_handle: uefi::Handle, mut system_table: SystemTable<Boot>) -> uefi::Result {
    uefi_services::init(&mut system_table)?;

    let stdout = system_table.stdout();
    stdout.output_string(cstr16!("Hello, rust!\n"))?;

    let boot_services = system_table.boot_services();
    let mut mmap_buffer = [0u8; 0x4000]; // 16KiB
    let mmap = boot_services.memory_map(&mut mmap_buffer)?; // now can iterate mmap.entries()

    let mut root_dir = boot_services.get_image_file_system(image_handle)?;

    // we want to open a file, grab the handle and append its contents as many time as we want
    // unfortunately, current `uefi` crate encapsulate all process and only allow us a one-time cascading write.

    let mut content_buffer = BufferWriter::<0x1000>::new();

    for (i, desc) in mmap.entries().enumerate() {
        writeln!(content_buffer,
            "{},{:X},{:?},{:08X},{:X},{:X}",
            i, desc.ty.0, desc.ty, desc.phys_start, desc.page_count, desc.att.bits()
        );
    }
    root_dir.write(cstr16!("memmap.csv"), content_buffer.as_slice())
        .map_err(|err| {
            if let uefi::fs::Error::Io(err) = err {
                err.uefi_error
            } else {
                unreachable!("The error should be IO error.")
            }
        });

    boot_services.stall(10_000_000);
    // loop {};

    Ok(())
}

#[entry]
fn uefi_start(image_handle: uefi::Handle, mut system_table: SystemTable<Boot>) -> Status {
    match uefi_boot(image_handle, system_table){
        Ok(()) => Status::SUCCESS,
        Err(err) => err.status(),
    }
}
