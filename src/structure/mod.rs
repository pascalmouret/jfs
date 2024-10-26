use crate::consts::{BlockPointer, SUPERBLOCK_SIZE};
use crate::io::IO;
use crate::structure::blockmap::BlockMap;
use crate::structure::inode::{Inode, InodeId};
use crate::structure::inode_table::InodeTable;
use crate::structure::superblock::SuperBlock;
use crate::util::format::pretty_size_from_bytes;
use crate::util::serializable::{ByteSerializable, KnownSize};

pub(crate) mod blockmap;
pub(crate) mod inode;
mod inode_table;
pub(crate) mod superblock;

pub struct Structure<META: ByteSerializable + KnownSize> {
    io: IO,
    pub(crate) super_block: SuperBlock,
    pub(crate) block_map: BlockMap,
    pub(crate) inode_table: InodeTable<META>,
}

impl<META: ByteSerializable + KnownSize> Structure<META> {
    pub fn is_initialized(io: &IO) -> bool {
        match SuperBlock::read(io) {
            Some(_) => true,
            None => false,
        }
    }

    pub fn new(mut io: IO, block_size: usize) -> Structure<META> {
        if block_size < io.get_sector_size() {
            panic!("Block size must be greater than or equal to sector size");
        }

        if block_size % io.get_sector_size() != 0 {
            panic!("Block size must be a multiple of sector size");
        }

        println!("Building structure...");
        println!("Inode size: {}", Inode::<META>::size_on_disk());
        println!(
            "Drive size: {}",
            pretty_size_from_bytes(
                io.drive.get_sector_size() as u64 * io.drive.get_sector_count() as u64
            )
        );
        println!("Sector size: {}", io.get_sector_size());
        println!("Sector count: {}", io.get_sector_count());
        println!("Block size: {}", block_size);
        println!("Block count: {}", io.block_count);

        io.set_block_size(block_size);
        let mut super_block = SuperBlock::new(block_size, io.block_count);
        super_block.write(&mut io);

        let mut block_map = BlockMap::new(
            (SUPERBLOCK_SIZE / io.get_block_size()) as u64,
            super_block.block_count,
            block_size,
        );
        block_map.write_full(&mut io);

        println!(
            "Block map size: {}",
            pretty_size_from_bytes(
                (block_map.last_block - block_map.first_block + 1) * block_size as u64
            )
        );
        println!(
            "Block map blocks: {}",
            block_map.last_block - block_map.first_block + 1
        );

        let inode_index = block_map.last_block + 1;
        let inode_table = InodeTable::create(inode_index, &mut io);
        for i in 0..inode_table.block_count {
            block_map.mark_used(&mut io, inode_index + i as u64);
        }
        super_block.set_inode_count(&mut io, inode_table.inode_count);

        println!(
            "Inode table size: {}",
            pretty_size_from_bytes(inode_table.block_count as u64 * block_size as u64)
        );
        println!("Inode table blocks: {}", inode_table.block_count);

        Structure {
            io,
            super_block,
            block_map,
            inode_table,
        }
    }

    pub fn mount(mut io: IO) -> Structure<META> {
        match SuperBlock::read(&mut io) {
            Some(super_block) => {
                io.set_block_size(super_block.block_size);
                let block_map =
                    BlockMap::read(&mut io, (SUPERBLOCK_SIZE / super_block.block_size) as u64);
                let inode_table =
                    InodeTable::read(&mut io, block_map.last_block + 1, super_block.inode_count);
                Structure {
                    io,
                    super_block,
                    block_map,
                    inode_table,
                }
            }
            None => panic!("No superblock found"),
        }
    }

    pub fn set_root_inode(&mut self, inode: &mut Inode<META>) {
        self.super_block
            .set_root_inode(&mut self.io, inode.id.unwrap());
    }

    pub fn get_root_inode(&self) -> Inode<META> {
        self.inode_table
            .read_inode(&self.io, self.super_block.root_inode)
    }

    pub fn create_inode(&mut self, meta: META) -> Inode<META> {
        let mut inode = Inode::new(meta);
        self.inode_table.write_inode(&mut self.io, &mut inode);
        inode
    }

    pub fn read_inode(&self, id: InodeId) -> Inode<META> {
        self.inode_table.read_inode(&self.io, id)
    }

    pub fn write_inode(&mut self, inode: &mut Inode<META>) {
        self.inode_table.write_inode(&mut self.io, inode);
    }

    pub fn get_block_size(&self) -> usize {
        self.super_block.block_size
    }

    pub fn allocate_block(&mut self) -> Option<BlockPointer> {
        self.block_map.allocate(&mut self.io)
    }

    pub fn write_block(&mut self, index: BlockPointer, block: &Vec<u8>) {
        self.io.write_block(index, block);
    }

    pub fn read_block(&self, index: BlockPointer) -> Vec<u8> {
        self.io.read_block(index)
    }
}
