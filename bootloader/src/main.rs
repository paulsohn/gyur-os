#![no_std]
#![no_main]
// #![feature(never_type)]
// #![feature(abi_efiapi)]

use shared::KernelArgs;
use shared::uefi_memory::{
    MemoryMap,
    MemoryType,
    AllocateType,
    PAGE_SIZE
}; // this is shortcut of `uefi::table::boot::*`
use uefi::{
    self,
    prelude::*,
    data_types::{Char16, CStr16},
    proto::{
        loaded_image::LoadedImage,
        device_path::DevicePath,
        media::fs::SimpleFileSystem,
        media::file::*,
        console::gop::GraphicsOutput,
        // console::gop::PixelFormat,
    },
    cstr16,
};

use elf::ElfBytes;
use elf::endian::AnyEndian;

use core::slice::from_raw_parts_mut;
use core::mem::size_of;
use core::fmt::Write;
// use core::arch::asm;

// use bootloader::ArrayWriter;

// static mut MMAP_BUFFER: [u8; 0x4000] = [0u8; 0x4000]; // 16KiB

#[inline]
fn uefi_boot(image_handle: Handle, system_table: &mut SystemTable<Boot>)
-> uefi::Result<(extern "sysv64" fn(MemoryMap<'static>, KernelArgs), KernelArgs)>
{
    uefi_services::init(system_table)?;

    // // print in stdout
    // system_table.stdout().write_str("Hello, Rust!\n")
    //     .unwrap();
    // writeln!(system_table.stdout(), "Hello, rust!\n").unwrap();

    // get FAT32 file system for UEFI loader
    //
    // Normally, the below code do the job
    // ```
    // let mut root_fs = system_table.boot_services().get_image_file_system(image_handle)?;
    // ```
    // but since we need more than high-level encapsulation `uefi::fs::FileSystem`, we stripped off its method body.
    let mut root_dir = {
        let loaded_image = system_table.boot_services().open_protocol_exclusive::<LoadedImage>(image_handle)?;
        let device_path = system_table.boot_services().open_protocol_exclusive::<DevicePath>(loaded_image.device().unwrap())?;
        let device_handle = system_table.boot_services().locate_device_path::<SimpleFileSystem>(&mut &*device_path)?;

        system_table.boot_services().open_protocol_exclusive::<SimpleFileSystem>(device_handle)?
            .open_volume()?
    };

    // read kernel file
    // relavent `uefi::fs::FileSystem` method: `root_fs.metadata(...)` and `root_fs.read(...)` which returns a vector result.
    const KERNEL_FILE_NAME: &CStr16 = cstr16!("kernel.elf");

    let mut kernel_file = root_dir
        .open(KERNEL_FILE_NAME, FileMode::Read, FileAttribute::empty())?
        .into_regular_file().unwrap();

    // writeln!(system_table.stdout(), "Kernel file read success");

    // retrieve kernel info and allocate page
    
    let kernel_file_size = {
        const KERNEL_FILE_NAME_LEN: usize = KERNEL_FILE_NAME.to_u16_slice_with_nul().len();
        const KERNEL_FILE_INFO_BUF_SIZE: usize = 3 * size_of::<u64>() + 3 * size_of::<uefi::table::runtime::Time>() + size_of::<FileAttribute>() + (KERNEL_FILE_NAME_LEN + 5) * size_of::<Char16>(); //5 is for extra padding
        // attempted `const BUF_SIZE = size_of::<FileInfo>() + KERNEL_FILE_NAME_LEN * size_of::<Char16>();` but unfortunately `FileInfo` is not sized..

        let mut kernel_file_info_buffer = [0u8; KERNEL_FILE_INFO_BUF_SIZE];
        let kernel_file_info = kernel_file.get_info::<FileInfo>(&mut kernel_file_info_buffer)
            .map_err(|err| err.to_err_without_payload() )?;

        kernel_file_info.file_size() as usize
    };

    // refering elf program headers, determine kernel base and bound addresses.
    // load all segments and determine the kernel entry point address.
    let kernel_entry_addr = {
        // read the kernel and load into temporarily allocated area
        let kernel_buffer_ptr = system_table.boot_services().allocate_pool(
            MemoryType::LOADER_DATA,
            kernel_file_size
        )?;
        let kernel_buffer_slice = unsafe{ from_raw_parts_mut(kernel_buffer_ptr as *mut u8, kernel_file_size) };
        kernel_file.read(kernel_buffer_slice)?;

        // parse the kernel.
        let elf = ElfBytes::<AnyEndian>::minimal_parse(kernel_buffer_slice).unwrap();
        // we can easily get entry address by peeking file header.
        // but we have to load all segments (and free the pool) before exiting this block
        // by loading each segment manually, 0x1000 displacement bug is resolved.
        let kernel_entry_addr = elf.ehdr.e_entry;

        // determine base and bound addresses.
        let mut kernel_base_addr = u64::MAX; // after the iteration, this should be 0x200000 (or the address specified as `--image-base` linker option, in target json config)
        let mut kernel_bound_addr = u64::MIN; // naively this is kernel_base_addr + kernel_file_size, but we may have e.g. .bss section.
        for phdr in elf.segments().unwrap().iter() {
            if phdr.p_type != elf::abi::PT_LOAD { continue; }
            // honestly, we want methods for 'assign if greater' and 'assign if less'.
            kernel_base_addr = kernel_base_addr.min(phdr.p_vaddr);
            kernel_bound_addr = kernel_bound_addr.max(phdr.p_vaddr + phdr.p_memsz);
        }

        // allocate real pages
        let kernel_byte_count = (kernel_bound_addr - kernel_base_addr) as usize;
        system_table.boot_services().allocate_pages(
            AllocateType::Address(kernel_base_addr),
            MemoryType::LOADER_DATA,
            (kernel_byte_count + PAGE_SIZE - 1) / PAGE_SIZE
        )?;

        // copy segment data into real target, and fill zero if necessary
        for phdr in elf.segments().unwrap().iter() {
            if phdr.p_type != elf::abi::PT_LOAD { continue; }

            unsafe{
                core::ptr::copy(
                    kernel_buffer_ptr.add(phdr.p_offset as usize),
                    phdr.p_vaddr as *mut u8,
                    phdr.p_filesz as usize
                );
                core::ptr::write_bytes(
                    (phdr.p_vaddr as *mut u8).add(phdr.p_filesz as usize),
                    0,
                    phdr.p_memsz.saturating_sub(phdr.p_filesz) as usize
                );
            }
        }

        // abandon the temp buffer
        // can we make this auto-drop?
        unsafe {
            system_table.boot_services().free_pool(kernel_buffer_ptr)?;
        }

        kernel_entry_addr
    };

    writeln!(system_table.stdout(), "Executing kernel (Entry {:p})", kernel_entry_addr as *const ()).unwrap();

    // get graphics output protocol info.
    // guess that if we open GOP protocol then stdout becomes no longer valid.
    // so we keep this process as late as possible.
    let (gop_frame_buffer, gop_mode_info) = {
        let gop_handle = system_table.boot_services().get_handle_for_protocol::<GraphicsOutput>()?;
        let mut gop = system_table.boot_services().open_protocol_exclusive::<GraphicsOutput>(gop_handle)?;

        // can select other GOP modes (iterate by `gop.modes()`)
        // but we will choose the default selected mode.
        // @TODO: can we set mode at runtime(by kernel)?
        (
            // gop.frame_buffer(),
            unsafe { core::mem::transmute(
                gop.frame_buffer()
            ) }, // @TODO: A frame buffer should have no lifetime. Open an issue (on `uefi` crate) for this.
            gop.current_mode_info()
        )
    };

    // system_table.boot_services().stall(500_000);

    let kernel_entry: extern "sysv64" fn(
        MemoryMap<'static>,
        KernelArgs
    ) = unsafe {
        core::mem::transmute(kernel_entry_addr)
    };

    let args = KernelArgs {
        gop_frame_buffer,
        gop_mode_info,
    };

    Ok((kernel_entry, args))
}

#[entry]
fn uefi_start(image_handle: Handle, mut system_table: SystemTable<Boot>) -> Status {
    match uefi_boot(image_handle, &mut system_table){
        Ok((kernel_entry, args)) => {
            // exit the booting process.
            let (_system_table, mmap) = system_table.exit_boot_services(MemoryType::LOADER_DATA);

            // alright. let's roll!
            kernel_entry(
                mmap,
                args
            );

            Status::SUCCESS
        },
        Err(err) => {
            writeln!(system_table.stderr(), "{:?}", err).unwrap();
            err.status()
        },
    }
}
