use std::io::{self, Write};
use std::fs::{OpenOptions, read};

fn main() -> io::Result<()> {
    const DISK_FILE: &str = "disk.img";
    const DISK_SIZE: u64 = 200 * 1024 * 1024; // 200MB
    const DISK_LABEL: [u8; 11] = *b"MIKAN OS   ";

    const SOURCE_FILENAME: &str = "BOOTX64.efi";
    const TARGET_FILENAME: &str = "BOOTX64.efi";
    
    /***** provide a img partition. *****/

    // first create a virtual disk img file and zero it.
    let disk = OpenOptions::new()
                .read(true).write(true).create(true).truncate(true)
                .open(DISK_FILE)?;
    disk.set_len(DISK_SIZE)?;

    // FAT32-format the disk
    fatfs::format_volume(
        disk,
        fatfs::FormatVolumeOptions::new()
            .sectors_per_track(2)            // -s 2
            .fats(2)                         // -f 2
            // .total_sectors(32)            // -R 32
            .fat_type(fatfs::FatType::Fat32) // -F 32
            .volume_label(DISK_LABEL)
    )?;

    println!("disk format completed");

    /***** add a bootable efi file. *****/

    // reopen the disk
    let disk = OpenOptions::new()
                .read(true).write(true)
                .open(DISK_FILE)?;
    let fs = fatfs::FileSystem::new(
        disk,
        fatfs::FsOptions::new()
    )?;
    println!("fs successfully created");

    // create a efi file
    let mut efi_file = fs.root_dir()
                .create_dir("efi")?
                .create_dir("boot")?
                .create_file(TARGET_FILENAME)?;
    
    // copy file contents into the partition
    let src: Vec<u8> = read(SOURCE_FILENAME)?;
    efi_file.write_all(&src)?;

    Ok(())
}