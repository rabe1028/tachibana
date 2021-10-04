all: tachibana-bootloader.efi tachibana-kernel.elf

tachibana-bootloader.efi:
	cd tachibana-bootloader && cargo build

tachibana-kernel.elf:
	cd tachibana-kernel && cargo build

qemu: tachibana-bootloader.efi tachibana-kernel.elf
	./run-qemu.sh target/x86_64-unknown-uefi/debug/tachibana-bootloader.efi target/x86_64-unknown-tachibana-elf/debug/tachibana-kernel.elf
