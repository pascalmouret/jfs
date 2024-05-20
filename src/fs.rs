use crate::blockmap::BlockMap;
use crate::consts::{SUPERBLOCK_SIZE};
use crate::directory::Directory;
use crate::driver::DeviceDriver;
use crate::inode_table::InodeTable;
use crate::io::IO;
use crate::superblock::SuperBlock;

pub struct FS<A: DeviceDriver> {
    pub(crate) io: IO<A>,
    pub(crate) superblock: SuperBlock,
    pub(crate) blockmap: BlockMap,
    pub(crate) inode_table: InodeTable,
}

impl <A: DeviceDriver>FS<A> {
    pub fn new(drive: A, block_size: usize) -> FS<A> {
        if block_size < drive.get_sector_size() {
            panic!("Block size must be greater than or equal to sector size");
        }

        if block_size % drive.get_sector_size() != 0 {
            panic!("Block size must be a multiple of sector size");
        }

        let mut io = IO::new(drive, block_size);
        let mut superblock = SuperBlock::new(block_size, io.device.get_size() / block_size as u64);
        superblock.write(&mut io);

        let mut blockmap = BlockMap::new((SUPERBLOCK_SIZE / io.block_size) as u64, superblock.block_count, block_size);
        blockmap.write_full(&mut io);

        let inode_index = blockmap.last_block + 1;
        let inode_table = InodeTable::create(inode_index, &mut io);
        for i in 0..inode_table.block_count {
            blockmap.mark_used(&mut io, inode_index + i as u64);
        }
        superblock.set_inode_count(&mut io, inode_table.inode_count);

        FS::mount(io.device)
    }

    pub fn mount(device: A) -> FS<A> {
        match SuperBlock::read(&device) {
            Some(superblock) => {
                let io = IO::new(device, superblock.block_size);
                let blockmap = BlockMap::read(&io, (SUPERBLOCK_SIZE / io.block_size) as u64);
                let inode_table = InodeTable::read(&io, blockmap.last_block + 1, superblock.inode_count);
                FS { io, superblock, blockmap, inode_table }
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
    use crate::driver::file_drive::FileDrive;
    use crate::superblock::SuperBlock;

    #[test]
    fn init() {
        let drive = FileDrive::new("./test-images/fs_init.img", 1024 * 512, 512);
        let fs = super::FS::new(drive, 512);
        assert_eq!(fs.superblock, SuperBlock::new(512, 1024));
    }
}
