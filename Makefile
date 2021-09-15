all: tachibana-bootloader.efi tachibana-kernel.elf

tachibana-bootloader.efi:
	cd tachibana-bootloader && cargo build

qemu: tachibana-bootloader.efi
	cd tachibana-kernel && cargo run