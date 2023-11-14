cd ./bootloader
cargo check
cd ../

rustup component add rust-src --toolchain nightly-x86_64-unknown-linux-gnu

cd ./kernel
cargo check
cd ../