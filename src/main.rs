extern crate core;

mod emu;
mod fs;
mod superblock;
mod consts;
mod raw;
mod blockmap;
mod inode_table;
mod inode;
mod fsio;

const DRIVE_SIZE: u64 = 10 * 1024 * 1024;

fn main() {
    let fs = fs::FS::new(emu::HardDrive::new("test.img", DRIVE_SIZE, 512), 512);
    println!("Superblock: {:?}", fs.superblock);
}
