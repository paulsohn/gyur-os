qemu-system-x86_64 \
    -drive if=pflash,format=raw,readonly,file=./OVMF_CODE.fd \
    -drive if=pflash,format=raw,file=./OVMF_VARS.fd \
    -device nec-usb-xhci,id=xhci \
    -device usb-mouse,bus=xhci.0 \
    -device usb-kbd,bus=xhci.0 \
    --trace "usb_xhci_queue_event" \
    --trace "usb_xhci_fetch_trb" \
    -monitor stdio \
    -hda disk.img