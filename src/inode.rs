use std::mem::size_of;
use crate::consts::{BlockPointer, DirectPointers, InodePointer};
use crate::driver::DeviceDriver;
use crate::fs::FS;
use crate::io::IO;

#[repr(u8)]
#[derive(PartialEq, Debug, Copy, Clone)]
pub enum InodeKind {
    Directory,
    File,
}

#[derive(PartialEq, Debug, Copy, Clone)]
#[repr(C, packed)]
pub struct Inode {
    pub kind: InodeKind,
    pub id: InodePointer,
    pub size: u64,
    pointers: DirectPointers,
}

impl Inode {
    pub fn new(tpe: InodeKind, id: InodePointer, pointers: [BlockPointer; 12]) -> Inode {
        Inode { kind: tpe, pointers, id, size: 0 }
    }

    pub fn to_bytes(&self) -> &[u8; size_of::<Inode>()] {
        unsafe { &*(self as *const Inode as *const [u8; size_of::<Inode>()]) }
    }

    pub fn from_bytes(bytes: [u8; size_of::<Inode>()]) -> Inode {
        unsafe { *(bytes.as_ptr() as *const Inode) }
    }

    pub fn read_all<A: DeviceDriver>(&self, io: &IO<A>) -> Vec<u8> {
        let mut data = Vec::new();
        let last_block = self.size as usize / io.block_size;
        for i in 0..last_block {
            let pointer = self.pointers[i];
            let block = io.read_block(pointer);
            data.extend_from_slice(&block);
        }
        data.truncate(self.size as usize);
        data
    }

    pub fn write_all<A: DeviceDriver>(&mut self, fs: &mut FS<A>, data: &[u8]) {
        let mut data = data.to_vec();

        self.set_size(fs, data.len() as u64);

        let last_block = data.len() / fs.superblock.block_size;
        for i in 0..last_block {
            let offset = i * fs.superblock.block_size;
            let limit = (i + 1) * fs.superblock.block_size;

            if limit > data.len() {
                data.extend_from_slice(&vec![0; limit - data.len()]);
            }

            let block = &data[offset..limit];
            let pointer = self.pointers[i as usize];
            fs.io.write_block(pointer, &block.to_vec());
        }
    }

    fn set_size<A: DeviceDriver>(&mut self, fs: &mut FS<A>, size: u64) {
        let current_blocks = (self.size / fs.superblock.block_size as u64) as usize;
        let new_blocks = (size / fs.superblock.block_size as u64) as usize;;

        if current_blocks > new_blocks {
            for i in new_blocks..current_blocks {
                fs.blockmap.mark_free(&mut fs.io, self.pointers[i]);
                self.pointers[i] = 0;
            }
        } else if current_blocks < new_blocks {
            for i in current_blocks..new_blocks {
                self.pointers[i] = fs.blockmap.allocate(&mut fs.io).unwrap();
            }
        }

        self.size = size;
        fs.inode_table.write_inode(&mut fs.io, *self);
    }

    pub fn set_pointers(&mut self, pointers: DirectPointers) {
        self.pointers = pointers;
    }

    pub fn set_pointer(&mut self, index: usize, pointer: BlockPointer) {
        self.pointers[index] = pointer;
    }
}

#[cfg(test)]
mod tests {
    use crate::driver::file_drive::FileDrive;
    use crate::fs::FS;
    use crate::inode::InodeKind;

    #[test]
    fn read_write_data() {
        let drive = FileDrive::new("./test-images/inode_read_write_data.img", 4096 * 512, 512);
        let mut fs = FS::new(drive, 512);
        let mut inode = fs.inode_table.create_inode(&mut fs.io, InodeKind::File);
        let data = vec![42; 1024];
        inode.write_all(&mut fs, &data);
        let read = inode.read_all(&fs.io);
        assert_eq!(data, read);
    }
}
