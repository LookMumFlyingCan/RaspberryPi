#!/usr/bin/env bash

echo This program requires a flash.img in the same folder contatining a dietpi 64-bit image

#sudo usbboot/rpiboot

echo The rpi has been mounted, what is the device path?

read devpat

sudo umount ${devpat}1
sudo umount ${devpat}2

echo Flashing the emmc memory now, DO NOT DISCONNECT THE DEVICE
#sudo dd if=flash.img | pv -s 1200M | sudo dd of=$devpat bs=4096

echo Flashing done

echo Attaining lock on devices

sudo mount ${devpat}1 mnt/boot
sudo mount ${devpat}2 mnt/rootfs

echo dtoverlay=dwc2 | sudo tee -a mnt/boot/config.txt
echo dr_mode=host | sudo tee -a mnt/boot/config.txt
sudo rm mnt/boot/dietpi.txt
sudo cp dietpi mnt/boot/dietpi.txt

sudo mkdir mnt/boot/initial
sudo cp rootprompt mnt/boot/initial/rootprompt
sudo cp fishconfig mnt/boot/initial/fishconfig
sudo cp userprompt mnt/boot/initial/userprompt
sudo cp bootscript.sh mnt/boot/initial/bootscript.sh
sudo cp bootscript1.sh mnt/boot/initial/bootscript1.sh

sudo umount ${devpat}1
sudo umount ${devpat}2
