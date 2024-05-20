extern crate core;

mod fs;
mod superblock;
mod consts;
mod blockmap;
mod inode_table;
mod inode;
mod directory;
mod driver;
mod io;

const DRIVE_SIZE: u64 = 10 * 1024 * 1024;

fn main() {
    let fs = fs::FS::new(driver::file_drive::FileDrive::new("test.img", DRIVE_SIZE, 512), 512);
    println!("Superblock: {:?}", fs.superblock);
}
