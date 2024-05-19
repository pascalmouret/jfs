use std::mem::size_of;
use crate::consts::{BlockPointer, BLOCKS_PER_INODE_MAP, DIRECT_POINTERS, DirectPointers, InodePointer};
use crate::fsio::FSIO;
use crate::inode::{Inode, InodeKind};

const EMPTY_POINTERS: DirectPointers = [0 as BlockPointer; DIRECT_POINTERS];

pub struct InodeTable {
    map: Vec<u8>,
    map_index: u64,
    pub(crate) inode_count: u64,
    table_index: u64,
    pub(crate) block_count: usize,
}

impl InodeTable {
    pub fn create(index: BlockPointer, fsio: &FSIO) -> InodeTable {
        let inode_count = InodeTable::calculate_inode_count(fsio.block_count, fsio.block_size);
        let map_blocks = inode_count / 8 / fsio.block_size as u64;
        let inode_per_block = fsio.block_size as u64 / size_of::<Inode>() as u64;
        let mut table_blocks = inode_count / inode_per_block;
        if inode_count % inode_per_block != 0 {
            table_blocks += 1;
        }
        let total_blocks = map_blocks + table_blocks;
        for i in 0..total_blocks {
            fsio.write_block(index + i, &vec![0; fsio.block_size]);
        }
        InodeTable { map: vec![0u8; (inode_count / 8u64) as usize], map_index: index, inode_count, table_index: index + map_blocks, block_count: total_blocks as usize }
    }

    pub fn read(fsio: &FSIO, index: BlockPointer, inode_count: u64) -> InodeTable {
        let map_blocks = inode_count / 8 / fsio.block_size as u64;
        let inodes_per_block = fsio.block_size / size_of::<Inode>();
        let mut table_blocks = inode_count / inodes_per_block as u64;
        if inode_count % inodes_per_block as u64 != 0 {
            table_blocks += 1;
        }
        let total_blocks = map_blocks + table_blocks;
        let map = InodeTable::read_map(fsio, index, inode_count);
        InodeTable { map, map_index: index, inode_count, table_index: index + map_blocks, block_count: total_blocks as usize }
    }

    pub fn create_inode(&mut self, fsio: &FSIO, tpe: InodeKind) -> Inode {
        let index = self.allocate(fsio).unwrap();
        let inode = Inode::new(tpe, index, EMPTY_POINTERS);
        self.write_inode(fsio, inode);
        inode
    }

    pub fn read_inode(&self, fsio: &FSIO, index: InodePointer) -> Inode {
        let inode_block = self.inode_block(index, fsio.block_size);
        let offset = Self::inode_offset(index, fsio.block_size);

        let mut block = fsio.read_block(inode_block);
        let mut buffer = [0u8; size_of::<Inode>()];
        buffer.copy_from_slice(&block[offset..offset + size_of::<Inode>()]);
        Inode::from_bytes(buffer)
    }

    pub fn write_inode(&self, fsio: &FSIO, inode: Inode) {
        let inode_block = self.inode_block(inode.id, fsio.block_size);
        let offset = Self::inode_offset(inode.id, fsio.block_size);

        let mut block = fsio.read_block(inode_block);
        block[offset..offset + size_of::<Inode>()].copy_from_slice(inode.to_bytes());
        fsio.write_block(inode_block, &block);
    }

    #[inline]
    fn inode_block(&self, index: InodePointer, block_size: usize) -> BlockPointer {
        self.table_index + (index / (block_size / size_of::<Inode>()) as u64)
    }

    #[inline]
    fn inode_offset(index: InodePointer, block_size: usize) -> usize {
        (index % (block_size / size_of::<Inode>()) as u64) as usize * size_of::<Inode>()
    }

    fn allocate(&mut self, fsio: &FSIO) -> Option<InodePointer> {
        for i in 0..self.map.len() {
            for j in 0..8 {
                if self.map[i] & (1 << j) == 0 {
                    self.mark_used(fsio, (i * 8 + j) as u64);
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

    fn mark_used(&mut self, fsio: &FSIO, index: InodePointer) {
        self.mark_used_mem(index);
        self.write_map(fsio);
    }

    fn mark_free_mem(&mut self, index: u64) {
        let byte = index / 8;
        let bit = index % 8;
        self.map[byte as usize] &= !(1 << bit);
    }

    fn mark_free(&mut self, fsio: &FSIO, index: InodePointer) {
        self.mark_free_mem(index);
        self.write_map(fsio);
    }

    fn calculate_inode_count(block_count: u64, block_size: usize) -> u64 {
        let bits_per_block = (block_size * 8) as u64;
        let blocks = block_count / BLOCKS_PER_INODE_MAP as u64;
        return blocks * bits_per_block;
    }

    fn read_map(fsio: &FSIO, index: BlockPointer, inode_count: u64) -> Vec<u8> {
        let mut map = vec![0u8; (inode_count / 8u64) as usize];
        let map_blocks = inode_count / 8 / fsio.block_size as u64;
        for i in 0..map_blocks as usize {
            let block = fsio.read_block(index + i as u64);
            map[i * fsio.block_size..(i + 1) * fsio.block_size].copy_from_slice(&block);
        }
        map
    }

    // TODO: optimize this
    // - only write affected blocks
    // - cache some values
    fn write_map(&self, fsio: &FSIO) {
        let blocks = self.map.len() / fsio.block_size;
        for i in 0..blocks {
            let start = i * 512;
            let end = start + 512;
            let block = self.map[start..end].to_vec();
            fsio.write_block(self.map_index + i as u64, &block);
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use crate::emu::HardDrive;
    use crate::fsio::{FSIO};

    #[test]
    fn read_write_table() {
        let drive = HardDrive::new("./test-images/inode_read_write_table.img", 2048 * 512, 512);
        let fsio = FSIO::new(drive, 512);

        let new_table = super::InodeTable::create(1, &fsio);
        assert_eq!(new_table.map.len(), 512);
        assert_eq!(new_table.map_index, 1);
        assert_eq!(new_table.inode_count, 512 * 8);
        assert_eq!(new_table.table_index, 2);
        assert_eq!(new_table.block_count, 1025);

        let inode_table = super::InodeTable::read(&fsio, 1, new_table.inode_count);
        assert_eq!(inode_table.map.len(), 512);
        assert_eq!(inode_table.map_index, 1);
        assert_eq!(inode_table.inode_count, 512 * 8);
        assert_eq!(inode_table.table_index, 2);
        assert_eq!(inode_table.block_count, 1025);
    }

    #[test]
    fn read_write_inode() {
        let drive = HardDrive::new("./test-images/inode_read_write_node.img", 2048 * 512, 512);
        let fsio = FSIO::new(drive, 512);

        let mut inode_table = super::InodeTable::create(1, &fsio);
        let mut memory_inode = inode_table.create_inode(&fsio, super::InodeKind::File);
        let mut fs_inode = inode_table.read_inode(&fsio, 0);
        assert_eq!(memory_inode.to_bytes(), fs_inode.to_bytes());

        memory_inode.kind = super::InodeKind::Directory;
        memory_inode.set_pointers([0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11]);
        inode_table.write_inode(&fsio, memory_inode);
        fs_inode = inode_table.read_inode(&fsio, memory_inode.id);
        assert_eq!(memory_inode.to_bytes(), fs_inode.to_bytes());
    }
}
