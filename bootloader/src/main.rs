#![no_std]
#![no_main]
// #![feature(abi_efiapi)]

extern crate uefi;
extern crate uefi_services;

extern crate shared;

use uefi::data_types::{Char16, CStr16};
use uefi::prelude::*;
use uefi::table::boot;
use uefi::proto::{
    loaded_image::LoadedImage,
    device_path::DevicePath,
    media::fs::SimpleFileSystem,
    media::file::*,
    console::gop::GraphicsOutput,
    // console::gop::PixelFormat,
};
use uefi_macros::cstr16;

use core::slice::from_raw_parts_mut;
use core::mem::size_of;
use core::fmt::Write;
// use core::arch::asm;

use bootloader::ArrayWriter;
use shared::{
    FrameBufferInfo,
    KernelArgs
};

#[inline]
fn uefi_boot(image_handle: uefi::Handle, system_table: &mut SystemTable<Boot>)
-> uefi::Result<(extern "C" fn(KernelArgs), KernelArgs)>
{
    uefi_services::init(system_table)?;

    // print in stdout
    system_table.stdout().write_str("Hello, Rust!\n")
        .unwrap();
    // writeln!(system_table.stdout(), "Hello, rust!\n");

    // get FAT32 file system for UEFI loader
    //
    // Normally, the below code do the job
    // ```
    // let mut root_fs = system_table.boot_services().get_image_file_system(image_handle)?;
    // ```
    // but since we need more than high-level encapsulation `uefi::fs::FileSystem`, we stripped off its method body.
    let mut root_dir = {
        let loaded_image = system_table.boot_services().open_protocol_exclusive::<LoadedImage>(image_handle)?;
        let device_path = system_table.boot_services().open_protocol_exclusive::<DevicePath>(loaded_image.device())?;
        let device_handle = system_table.boot_services().locate_device_path::<SimpleFileSystem>(&mut &*device_path)?;

        system_table.boot_services().open_protocol_exclusive::<SimpleFileSystem>(device_handle)?
            .open_volume()?
    };

    // acquire memory map for later use
    let mut mmap_buffer = [0u8; 0x4000]; // 16KiB
    let mmap = system_table.boot_services().memory_map(&mut mmap_buffer)?;

    // write memory map info into `/mmap.csv`.
    // relavent `uefi::fs::FileSystem` method: `root_fs.write(...)`
    let mut mmap_file = root_dir
        .open(cstr16!("mmap.csv"), FileMode::CreateReadWrite, FileAttribute::empty())?
        .into_regular_file().unwrap();
    for (i, desc) in mmap.entries().enumerate() {
        let mut content_buffer = ArrayWriter::<0x100>::new();
        // let mut content_buffer = [0u8; 0x100];
        writeln!(content_buffer,
            "{},{:X},{:?},{:08X},{:X},{:X}",
            i, desc.ty.0, desc.ty, desc.phys_start, desc.page_count, desc.att.bits()
        ).unwrap();
        mmap_file.write(content_buffer.as_slice()) // .write(&content_buffer)
            .map_err(|err|err.to_err_without_payload())?;
    };
    mmap_file.flush()?;

    // writeln!(system_table.stdout(), "Memory map file write success");

    // read kernel file
    // relavent `uefi::fs::FileSystem` method: `root_fs.metadata(...)` and `root_fs.read(...)` which returns a vector result.
    const KERNEL_FILE_NAME: &CStr16 = cstr16!("kernel.elf");
    let mut kernel_file = root_dir
        .open(KERNEL_FILE_NAME, FileMode::Read, FileAttribute::empty())?
        .into_regular_file().unwrap();

    // writeln!(system_table.stdout(), "Kernel file read success");

    // retrieve kernel info and allocate page
    const KERNEL_FILE_NAME_LEN: usize = KERNEL_FILE_NAME.to_u16_slice_with_nul().len();
    const KERNEL_FILE_INFO_BUF_SIZE: usize = 3 * size_of::<u64>() + 3 * size_of::<uefi::table::runtime::Time>() + size_of::<FileAttribute>() + (KERNEL_FILE_NAME_LEN + 5) * size_of::<Char16>(); //5 is for extra padding
    // attempted `const BUF_SIZE = size_of::<FileInfo>() + KERNEL_FILE_NAME_LEN * size_of::<Char16>();` but unfortunately `FileInfo` is not sized..
    let mut kernel_file_info_buffer = [0u8; KERNEL_FILE_INFO_BUF_SIZE];
    let kernel_file_info = kernel_file.get_info::<FileInfo>(&mut kernel_file_info_buffer)
        .map_err(|err| err.to_err_without_payload() )?;
    let kernel_file_size = kernel_file_info.file_size() as usize;
    let kernel_base_addr: uefi::data_types::PhysicalAddress = 0x100000; // should be synced with target json configuration
    // @TODO kernel should be a position-independent executable (PIE) if possible.
    system_table.boot_services().allocate_pages(
        boot::AllocateType::Address(kernel_base_addr),
        boot::MemoryType::LOADER_DATA,
        (kernel_file_size + boot::PAGE_SIZE - 1) / boot::PAGE_SIZE
    )?;

    // load the kernel
    let kernel_slice = unsafe{ from_raw_parts_mut(kernel_base_addr as *mut u8, kernel_file_size) };
    kernel_file.read(kernel_slice)?;

    // determine the entry point
    let kernel_entry_ptr = unsafe {
        // @TODO
        // I don't know why do we have a page-sized displacement(0x1000). Somebody help!
        core::ptr::read((kernel_base_addr + 24) as *const u64) - 0x1000
    } as *const ();

    system_table.boot_services().stall(1_000_000); // stall for 1 second
    writeln!(system_table.stdout(), "Executing kernel (Entry {:p})", kernel_entry_ptr).unwrap();

    // get graphics output protocol info, into file
    // guess that if we open GOP protocol then stdout becomes no longer valid.
    // so we keep this process as late as possible.
    let frame_buffer_info = {
        let gop_handle = system_table.boot_services().get_handle_for_protocol::<GraphicsOutput>()?;
        let mut gop = system_table.boot_services().open_protocol_exclusive::<GraphicsOutput>(gop_handle)?;

        // const GOP_MODES_FILE_NAME: &CStr16 = cstr16!("gop_modes.csv");
        // let mut gop_modes_file = root_dir
        //     .open(GOP_MODES_FILE_NAME, FileMode::CreateReadWrite, FileAttribute::empty())?
        //     .into_regular_file().unwrap();
        // for (i, mode) in gop.modes().enumerate() {
        //     let mut content_buffer = ArrayWriter::<0x100>::new();
        //     // let mut content_buffer = [0u8; 0x100];
        //     writeln!(content_buffer,
        //         "{},{:?},{:?},{:?},{}",
        //         i, mode.info().resolution(), mode.info().pixel_format(), mode.info().pixel_bitmask(), mode.info().stride()
        //     ).unwrap();
        //     gop_modes_file.write(content_buffer.as_slice()) // .write(&content_buffer)
        //         .map_err(|err|err.to_err_without_payload())?;
        // };
        // gop_modes_file.flush()?;
        
        FrameBufferInfo::new( gop.frame_buffer().as_mut_ptr(), gop.current_mode_info() )
    };

    system_table.boot_services().stall(1_000_000);

    // // kernel executing closure with parameters.
    // let kernel_main = unsafe {
    //     let kernel_entry: extern "C" fn(
    //         FrameBufferInfo
    //     ) = core::mem::transmute(kernel_entry_ptr);
    //     // move || kernel_entry(
    //     //     frame_buffer_info
    //     // )
    //     move || {
    //         kernel_entry(frame_buffer_info);
    //     }
    // };

    Ok((
        unsafe { core::mem::transmute(kernel_entry_ptr) },
        KernelArgs {
            frame_buffer_info
        }
    ))
}

#[entry]
fn uefi_start(image_handle: uefi::Handle, mut system_table: SystemTable<Boot>) -> Status {
    match uefi_boot(image_handle, &mut system_table){
        Ok((kernel_entry, kernel_args)) => {
            // exit the booting process.
            let _ = system_table.exit_boot_services();

            // let's roll!
            kernel_entry(kernel_args);

            Status::SUCCESS
        },
        Err(err) => {
            writeln!(system_table.stderr(), "{:?}", err).unwrap();
            err.status()
        },
    }
}
