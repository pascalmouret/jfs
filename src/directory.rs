use crate::consts::{FileName, InodePointer};
use crate::driver::DeviceDriver;
use crate::fs::FS;
use crate::inode::{Inode, InodeKind};
use crate::io::IO;

#[repr(C, packed)]
struct DirectoryEntry {
    name: FileName,
    pointer: InodePointer,
}

pub struct Directory {
    inode: Inode,
    entries: Vec<DirectoryEntry>
}

impl Directory {
    pub fn create<A: DeviceDriver>(fs: &mut FS<A>) -> Directory {
        let inode = fs.inode_table.create_inode(&mut fs.io, InodeKind::Directory);
        Directory { inode, entries: Vec::new() }
    }

    pub fn read<A: DeviceDriver>(fs: &mut FS<A>, pointer: InodePointer) -> Directory {
        let inode = fs.inode_table.read_inode(&fs.io, pointer);

        if inode.kind != InodeKind::Directory {
            panic!("Inode is not a directory");
        }

        Directory { inode, entries: Vec::new() }
    }
}
