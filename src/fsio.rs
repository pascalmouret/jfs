use crate::consts::BlockPointer;
use crate::emu::HardDrive;
use crate::raw::{raw_read_block, raw_write_block};

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

#[cfg(test)]
mod tests {
    use crate::emu::HardDrive;

    #[test]
    fn read_write() {
        let drive = HardDrive::new("./test-images/fsio_read_write.img", 1024 * 512, 1024);
        let fsio = super::FSIO::new(drive, 1024);

        let block = vec![42; 1024];
        fsio.write_block(0, &block);
        let read = fsio.read_block(0);

        assert_eq!(block, read);
    }

    #[test]
    fn read_write_large_block() {
        let drive = HardDrive::new("./test-images/fsio_large_block.img", 1024 * 512, 512);
        let fs = super::FSIO::new(drive, 1024);

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
}
