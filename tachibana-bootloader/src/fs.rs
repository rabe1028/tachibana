use alloc::boxed::Box;
use alloc::vec::Vec;
use uefi::prelude::*;
use uefi::proto::media::file::{
    Directory, File, FileAttribute, FileInfo, FileMode, FileType, RegularFile,
};
pub struct Root {
    root: Directory
}

impl Root {
    pub fn open(image: Handle, bs: &BootServices) -> Self {
        let sfs = bs
        .get_image_file_system(image)
        .expect_success("Failed to get file system");
        let root = unsafe { &mut *sfs.get() }
            .open_volume()
            .expect_success("Failed to get volume");

        Root {
            root: root
        }
    }

    pub fn create_file(&mut self, filename: &str) -> RegularFile {
        match create(&mut (self.root), filename, false) {
            FileType::Regular(file) => file,
            FileType::Dir(_) => panic!("Not a regular file: {}", filename),
        }
    }

    pub fn create_dir(&mut self, dirname: &str) -> Directory {
        match create(&mut (self.root), dirname, true) {
            FileType::Regular(_) => panic!("Not a directory: {}", dirname),
            FileType::Dir(directory) => directory
        }
    }
}



pub fn open_root_dir(image: Handle, bs: &BootServices) -> Root {
    let sfs = bs
        .get_image_file_system(image)
        .expect_success("Failed to get file system");
    let root = unsafe { &mut *sfs.get() }
        .open_volume()
        .expect_success("Failed to get volume");

    Root {
        root: root
    }
}

pub fn create(dir: &mut Directory, filename: &str, create_dir: bool) -> FileType {
    let attr = if create_dir {
        FileAttribute::DIRECTORY
    } else {
        FileAttribute::empty()
    };
    dir.open(filename, FileMode::CreateReadWrite, attr)
        .expect_success("Failed to create file")
        .into_type()
        .unwrap_success()
}

pub fn open(dir: &mut Directory, filename: &str) -> FileType {
    dir.open(filename, FileMode::Read, FileAttribute::empty())
        .expect_success("Failed to open file")
        .into_type()
        .unwrap_success()
}

pub fn create_file(dir: &mut Directory, filename: &str) -> RegularFile {
    match create(dir, filename, false) {
        FileType::Regular(file) => file,
        FileType::Dir(_) => panic!("Not a regular file: {}", filename),
    }
}

pub fn open_file(dir: &mut Directory, filename: &str) -> RegularFile {
    match open(dir, filename) {
        FileType::Regular(file) => file,
        FileType::Dir(_) => panic!("Not a regular file: {}", filename),
    }
}

pub fn read_file_to_vec(file: &mut RegularFile) -> Vec<u8> {
    let size = get_file_info(file).file_size() as usize;
    let mut buf = vec![0; size];
    file.read(&mut buf).unwrap_success();
    buf
}

pub fn get_file_info(file: &mut impl File) -> Box<FileInfo> {
    file.get_boxed_info::<FileInfo>().unwrap_success()
}

macro_rules! fwrite {
    ($file:expr, $format:tt $( $rest:tt )*) => {
        $file.write(format!($format $( $rest )*).as_bytes()).unwrap_success()
    };
}

macro_rules! fwriteln {
    ($file:expr, $format:tt $( $rest:tt )*) => {
        fwrite!($file, concat!($format, "\n") $( $rest )*)
    };
}
