use crate::consts::BlockPointer;
use crate::driver::DeviceDriver;
use raw::{raw_read_block, raw_write_block};

mod raw;

pub(crate) struct IO<A: DeviceDriver> {
    // TODO: make private
    pub(crate) device: A,
    pub block_size: usize,
    pub block_count: u64,
}

impl <A: DeviceDriver>IO<A> {
    pub fn new(device: A, block_size: usize) -> IO<A> {
        let block_count = device.get_size() / block_size as u64;
        IO { device, block_size, block_count }
    }

    pub(crate) fn write_block(&mut self, index: BlockPointer, block: &Vec<u8>) {
        if block.len() != self.block_size {
            panic!("Block size mismatch");
        }

        if index >= self.block_count {
            panic!("Block index out of range");
        }

        raw_write_block(&mut self.device, self.block_size, block, index);
    }

    pub(crate) fn read_block(&self, index: BlockPointer) -> Vec<u8> {
        if index >= self.block_count {
            panic!("Block index out of range");
        }

        raw_read_block(&self.device, self.block_size, index)
    }
}

#[cfg(test)]
mod tests {
    use crate::driver::file_drive::FileDrive;

    #[test]
    fn read_write() {
        let drive = FileDrive::new("./test-images/fsio_read_write.img", 1024 * 512, 1024);
        let mut io = super::IO::new(drive, 1024);

        let block = vec![42; 1024];
        io.write_block(0, &block);
        let read = io.read_block(0);

        assert_eq!(block, read);
    }

    #[test]
    fn read_write_large_block() {
        let drive = FileDrive::new("./test-images/fsio_large_block.img", 1024 * 512, 512);
        let mut io = super::IO::new(drive, 1024);

        let block1 = vec![0x42; 1024];
        io.write_block(3, &block1);
        assert_eq!(io.read_block(3), block1);

        let block2 = vec![0x1; 1024];
        io.write_block(4, &block2);
        assert_eq!(io.read_block(4), block2);

        let block3 = vec![0x8; 1024];
        io.write_block(3, &block3);
        assert_eq!(io.read_block(3), block3);
    }
}
