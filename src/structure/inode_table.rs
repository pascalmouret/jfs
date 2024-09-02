use std::marker::PhantomData;
use crate::consts::{BlockPointer, BLOCKS_PER_INODE_MAP, DIRECT_POINTERS, DirectPointers, InodePointer};
use crate::driver::DeviceDriver;
use crate::io::IO;
use crate::structure::inode::{Inode};

const EMPTY_POINTERS: DirectPointers = [0 as BlockPointer; DIRECT_POINTERS];

pub struct InodeTable {
    map: Vec<u8>,
    map_index: u64,
    pub(crate) inode_count: u64,
    table_index: u64,
    pub(crate) block_count: usize,
}

impl InodeTable {
    pub fn create(index: BlockPointer, io: &mut IO) -> InodeTable {
        let inode_count = InodeTable::calculate_inode_count(io.get_block_count(), io.get_block_size());
        let map_blocks = inode_count / 8 / io.get_block_size() as u64;
        let inode_per_block = io.get_block_size() as u64 / Inode::size_on_disk() as u64;
        let mut table_blocks = inode_count / inode_per_block;
        if inode_count % inode_per_block != 0 {
            table_blocks += 1;
        }
        let total_blocks = map_blocks + table_blocks;
        for i in 0..total_blocks {
            io.write_block(index + i, &vec![0; io.get_block_size()]);
        }
        InodeTable { map: vec![0u8; (inode_count / 8u64) as usize], map_index: index, inode_count, table_index: index + map_blocks, block_count: total_blocks as usize }
    }

    pub fn read(io: &IO, index: BlockPointer, inode_count: u64) -> InodeTable {
        let map_blocks = inode_count / 8 / io.get_block_size() as u64;
        let inodes_per_block = io.get_block_size() / Inode::size_on_disk();
        let mut table_blocks = inode_count / inodes_per_block as u64;
        if inode_count % inodes_per_block as u64 != 0 {
            table_blocks += 1;
        }
        let total_blocks = map_blocks + table_blocks;
        let map = InodeTable::read_map(io, index, inode_count);
        InodeTable { map, map_index: index, inode_count, table_index: index + map_blocks, block_count: total_blocks as usize }
    }

    pub fn read_inode(&self, io: &IO, index: InodePointer) -> Inode {
        let inode_block = self.inode_block(index, io.get_block_size());
        let offset = Self::inode_offset(index, io.get_block_size());

        let mut block = io.read_block(inode_block);
        let mut buffer = vec![0u8; Inode::size_on_disk()];
        buffer.copy_from_slice(&block[offset..offset + Inode::size_on_disk()]);
        let mut inode = Inode::from_bytes(index, &buffer, io.get_block_size());
        inode
    }

    pub fn write_inode(&mut self, io: &mut IO, inode: &mut Inode) {
        match inode.id {
            None => {
                let index = self.allocate(io).unwrap();
                inode.set_id(index);
                self.write_inode(io, inode);
            }
            Some(index) => {
                let inode_block = self.inode_block(index, io.get_block_size());
                let offset = Self::inode_offset(index, io.get_block_size());

                let mut block = io.read_block(inode_block);
                block[offset..offset + Inode::size_on_disk()].copy_from_slice(inode.to_bytes().as_slice());
                io.write_block(inode_block, &block);
            }
        }
    }

    #[inline]
    fn inode_block(&self, index: InodePointer, block_size: usize) -> BlockPointer {
        self.table_index + (index / (block_size / Inode::size_on_disk()) as u64)
    }

    #[inline]
    fn inode_offset(index: InodePointer, block_size: usize) -> usize {
        (index % (block_size / Inode::size_on_disk()) as u64) as usize * Inode::size_on_disk()
    }

    fn allocate(&mut self, io: &mut IO) -> Option<InodePointer> {
        for i in 0..self.map.len() {
            for j in 0..8 {
                if self.map[i] & (1 << j) == 0 {
                    self.mark_used(io, (i * 8 + j) as u64);
                    return Some((i * 8 + j) as u64);
                }
            }
        }
        None
    }

    fn mark_used_mem(&mut self, index: InodePointer) {
        let byte = index / 8;
        let bit = index % 8;
        self.map[byte as usize] |= 1 << bit;
    }

    fn mark_used(&mut self, io: &mut IO, index: InodePointer) {
        self.mark_used_mem(index);
        self.write_map(io);
    }

    fn mark_free_mem(&mut self, index: u64) {
        let byte = index / 8;
        let bit = index % 8;
        self.map[byte as usize] &= !(1 << bit);
    }

    fn mark_free(&mut self, io: &mut IO, index: InodePointer) {
        self.mark_free_mem(index);
        self.write_map(io);
    }

    fn calculate_inode_count(block_count: u64, block_size: usize) -> u64 {
        let bits_per_block = (block_size * 8) as u64;
        let blocks = block_count / BLOCKS_PER_INODE_MAP as u64;
        return blocks * bits_per_block;
    }

    fn read_map(io: &IO, index: BlockPointer, inode_count: u64) -> Vec<u8> {
        let mut map = vec![0u8; (inode_count / 8u64) as usize];
        let map_blocks = inode_count / 8 / io.get_block_size() as u64;
        for i in 0..map_blocks as usize {
            let block = io.read_block(index + i as u64);
            map[i * io.get_block_size()..(i + 1) * io.get_block_size()].copy_from_slice(&block);
        }
        map
    }

    // TODO: optimize this
    // - only write affected blocks
    // - cache some values
    fn write_map(&self, io: &mut IO) {
        let blocks = self.map.len() / io.get_block_size();
        for i in 0..blocks {
            let start = i * 512;
            let end = start + 512;
            let block = self.map[start..end].to_vec();
            io.write_block(self.map_index + i as u64, &block);
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::driver::file_drive::FileDrive;
    use crate::io::IO;
    use crate::structure::inode::Inode;

    #[test]
    fn read_write_table() {
        let drive = FileDrive::new("./test-images/structure_inode_read_write_table.img", 2048 * 512, 512);
        let mut io = IO::new(drive, 512);

        let new_table = super::InodeTable::create(1, &mut io);
        assert_eq!(new_table.map.len(), 512);
        assert_eq!(new_table.map_index, 1);
        assert_eq!(new_table.inode_count, 512 * 8);
        assert_eq!(new_table.table_index, 2);
        assert_eq!(new_table.block_count, 1025);

        let inode_table = super::InodeTable::read(&mut io, 1, new_table.inode_count);
        assert_eq!(inode_table.map.len(), 512);
        assert_eq!(inode_table.map_index, 1);
        assert_eq!(inode_table.inode_count, 512 * 8);
        assert_eq!(inode_table.table_index, 2);
        assert_eq!(inode_table.block_count, 1025);
    }

    #[test]
    fn read_write_inode() {
        let drive = FileDrive::new("./test-images/structure_inode_read_write_node.img", 2048 * 512, 512);
        let mut io = IO::new(drive, 512);

        let mut inode_table = super::InodeTable::create(1, &mut io);
        let mut memory_inode = Inode::new();
        inode_table.write_inode(&mut io, &mut memory_inode);
        let mut fs_inode = inode_table.read_inode(&mut io, memory_inode.id.unwrap());
        assert_eq!(memory_inode.to_bytes(), fs_inode.to_bytes());

        // TODO: manipulate metadata

        inode_table.write_inode(&mut io, &mut memory_inode);
        fs_inode = inode_table.read_inode(&io, memory_inode.id.unwrap());
        assert_eq!(memory_inode.to_bytes(), fs_inode.to_bytes());
    }
}
