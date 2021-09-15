#!/bin/sh

OVMF_CODE=resource/OVMF_CODE-pure-efi.fd
OVMF_VARS=resource/OVMF_VARS-pure-efi.fd
BUILD_DIR=mnt
BOOT_DIR=$BUILD_DIR/EFI/BOOT

echo $OVMF_CODE
echo $OVMF_VARS
echo $1

mkdir -p mnt/EFI/BOOT
cp $1 $BOOT_DIR/BootX64.efi

echo "\EFI\BOOT\BOOTX64.EFI" > mnt/startup.nsh

qemu-system-x86_64 \
    -nodefaults \
    -machine q35,accel=kvm:tcg \
    -vga std \
    -m 128M \
    -drive if=pflash,format=raw,file=$OVMF_CODE,readonly=on \
    -drive if=pflash,format=raw,file=$OVMF_VARS \
    -drive format=raw,file=fat:rw:$BUILD_DIR \
    -serial stdio \
    -monitor vc:1024x768 \