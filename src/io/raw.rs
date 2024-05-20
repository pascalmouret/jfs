use std::ops::Range;
use crate::consts::BlockPointer;
use crate::driver::DeviceDriver;

pub(crate) fn raw_write_block<A: DeviceDriver>(drive: &mut A, block_size: usize, data: &Vec<u8>, index: BlockPointer) {
    if block_size == drive.get_sector_size() {
        drive.write_sector(index, data);
    } else {
        let ratio = (block_size / drive.get_sector_size()) as u64;
        let start = index * ratio;
        let end = start + ratio;

        for i in start..end {
            let offset = (i - start) as usize * drive.get_sector_size();
            let limit = offset + drive.get_sector_size();
            println!("Writing sector {} - Offset {} :: Limit {}", i, offset, limit);
            drive.write_sector(i, &data[(offset..limit) as Range<usize>].to_vec())
        }
    }
}

pub(crate) fn raw_read_block<A: DeviceDriver>(drive: &A, block_size: usize, index: BlockPointer) -> Vec<u8> {
    if block_size == drive.get_sector_size() {
        drive.read_sector(index)
    } else {
        let ratio = (block_size / drive.get_sector_size()) as u64;
        let mut buffer = Vec::new();

        let start = index * ratio;
        let end = start + (block_size / drive.get_sector_size()) as u64;

        if block_size >= drive.get_sector_size() {
            for i in start..end {
                buffer.append(&mut drive.read_sector(i));
            }
        }

        buffer
    }
}
