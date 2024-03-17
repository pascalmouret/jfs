use crate::fs::FSIO;

pub struct BlockMap {
    first_block: u64,
    last_block: u64,
    data: Vec<u8>,
}

impl BlockMap {
    pub fn new(first_block: u64, block_count: u64, block_size: usize) -> BlockMap {
        let data = BlockMap::create_data(block_count, block_size);
        let last_block = first_block + data.len() as u64 / block_size as u64;
        let mut map = BlockMap { first_block, last_block, data };
        for i in 0..last_block {
            map.mark_used_mem(i)
        }
        map
    }

    pub fn read(fsio: &FSIO, index: u64) -> BlockMap {
        let mut data = BlockMap::create_data(fsio.block_count, fsio.block_size);
        let last_block = index + data.len() as u64 / fsio.block_size as u64;
        for i in index..last_block {
            let offset = (i as usize - index as usize) * fsio.block_size;
            let limit = (i as usize - index as usize + 1) * fsio.block_size;
            let block = fsio.read_block(i);
            data[offset..limit].copy_from_slice(&block);
        }
        BlockMap { first_block: index, last_block, data }
    }

    fn create_data(block_count: u64, block_size: usize) -> Vec<u8> {
        let mut data = vec![0; block_count as usize / 8];
        if (block_count as usize) % 8 != 0 {
            data.push(0);
        }
        if data.len() % block_size != 0 {
            data.append(&mut vec![0; block_size - (data.len() % block_size)]);
        }
        data
    }

    pub fn write_part(&self, fsio: &FSIO, including_index: u64) {
        let block = (including_index / fsio.block_size as u64 / 8) as usize;
        let data = &self.data[block * fsio.block_size..(block + 1) * fsio.block_size];
        fsio.write_block(self.first_block + block as u64, &data.to_vec());
    }

    pub fn write_full(&self, fsio: &FSIO) {
        for i in self.first_block..self.last_block {
            let offset = (i as usize - self.first_block as usize) * fsio.block_size;
            let limit = (i as usize - self.first_block as usize + 1) * fsio.block_size;
            fsio.write_block(i, &self.data[offset..limit].to_vec());
        }
    }

    pub fn allocate(&mut self, fsio: &FSIO) -> Option<u64> {
        for byte_index in 0..self.data.len() {
            let byte = self.data[byte_index];
            for j in 0..8 {
                if byte & (1 << j) == 0 {
                    let result = byte_index as u64 * 8 + j;
                    self.mark_used(fsio, result);
                    return Some(result);
                }
            }
        }
        None
    }

    fn is_free(&self, index: u64) -> bool {
        self.data[(index / 8) as usize] & (1 << (index % 8)) == 0
    }

    fn is_used(&self, index: u64) -> bool {
        !self.is_free(index)
    }

    fn mark_used_mem(&mut self, index: u64) {
        let byte_index = (index / 8) as usize;
        let bit_index = (index % 8) as usize;
        self.data[byte_index] |= 1 << bit_index;
    }

    fn mark_used(&mut self, fsio: &FSIO, index: u64) {
        self.mark_used_mem(index);
        self.write_part(fsio, index);
    }

    fn mark_free_mem(&mut self, index: u64) {
        let byte_index = (index / 8) as usize;
        let bit_index = (index % 8) as usize;
        self.data[byte_index] &= !(1 << bit_index);
    }

    fn mark_free(&mut self, fsio: &FSIO, index: u64) {
        self.mark_free_mem(index);
        self.write_part(fsio, index);
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use crate::fs::FSIO;
    use crate::emu::HardDrive;

    #[test]
    fn read_write() {
        {
            let drive = HardDrive::new("blockmap_read_write.img", 1024 * 512, 512);
            let fsio = FSIO::new(drive, 1024);
            let blockmap = super::BlockMap::new(1, 1024, 1024);
            blockmap.write_full(&fsio);
            assert_eq!(blockmap.data, super::BlockMap::read(&fsio, 1).data)
        }
        fs::remove_file("blockmap_read_write.img").unwrap();
    }

    #[test]
    fn allocate() {
        {
            let drive = HardDrive::new("blockmap_allocate.img", 1024 * 512, 512);
            let fsio = FSIO::new(drive, 1024);
            let mut blockmap = super::BlockMap::new(1, 1024, 1024);
            let index = blockmap.allocate(&fsio).unwrap();
            assert_eq!(blockmap.is_used(index), true);
            blockmap.mark_free(&fsio, index);
            assert_eq!(blockmap.is_free(index), true);
            assert_eq!(blockmap.data, super::BlockMap::read(&fsio, 1).data)
        }
        fs::remove_file("blockmap_allocate.img").unwrap();
    }
}
