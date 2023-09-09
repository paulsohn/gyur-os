qemu-system-x86_64 \
    -drive if=pflash,format=raw,readonly,file=./OVMF_CODE.fd \
    -drive if=pflash,format=raw,file=./OVMF_VARS.fd \
    -device qemu-xhci,id=xhci \
    -device usb-mouse,bus=xhci.0 \
    -monitor stdio \
    -hda disk.img