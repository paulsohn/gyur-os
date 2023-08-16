mkdir -p ./mnt
sudo mount -o loop ./disk.img ./mnt
mkdir -p ./mnt/efi/boot
cargo build --package bootloader --target x86_64-unknown-uefi
sudo cp ./target/x86_64-unknown-uefi/debug/bootloader.efi ./mnt/efi/boot/BOOTX64.EFI
cargo +nightly build --package kernel --target ./kernel.json
sudo cp ./target/kernel/debug/kernel ./mnt/kernel.elf
sudo umount ./mnt