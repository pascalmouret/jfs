extern crate core;

mod fs;
mod consts;
mod inode_table;
mod inode;
mod directory;
mod driver;
mod io;
mod structure;

const DRIVE_SIZE: u64 = 10 * 1024 * 1024;

fn main() {}
