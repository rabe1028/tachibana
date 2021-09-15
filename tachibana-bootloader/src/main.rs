#![no_std]
#![no_main]
#![feature(abi_efiapi)]

// uefi-servicesを明示的にリンク対象に含める
extern crate uefi_services;

#[macro_use]
extern crate alloc;

mod fs;

use crate::alloc::vec::*;
use crate::fs::*;
use core::fmt::Write;
use core::panic::PanicInfo;
use log::*;
use uefi::prelude::*;
use uefi::proto::media::file::{
    Directory, File, FileAttribute, FileHandle, FileInfo, FileMode, FileType,
};
use uefi::table::boot::AllocateType;
use uefi::table::boot::{MemoryDescriptor, MemoryType};

#[entry]
fn efi_main(image: Handle, mut system_table: SystemTable<Boot>) -> Status {
    // st.stdout().reset(false).unwrap_success();
    // writeln!(st.stdout(), "Hello, World!").unwrap();

    uefi_services::init(&system_table).expect_success("Failed to initialize utils");

    // reset console before doing anything else
    system_table
        .stdout()
        .reset(false)
        .expect_success("Failed to reset output buffer");

    info!("Hello, Mikan World!");

    // Print out UEFI revision number
    {
        let rev = system_table.uefi_revision();
        let (major, minor) = (rev.major(), rev.minor());

        info!("UEFI {}.{}", major, minor);
    }

    memory_map("memmap", image, system_table.boot_services());

    loop {}
}

fn memory_map(filepath: &str, image: uefi::Handle, bt: &BootServices) {
    // Get the estimated map size
    let map_size = bt.memory_map_size() + 8 * core::mem::size_of::<MemoryDescriptor>();
    // let map_size = 4096 * 4;
    info!("map_size : {}", map_size);

    // Build a buffer bigger enough to handle the memory map
    let mut buffer = vec![0; map_size];

    let (_k, desc_iter) = bt
        .memory_map(&mut buffer)
        .expect_success("Failed to retrieve UEFI memory map");

    // write memory map
    let mut root_dir = Root::open(image, bt);

    let mut file = root_dir.create_file(filepath);

    // Print out a list of all the usable memory we see in the memory map.
    // Don't print out everything, the memory map is probably pretty big
    // (e.g. OVMF under QEMU returns a map with nearly 50 entries here).

    info!("efi: usable memory ranges ({} total)", desc_iter.len());

    file.write("Index, Type, Type(name), PhysicalStart, NumberOfPages, Attribute\n".as_bytes())
        .unwrap_success();
    // 本書のCコードでやってるプリミティブなイテレート等はuefi-rsでは不要だった
    for (i, d) in desc_iter.enumerate() {
        let line = format!(
            "{}, {:x}, {:?}, {:08x}, {:x}, {:x}\n",
            i,
            d.ty.0,
            d.ty,
            d.phys_start,
            d.page_count,
            d.att.bits() & 0xfffff
        );
        file.write(line.as_bytes()).unwrap_success();
        // info!("{}",line);
    }
    // drop(file);

    info!("done.");
}

