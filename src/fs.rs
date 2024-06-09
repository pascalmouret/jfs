use crate::driver::DeviceDriver;
use crate::io::IO;
use crate::structure::Structure;
use crate::structure::inode::Inode;

pub struct FS<INODE: Inode> {
    pub(crate) io: IO,
    pub(crate) structure: Structure<INODE>,
}

impl <INODE: Inode>FS<INODE> {
    pub fn new<D: DeviceDriver + 'static>(drive: D, block_size: usize) -> FS<INODE> {
        let mut io = IO::new(drive, block_size);
        let structure = Structure::<INODE>::new(&mut io, block_size);
        FS { io, structure }
    }

    pub fn mount<D: DeviceDriver + 'static>(drive: D) -> FS<INODE> {
        let default_block_size = drive.get_sector_size();
        let mut io = IO::new(drive, default_block_size);
        let structure = Structure::<INODE>::mount(&mut io);
        FS { io, structure }
    }
}

#[cfg(test)]
mod tests {
    use std::mem::size_of;
    use crate::driver::file_drive::FileDrive;
    use crate::structure::inode::{Inode, INODE_ID};
    use crate::structure::superblock::SuperBlock;

    struct DummyInode {
        id: Option<INODE_ID>,
        data: u64,
    }

    impl DummyInode {
        fn new(data: u64) -> DummyInode {
            DummyInode { id: None, data }
        }
    }

    impl Inode for DummyInode {
        fn id(&self) -> Option<INODE_ID> {
            self.id
        }

        fn set_id(&mut self, id: INODE_ID) {
            self.id = Some(id);
        }

        fn to_bytes(&self) -> Vec<u8> {
            self.data.to_le_bytes().to_vec()
        }

        fn from_bytes(bytes: &Vec<u8>) -> Self {
            DummyInode { id: None, data: u64::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7]]) }
        }

        fn disk_size() -> usize {
            size_of::<u64>()
        }
    }

    #[test]
    fn init() {
        let drive = FileDrive::new("./test-images/fs_init.img", 1024 * 512, 512);
        let fs = super::FS::<DummyInode>::new(drive, 512);
        assert_eq!(fs.structure.super_block, SuperBlock::new(512, 1024));
    }
}
