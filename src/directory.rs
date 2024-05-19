use crate::consts::{FileName, InodePointer};
use crate::fs::FS;
use crate::inode::{Inode, InodeKind};

#[repr(C, packed)]
struct DirectoryEntry {
    name: FileName,
    pointer: InodePointer,
}

pub struct Directory {
    fs: &'static mut FS,
    inode: Inode,
    entries: Vec<DirectoryEntry>
}

impl Directory {
    pub fn create(fs: &'static mut FS) -> Directory {
        let inode = fs.inode_table.create_inode(&fs.fsio, InodeKind::Directory);
        Directory { fs, inode, entries: Vec::new() }
    }

    pub fn read(fs: &'static mut FS, pointer: InodePointer) -> Directory {
        let inode = fs.inode_table.read_inode(&fs.fsio, pointer);

        if inode.kind != InodeKind::Directory {
            panic!("Inode is not a directory");
        }



        Directory { fs, inode, entries: Vec::new() }
    }

    pub fn add_entry(&mut self, name: &str, pointer: InodePointer) {
    }

    pub fn remove_entry(&mut self, name: &str) {
    }
}
