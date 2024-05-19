use crate::consts::SUPERBLOCK_SIZE;
use crate::emu::HardDrive;
use crate::fsio::FSIO;

const MAGIC: u32 = 0xdeadbeef;

#[derive(Debug, PartialEq)]
pub struct SuperBlock {
    pub magic: u32,
    pub block_size: usize,
    pub block_count: u64,
    pub inode_count: u64,
}

impl SuperBlock {
    pub fn new(block_size: usize, block_count: u64) -> SuperBlock {
        SuperBlock { magic: MAGIC, block_size, block_count, inode_count: 0 }
    }

    pub fn set_inode_count(&mut self, fsio: &FSIO, inode_count: u64) {
        self.inode_count = inode_count;
        self.write(fsio);
    }

    // note: we can't use block reading since we don't know the block size yet
    pub fn read(drive: &HardDrive) -> Option<SuperBlock> {
        let mut buffer = drive.read_sector(0);

        if u32::from_le_bytes([buffer[0], buffer[1], buffer[2], buffer[3]]) != MAGIC {
            return None;
        }

        return if drive.sector_size >= SUPERBLOCK_SIZE {
            Some(SuperBlock::from_buffer(&buffer))
        } else {
            let block_count = SUPERBLOCK_SIZE / drive.sector_size;
            for i in 1..(block_count - 1) {
                buffer.append(&mut drive.read_sector(i as u64))
            }
            Some(SuperBlock::from_buffer(&buffer))
        }
    }

    fn from_buffer(buffer: &Vec<u8>) -> SuperBlock {
        let magic = u32::from_le_bytes([buffer[0], buffer[1], buffer[2], buffer[3]]);
        let block_size = u32::from_le_bytes([buffer[4], buffer[5], buffer[6], buffer[7]]) as usize;
        let block_count = u64::from_le_bytes([buffer[8], buffer[9], buffer[10], buffer[11], buffer[12], buffer[13], buffer[14], buffer[15]]);
        let inode_count = u64::from_le_bytes([buffer[24], buffer[25], buffer[26], buffer[27], buffer[28], buffer[29], buffer[30], buffer[31]]);
        SuperBlock { magic, block_size, block_count, inode_count }
    }

    fn to_buffer(&self) -> Vec<u8> {
        let mut buffer = Vec::new();
        buffer.extend_from_slice(&self.magic.to_le_bytes());
        buffer.extend_from_slice(&(self.block_size as u32).to_le_bytes());
        buffer.extend_from_slice(&self.block_count.to_le_bytes());
        buffer.extend_from_slice(&self.inode_count.to_le_bytes());
        buffer
    }

    pub fn write(&self, fsio: &FSIO) {
        let mut buffer = self.to_buffer();
        buffer.append(&mut vec![0; self.block_size - buffer.len()]);
        fsio.write_block(0, &buffer);
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn read_write_superblock() {
        let drive = super::HardDrive::new("./test-images/test_superblock.img", 1024 * 512, 512);
        let fsio = super::FSIO::new(drive, 512);
        let superblock = super::SuperBlock::new(512, 1024);
        superblock.write(&fsio);
        let drive_superblock = super::SuperBlock::read(&fsio.drive).unwrap();
        assert_eq!(superblock, drive_superblock);
    }
}
