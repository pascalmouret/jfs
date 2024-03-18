use std::mem::size_of;
use crate::consts::{BLOCKS_PER_INODE_MAP, DIRECT_POINTERS};
use crate::fs::FSIO;

#[repr(u8)]
#[derive(PartialEq, Debug, Copy, Clone)]
pub enum InodeType {
    Directory,
    File,
}

#[derive(PartialEq, Debug, Copy, Clone)]
#[repr(C, packed)]
pub struct Inode {
    tpe: InodeType,
    pointers: [u64; DIRECT_POINTERS],
}

impl Inode {
    pub fn new(tpe: InodeType, name: [u8; 255], pointers: [u64; 12]) -> Inode {
        if name.len() > 255 {
            panic!("Name too long");
        }

        Inode { tpe, pointers }
    }

    pub fn raw_ptr(&self) -> &[u8; size_of::<Inode>()] {
        unsafe { &*(self as *const Inode as *const [u8; size_of::<Inode>()]) }
    }

    pub fn from_raw_ptr(bytes: [u8; size_of::<Inode>()]) -> Inode {
        unsafe { *(bytes.as_ptr() as *const Inode) }
    }
}

pub struct InodeTable {
    map: Vec<u8>,
    map_index: u64,
    pub(crate) inode_count: u64,
    table_index: u64,
    pub(crate) block_count: usize,
}

impl InodeTable {
    pub fn create(index: u64, fsio: &FSIO) -> InodeTable {
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

    pub fn read(fsio: &FSIO, index: u64, inode_count: u64) -> InodeTable {
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

    pub fn create_inode(&mut self, fsio: &FSIO, inode: Inode) {
        let index = self.allocate(fsio).unwrap();
        let inodes_per_block = fsio.block_size / size_of::<Inode>();
        let inode_block = index / inodes_per_block as u64;

        let mut block = fsio.read_block(self.table_index + inode_block);
        let offset = (index % inodes_per_block as u64) as usize * size_of::<Inode>();
        block[offset..offset + size_of::<Inode>()].copy_from_slice(inode.raw_ptr());
        fsio.write_block(self.table_index + inode_block, &block);
    }

    pub fn read_inode(&self, fsio: &FSIO, index: u64) -> Inode {
        let inodes_per_block = fsio.block_size / size_of::<Inode>();
        let inode_block = index / inodes_per_block as u64;
        let block = fsio.read_block(self.table_index + inode_block);
        let offset = (index % inodes_per_block as u64) as usize * size_of::<Inode>();
        let mut buffer = [0u8; size_of::<Inode>()];
        buffer.copy_from_slice(&block[offset..offset + size_of::<Inode>()]);
        Inode::from_raw_ptr(buffer)
    }

    fn allocate(&mut self, fsio: &FSIO) -> Option<u64> {
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

    fn mark_used_mem(&mut self, index: u64) {
        let byte = index / 8;
        let bit = index % 8;
        self.map[byte as usize] |= 1 << bit;
    }

    fn mark_used(&mut self, fsio: &FSIO, index: u64) {
        self.mark_used_mem(index);
        self.write_map(fsio);
    }

    fn mark_free_mem(&mut self, index: u64) {
        let byte = index / 8;
        let bit = index % 8;
        self.map[byte as usize] &= !(1 << bit);
    }

    fn mark_free(&mut self, fsio: &FSIO, index: u64) {
        self.mark_free_mem(index);
        self.write_map(fsio);
    }

    fn calculate_inode_count(block_count: u64, block_size: usize) -> u64 {
        let bits_per_block = (block_size * 8) as u64;
        let blocks = block_count / BLOCKS_PER_INODE_MAP as u64;
        return blocks * bits_per_block;
    }

    fn read_map(fsio: &FSIO, index: u64, inode_count: u64) -> Vec<u8> {
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
    use crate::fs::{FSIO};

    #[test]
    fn read_write_table() {
        {
            let drive = HardDrive::new("inode_read_write_table.img", 1024 * 512, 512);
            let fsio = FSIO::new(drive, 512);

            let new_table = super::InodeTable::create(1, &fsio);
            assert_eq!(new_table.map.len(), 512);
            assert_eq!(new_table.map_index, 1);
            assert_eq!(new_table.inode_count, 512 * 8);
            assert_eq!(new_table.table_index, 2);
            assert_eq!(new_table.block_count, 821);

            let inode_table = super::InodeTable::read(&fsio, 1, new_table.inode_count);
            assert_eq!(inode_table.map.len(), 512);
            assert_eq!(inode_table.map_index, 1);
            assert_eq!(inode_table.inode_count, 512 * 8);
            assert_eq!(inode_table.table_index, 2);
            assert_eq!(inode_table.block_count, 821);
        }
        fs::remove_file("inode_read_write_table.img").unwrap();
    }

    #[test]
    fn store_inode() {
        {
            let drive = HardDrive::new("inode_read_write_node.img", 1024 * 512, 512);
            let fsio = FSIO::new(drive, 512);

            let mut inode_table = super::InodeTable::create(1, &fsio);
            let inode = super::Inode::new(super::InodeType::File, [0; 255], [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11]);
            inode_table.create_inode(&fsio, inode);
            let read_inode = inode_table.read_inode(&fsio, 0);
            assert_eq!(inode.raw_ptr(), read_inode.raw_ptr());
        }
        fs::remove_file("inode_read_write_node.img").unwrap();
    }
}
