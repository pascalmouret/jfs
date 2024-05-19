use crate::blockmap::BlockMap;
use crate::consts::{SUPERBLOCK_SIZE};
use crate::directory::Directory;
use crate::emu::HardDrive;
use crate::fsio::FSIO;
use crate::inode_table::InodeTable;
use crate::superblock::SuperBlock;

pub struct FS {
    pub(crate) fsio: FSIO,
    pub(crate) superblock: SuperBlock,
    pub(crate) blockmap: BlockMap,
    pub(crate) inode_table: InodeTable,
}

impl FS {
    pub fn new(drive: HardDrive, block_size: usize) -> FS {
        if block_size < drive.sector_size {
            panic!("Block size must be greater than or equal to sector size");
        }

        if block_size % drive.sector_size != 0 {
            panic!("Block size must be a multiple of sector size");
        }

        let fsio = FSIO::new(drive, block_size);
        let mut superblock = SuperBlock::new(block_size, fsio.drive.bytes / block_size as u64);
        superblock.write(&fsio);

        let mut blockmap = BlockMap::new((SUPERBLOCK_SIZE / fsio.block_size) as u64, superblock.block_count, block_size);
        blockmap.write_full(&fsio);

        let inode_index = blockmap.last_block + 1;
        let inode_table = InodeTable::create(inode_index, &fsio);
        for i in 0..inode_table.block_count {
            blockmap.mark_used(&fsio, inode_index + i as u64);
        }
        superblock.set_inode_count(&fsio, inode_table.inode_count);

        FS::mount(fsio.drive)
    }

    pub fn mount(drive: HardDrive) -> FS {
        match SuperBlock::read(&drive) {
            Some(superblock) => {
                let fsio = FSIO::new(drive, superblock.block_size);
                let blockmap = BlockMap::read(&fsio, (SUPERBLOCK_SIZE / fsio.block_size) as u64);
                let inode_table = InodeTable::read(&fsio, blockmap.last_block + 1, superblock.inode_count);
                FS { fsio, superblock, blockmap, inode_table }
            }
            None => panic!("No superblock found"),
        }
    }

    pub fn create_dir(&'static mut self) -> Directory {
        Directory::create(self)
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use crate::emu::HardDrive;
    use crate::superblock::SuperBlock;

    #[test]
    fn init() {
        {
            let drive = HardDrive::new("fs_init.img", 1024 * 512, 512);
            let fs = super::FS::new(drive, 512);
            assert_eq!(fs.superblock, SuperBlock::new(512, 1024));
        }
        fs::remove_file("fs_init.img").unwrap();
    }
}
