mkdir -p ./mnt
sudo mount -o loop ./disk.img ./mnt
mkdir -p ./mnt/efi/boot
cargo build --target x86_64-unknown-uefi
sudo cp ./target/x86_64-unknown-uefi/debug/mikanos_rust.efi ./mnt/efi/boot/BOOTX64.EFI
sudo umount ./mnt