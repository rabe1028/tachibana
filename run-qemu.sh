#!/bin/sh

OVMF_CODE=resource/OVMF_CODE-pure-efi.fd
OVMF_VARS=resource/OVMF_VARS-pure-efi.fd
BUILD_DIR=mnt
BOOT_DIR=$BUILD_DIR/EFI/BOOT

BOOT_LOADER=$1
KERNEL=$2

echo $OVMF_CODE
echo $OVMF_VARS
echo $BOOT_LOADER
echo $KERNEL

mkdir -p mnt/EFI/BOOT
cp $BOOT_LOADER $BOOT_DIR/BootX64.efi
cp $KERNEL $BUILD_DIR/kernel.elf

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

# qemu-system-x86_64 \
#     -nodefaults \
#     -machine q35,accel=kvm:tcg \
#     -vga std \
#     -m 128M \
#     -drive if=pflash,format=raw,file=$OVMF_CODE,readonly=on \
#     -drive if=pflash,format=raw,file=$OVMF_VARS \
#     -drive format=raw,file=fat:rw:$BUILD_DIR \
#     -monitor stdio