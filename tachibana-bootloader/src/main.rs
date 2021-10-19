#![no_std]
#![no_main]
#![feature(abi_efiapi)]

// uefi-servicesを明示的にリンク対象に含める
extern crate uefi_services;

#[macro_use]
extern crate alloc;

mod fs;

use crate::fs::*;
use alloc::vec;

use log::*;
use tachibana_common::frame_buffer::FrameBufferPayload;
use uefi::prelude::*;
use uefi::proto::console::gop::{GraphicsOutput, PixelFormat};
use uefi::proto::media::file::File;
use uefi::table::boot::AllocateType;
use uefi::table::boot::{MemoryDescriptor, MemoryType};

use goblin::elf::{self, Elf};

use uefi::table::Runtime;

#[entry]
fn efi_main(image: Handle, system_table: SystemTable<Boot>) -> Status {
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

    let entry_point_addr = load_kernel(image, system_table.boot_services());

    info!("entry_point_addr = 0x{:x}", entry_point_addr);
    let entry_point: extern "sysv64" fn(&tachibana_common::frame_buffer::FrameBuffer) =
        unsafe { core::mem::transmute(entry_point_addr) };

    info!("get_frame_buffer_config");
    let frame_buffer = get_frame_buffer(system_table.boot_services());

    info!("exit_boot_services");
    let _st = exit_boot_services(image, system_table);

    entry_point(&frame_buffer);

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

fn load_kernel(image: Handle, bs: &BootServices) -> usize {
    const KERNEL_BASE_ADDR: usize = 0x100000;
    const EFI_PAGE_SIZE: usize = 0x1000;

    let mut root_dir = Root::open(image, bs);
    let mut file = root_dir.open_file("kernel.elf");

    // copy elf file to KERNEL_BASE_ADDR
    let size = get_file_info(&mut file).file_size() as usize;
    let mut buf = {
        let kernel_ptr = bs
            .allocate_pool(MemoryType::LOADER_DATA, size)
            .unwrap_success();
        info!(
            "buffer for elf : ptr = {}, size = {}",
            kernel_ptr as usize, size
        );
        unsafe { core::slice::from_raw_parts_mut(kernel_ptr, size) }
    };

    // file.read(buf).unwrap_success();

    // let mut buf = vec![0; size];
    file.read(&mut buf).unwrap_success();

    // loading elf
    let elf = elf::Elf::parse(buf).expect("Failed to parse ELF");

    let (kernel_first_addr, kernel_last_addr) = calc_load_address_range(&elf);

    info!("Kernel: {} - {}", kernel_first_addr, kernel_last_addr);

    let num_pages = (kernel_last_addr - kernel_first_addr + EFI_PAGE_SIZE - 1) / EFI_PAGE_SIZE;
    info!("num_pages: {}", num_pages);

    // kernel_base_addr -> kernel_first_addrにすると、エラーが発生
    bs.allocate_pages(
        AllocateType::Address(KERNEL_BASE_ADDR),
        MemoryType::LOADER_DATA,
        num_pages,
    )
    .expect_success("Failed to allocate pages for kernel");
    // NOTFOUNDは、メモリを確保できない時
    // ref : https://tnishinaga.hatenablog.com/entry/2015/10/13/033536

    copy_load_segments(&elf, buf);

    elf.entry as usize
}

fn calc_load_address_range(elf: &Elf) -> (usize, usize) {
    let mut first = usize::MAX;
    let mut last = 0;
    for ph in elf.program_headers.iter() {
        if ph.p_type != elf::program_header::PT_LOAD {
            continue;
        }

        info!("{:?}", ph);
        first = last.min(ph.p_vaddr as usize);
        last = last.max((ph.p_vaddr + ph.p_memsz) as usize);
    }

    (first, last)
}

fn copy_load_segments(elf: &Elf, src: &[u8]) {
    for ph in elf.program_headers.iter() {
        if ph.p_type != elf::program_header::PT_LOAD {
            continue;
        }
        let ofs = ph.p_offset as usize;
        let fsize = ph.p_filesz as usize;
        let msize = ph.p_memsz as usize;
        let dest = unsafe { core::slice::from_raw_parts_mut(ph.p_vaddr as *mut u8, msize) };
        dest[..fsize].copy_from_slice(&src[ofs..ofs + fsize]);
        dest[fsize..].fill(0);
    }
}

fn get_frame_buffer(bs: &BootServices) -> tachibana_common::frame_buffer::FrameBuffer {
    use tachibana_common::*;

    let gop = bs.locate_protocol::<GraphicsOutput>().unwrap_success();
    let gop = unsafe { &mut *gop.get() };

    match gop.current_mode_info().pixel_format() {
        PixelFormat::Rgb => frame_buffer::FrameBuffer::Rgb(FrameBufferPayload::new(
            gop.frame_buffer().as_mut_ptr(), gop.current_mode_info().stride() as u32, 
            (gop.current_mode_info().resolution().0 as u32,gop.current_mode_info().resolution().1 as u32)
        )),
        PixelFormat::Bgr => frame_buffer::FrameBuffer::Bgr(FrameBufferPayload::new(
            gop.frame_buffer().as_mut_ptr(), gop.current_mode_info().stride() as u32, 
            (gop.current_mode_info().resolution().0 as u32,
            gop.current_mode_info().resolution().1 as u32)
        )),
        f => panic!("Unsupported pixel format: {:?}", f),
    }

}

fn exit_boot_services(image: Handle, st: SystemTable<Boot>) -> SystemTable<Runtime> {
    let enough_mmap_size =
        st.boot_services().memory_map_size() + 8 * core::mem::size_of::<MemoryDescriptor>();
    let mut mmap_buf = vec![0; enough_mmap_size];
    let (st, _) = st
        .exit_boot_services(image, &mut mmap_buf[..])
        .expect_success("Failed to exit boot services");
    core::mem::forget(mmap_buf);
    st
}
