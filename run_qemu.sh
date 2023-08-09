qemu-system-x86_64 \
    -drive if=pflash,file=./OVMF_CODE.fd \
    -drive if=pflash,file=./OVMF_VARS.fd \
    -hda disk.img