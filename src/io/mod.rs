use std::ops::Range;
use crate::consts::BlockPointer;
use crate::driver::DeviceDriver;

pub(crate) struct IO {
    pub drive: Box<dyn DeviceDriver>,
    pub block_size: usize,
    pub block_count: u64,
}

impl IO {
    pub(crate) fn new<D: DeviceDriver + 'static>(drive: D, block_size: usize) -> IO {
        let block_count = drive.get_size() / block_size as u64;
        IO { drive: Box::new(drive), block_size, block_count }
    }

    pub(crate) fn get_block_size(&self) -> usize {
        self.block_size
    }

    pub(crate) fn get_block_count(&self) -> u64 {
        self.block_count
    }

    pub(crate) fn set_block_size(&mut self, block_size: usize) {
        self.block_size = block_size;
        self.block_count = self.drive.get_size() / block_size as u64;
    }

    pub(crate) fn get_sector_size(&self) -> usize {
        self.drive.get_sector_size()
    }

    pub(crate) fn get_sector_count(&self) -> u64 {
        self.drive.get_sector_count()
    }

    pub(crate) fn write_block(&mut self, index: BlockPointer, block: &Vec<u8>) {
        if block.len() != self.block_size {
            panic!("Block size mismatch");
        }

        if index >= self.block_count {
            panic!("Block index out of range");
        }

        if self.block_size == self.drive.get_sector_size() {
            self.drive.write_sector(index, block);
        } else {
            let ratio = (self.block_size / self.drive.get_sector_size()) as u64;
            let start = index * ratio;
            let end = start + ratio;

            for i in start..end {
                let offset = (i - start) as usize * self.drive.get_sector_size();
                let limit = offset + self.drive.get_sector_size();
                println!("Writing sector {} - Offset {} :: Limit {}", i, offset, limit);
                self.drive.write_sector(i, &block[(offset..limit) as Range<usize>].to_vec())
            }
        }
    }

    pub(crate) fn read_block(&self, index: BlockPointer) -> Vec<u8> {
        if index >= self.block_count {
            panic!("Block index out of range");
        }

        if self.block_size == self.drive.get_sector_size() {
            self.drive.read_sector(index)
        } else {
            let ratio = (self.block_size / self.drive.get_sector_size()) as u64;
            let mut buffer = Vec::new();

            let start = index * ratio;
            let end = start + (self.block_size / self.drive.get_sector_size()) as u64;

            if self.block_size >= self.drive.get_sector_size() {
                for i in start..end {
                    buffer.append(&mut self.drive.read_sector(i));
                }
            }

            buffer
        }
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
