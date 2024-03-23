use crate::blockmap::BlockMap;
use crate::consts::{BlockPointer, SUPERBLOCK_SIZE};
use crate::emu::HardDrive;
use crate::inode_table::InodeTable;
use crate::raw::{raw_read_block, raw_write_block};
use crate::superblock::SuperBlock;

pub(crate) struct FSIO {
    pub(crate) drive: HardDrive,
    pub block_size: usize,
    pub block_count: u64,
}

impl FSIO {
    pub fn new(drive: HardDrive, block_size: usize) -> FSIO {
        let block_count = drive.bytes / block_size as u64;
        FSIO { drive, block_size, block_count }
    }

    pub(crate) fn write_block(&self, index: BlockPointer, block: &Vec<u8>) {
        if block.len() != self.block_size {
            panic!("Block size mismatch");
        }

        if index >= self.block_count {
            panic!("Block index out of range");
        }

        raw_write_block(&self.drive, self.block_size, block, index);
    }

    pub(crate) fn read_block(&self, index: BlockPointer) -> Vec<u8> {
        if index >= self.block_count {
            panic!("Block index out of range");
        }

        raw_read_block(&self.drive, self.block_size, index)
    }
}

pub struct FS {
    fsio: FSIO,
    pub(crate) superblock: SuperBlock,
    blockmap: BlockMap,
    inode_table: InodeTable,
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

    pub(crate) fn write_block(&self, index: BlockPointer, block: &Vec<u8>) {
        self.fsio.write_block(index, block)
    }

    pub(crate) fn read_block(&self, index: BlockPointer) -> Vec<u8> {
        self.fsio.read_block(index)
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

    #[test]
    fn read_write_large_block() {
        {
            let drive = HardDrive::new("fs_read_write_block.img", 1024 * 512, 512);
            let fs = super::FS::new(drive, 1024);

            let block1 = vec![0x42; 1024];
            fs.write_block(3, &block1);
            assert_eq!(fs.read_block(3), block1);

            let block2 = vec![0x1; 1024];
            fs.write_block(4, &block2);
            assert_eq!(fs.read_block(4), block2);

            let block3 = vec![0x8; 1024];
            fs.write_block(3, &block3);
            assert_eq!(fs.read_block(3), block3);
        }
        fs::remove_file("fs_read_write_block.img").unwrap();
    }
}
