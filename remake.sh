sudo umount mnt
cp back.img ext2.img
sudo mount -o sync ext2.img mnt
sudo chown mrfan:users mnt

