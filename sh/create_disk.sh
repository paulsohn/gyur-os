qemu-img create -f raw disk.img 200M
mkfs.fat -n 'GYUR OS' -s 2 -f 2 -R 32 -F 32 disk.img