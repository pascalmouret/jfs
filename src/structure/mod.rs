use crate::consts::SUPERBLOCK_SIZE;
use crate::driver::DeviceDriver;
use crate::fs::FS;
use crate::io::IO;
use crate::structure::blockmap::BlockMap;
use crate::structure::inode::Inode;
use crate::structure::superblock::SuperBlock;
use crate::structure::inode_table::InodeTable;

mod inode_table;
pub(crate) mod inode;
pub(crate) mod blockmap;
pub(crate) mod superblock;

pub struct Structure<INODE: Inode> {
    pub(crate) super_block: SuperBlock,
    block_map: BlockMap,
    inode_table: InodeTable<INODE>,
}

impl <INODE: Inode>Structure<INODE> {
    pub fn new(io: &mut IO, block_size: usize) -> Structure<INODE> {
        if block_size < io.get_sector_size() {
            panic!("Block size must be greater than or equal to sector size");
        }

        if block_size % io.get_sector_size() != 0 {
            panic!("Block size must be a multiple of sector size");
        }

        io.set_block_size(block_size);
        let mut superblock = SuperBlock::new(block_size, io.block_count);
        superblock.write(io);

        let mut blockmap = BlockMap::new((SUPERBLOCK_SIZE / io.get_block_size()) as u64, superblock.block_count, block_size);
        blockmap.write_full(io);

        let inode_index = blockmap.last_block + 1;
        let inode_table = InodeTable::<INODE>::create(inode_index, io);
        for i in 0..inode_table.block_count {
            blockmap.mark_used(io, inode_index + i as u64);
        }
        superblock.set_inode_count(io, inode_table.inode_count);

        Structure::mount(io)
    }

    pub fn mount(io: &mut IO) -> Structure<INODE> {
        match SuperBlock::read(io) {
            Some(super_block) => {
                io.set_block_size(super_block.block_size);
                let block_map = BlockMap::read(io, (SUPERBLOCK_SIZE / io.get_block_size()) as u64);
                let inode_table = InodeTable::<INODE>::read(io, block_map.last_block + 1, super_block.inode_count);
                Structure { super_block, block_map, inode_table }
            }
            None => panic!("No superblock found"),
        }
    }
}
