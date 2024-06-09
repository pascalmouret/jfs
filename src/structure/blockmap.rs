use crate::consts::BlockPointer;
use crate::driver::DeviceDriver;
use crate::io::IO;

pub struct BlockMap {
    pub(crate) first_block: BlockPointer,
    pub(crate) last_block: BlockPointer,
    data: Vec<u8>,
}

impl BlockMap {
    pub fn new(first_block: BlockPointer, block_count: u64, block_size: usize) -> BlockMap {
        let data = BlockMap::create_data(block_count, block_size);
        let last_block = first_block + data.len() as u64 / block_size as u64;
        let mut map = BlockMap { first_block, last_block, data };
        for i in 0..last_block + 1 {
            map.mark_used_mem(i)
        }
        map
    }

    pub fn read(io: &IO, index: BlockPointer) -> BlockMap {
        let mut data = BlockMap::create_data(io.get_block_count(), io.get_block_size());
        let last_block = index + data.len() as u64 / io.get_block_size() as u64;
        for i in index..last_block {
            let offset = (i as usize - index as usize) * io.get_block_size();
            let limit = (i as usize - index as usize + 1) * io.get_block_size();
            let block = io.read_block(i);
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

    pub fn write_part(&self, io: &mut IO, including_index: BlockPointer) {
        let block = (including_index / io.get_block_size() as u64 / 8) as usize;
        let data = &self.data[block * io.get_block_size()..(block + 1) * io.get_block_size()];
        io.write_block(self.first_block + block as u64, &data.to_vec());
    }

    pub fn write_full(&self, io: &mut IO) {
        for i in self.first_block..self.last_block {
            let offset = (i as usize - self.first_block as usize) * io.get_block_size();
            let limit = (i as usize - self.first_block as usize + 1) * io.get_block_size();
            io.write_block(i, &self.data[offset..limit].to_vec());
        }
    }

    pub fn allocate(&mut self, io: &mut IO) -> Option<u64> {
        for byte_index in 0..self.data.len() {
            let byte = self.data[byte_index];
            for j in 0..8 {
                if byte & (1 << j) == 0 {
                    let result = byte_index as u64 * 8 + j;
                    self.mark_used(io, result);
                    return Some(result);
                }
            }
        }
        None
    }

    fn is_free(&self, index: BlockPointer) -> bool {
        self.data[(index / 8) as usize] & (1 << (index % 8)) == 0
    }

    fn is_used(&self, index: BlockPointer) -> bool {
        !self.is_free(index)
    }

    fn mark_used_mem(&mut self, index: BlockPointer) {
        let byte_index = (index / 8) as usize;
        let bit_index = (index % 8) as usize;
        println!("Marking Used: {} {} {}", index, byte_index, bit_index);
        self.data[byte_index] |= 1 << bit_index;
    }

    pub(crate) fn mark_used(&mut self, io: &mut IO, index: BlockPointer) {
        self.mark_used_mem(index);
        self.write_part(io, index);
    }

    fn mark_free_mem(&mut self, index: BlockPointer) {
        let byte_index = (index / 8) as usize;
        let bit_index = (index % 8) as usize;
        println!("Marking Free: {} {} {}", index, byte_index, bit_index);
        self.data[byte_index] &= !(1 << bit_index);
    }

    pub(crate) fn mark_free(&mut self, io: &mut IO, index: BlockPointer) {
        self.mark_free_mem(index);
        self.write_part(io, index);
    }
}

#[cfg(test)]
mod tests {
    use crate::driver::file_drive::FileDrive;
    use crate::io::IO;

    #[test]
    fn read_write() {
        let drive = FileDrive::new("./test-images/blockmap_read_write.img", 1024 * 512, 512);
        let mut io = IO::new(drive, 1024);
        let blockmap = super::BlockMap::new(1, 1024, 1024);
        blockmap.write_full(&mut io);
        assert_eq!(blockmap.data, super::BlockMap::read(&io, 1).data)
    }

    #[test]
    fn allocate() {
        let drive = FileDrive::new("./test-images/blockmap_allocate.img", 1024 * 512, 512);
        let mut io = IO::new(drive, 1024);
        let mut blockmap = super::BlockMap::new(1, 1024, 1024);
        let index = blockmap.allocate(&mut io).unwrap();
        assert_eq!(blockmap.is_used(index), true);
        blockmap.mark_free(&mut io, index);
        assert_eq!(blockmap.is_free(index), true);
        assert_eq!(blockmap.data, super::BlockMap::read(&io, 1).data)
    }
}
