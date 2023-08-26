mkdir -p ./mnt
sudo mount -o loop ./disk.img ./mnt
mkdir -p ./mnt/efi/boot

cd ./bootloader
cargo build
cd ../
sudo cp ./target/x86_64-unknown-uefi/debug/bootloader.efi ./mnt/efi/boot/BOOTX64.EFI

cd ./kernel
cargo build
cd ../
sudo cp ./target/x86_64-gyur/debug/kernel ./mnt/kernel.elf

sudo umount ./mnt