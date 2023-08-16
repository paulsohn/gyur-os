cargo check --package bootloader --target x86_64-unknown-uefi
cargo +nightly check --package kernel --target ./kernel.json