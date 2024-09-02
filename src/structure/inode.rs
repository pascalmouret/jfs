use std::mem::size_of;
use crate::consts::{BlockPointer, DIRECT_POINTERS};
use crate::consts::DirectPointers;
use crate::structure::Structure;

const DATA_SIZE: usize = 96;
const NULL_POINTER: BlockPointer = 0;


pub type INODE_ID = u64;

// TODO: add metadata support
// TODO: probably doesn't need public members
pub struct Inode {
    pub(crate) id: Option<INODE_ID>,
    pub(crate) pointers: DirectPointers,
    pub(crate) size: u64,
    pub(crate) used_pointers: usize,
    pub(crate) allocated_size: u64,
}

impl Inode {
    pub fn new() -> Inode {
        Inode { id: None, pointers: [NULL_POINTER; 12], size: 0, used_pointers: 0, allocated_size: 0 }
    }

    pub fn set_id(&mut self, id: INODE_ID) {
        self.id = Some(id);
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::<u8>::new();
        bytes.extend_from_slice(&self.size.to_le_bytes());
        bytes.extend_from_slice(Inode::pointers_to_bytes(self.pointers).as_slice());
        bytes
    }

    pub fn from_bytes(id: INODE_ID, bytes: &Vec<u8>, block_size: usize) -> Self {
        let (size_bytes, pointers) = bytes.as_slice().split_at(size_of::<u64>());
        let size = u64::from_le_bytes(size_bytes.try_into().unwrap());
        let pointers = Inode::bytes_to_pointers(pointers);

        Inode {
            id: Some(id),
            size,
            pointers,
            used_pointers: Inode::count_used_pointers(&pointers),
            allocated_size: Inode::calculate_allocated_size(Inode::count_used_pointers(&pointers), block_size)
        }
    }

    #[inline]
    pub fn size_on_disk() -> usize {
        size_of::<u64>() + size_of::<DirectPointers>()
    }

    pub fn set_data(&mut self, structure: &mut Structure, data: Vec<u8>) {
        self.ensure_size(structure, data.len() as u64);
        let chunks = data.chunks(structure.get_block_size());
        for (i, chunk) in chunks.enumerate() {
            let block = self.pointers[i];
            structure.io.write_block(block, &chunk.to_vec());
        }
    }

    pub fn get_data(&self, structure: &mut Structure) -> Vec<u8> {
        let mut result = Vec::<u8>::new();

        for i in 0..self.used_pointers {
            result.append(&mut structure.read_block(self.pointers[i]));
        }

        return result[0..self.size as usize].to_vec();
    }

    pub fn append_data(&mut self, data: Vec<u8>) {
        unimplemented!();
    }

    fn count_used_pointers(pointers: &DirectPointers) -> usize {
        let mut count = 0;

        for i in 0..DIRECT_POINTERS {
            if pointers[i] != NULL_POINTER {
                count += 1;
            } else {
                break;
            }
        }

        count
    }

    fn calculate_allocated_size(used_pointers: usize, block_size: usize) -> u64 {
        used_pointers as u64 * block_size as u64
    }

    fn bytes_to_pointers(data: &[u8]) -> DirectPointers {
        let mut pointers = [NULL_POINTER; 12];
        let mut data = data;
        for i in 0..DIRECT_POINTERS {
            let mut bytes = [0u8; 8];
            bytes.copy_from_slice(&data[i * 8..8 + i * 8]);
            pointers[i] = u64::from_le_bytes(bytes);
        }
        pointers
    }

    fn pointers_to_bytes(pointers: DirectPointers) -> Vec<u8> {
        let mut data = Vec::<u8>::new();
        for i in 0..12 {
            data.extend_from_slice(&pointers[i].to_le_bytes());
        }
        data
    }

    fn ensure_size(&mut self, structure: &mut Structure, new_size: u64) {
        if new_size > (structure.get_block_size() as u64 * DIRECT_POINTERS as u64) {
            panic!("File cannot be larger than {} bytes", structure.get_block_size() * DIRECT_POINTERS);
        }

        let mut target_pointer_count = new_size / structure.get_block_size() as u64;

        if (new_size % structure.get_block_size() as u64) > 0 {
            target_pointer_count += 1;
        }

        if self.used_pointers < target_pointer_count as usize {
            while self.used_pointers < target_pointer_count as usize {
                self.allocate_block(structure);
            }
        }

        if self.used_pointers > target_pointer_count as usize {
            while self.used_pointers > target_pointer_count as usize {
                self.deallocate_block(structure);
            }
        }

        self.size = new_size;
    }

    fn allocate_block(&mut self, structure: &mut Structure) -> BlockPointer {
        if self.used_pointers >= DIRECT_POINTERS {
            panic!("All pointers are used");
        }

        let block = structure.allocate_block().unwrap();
        self.pointers[self.used_pointers] = block;
        self.used_pointers += 1;
        self.allocated_size = Inode::calculate_allocated_size(self.used_pointers, structure.get_block_size());
        block
    }

    fn deallocate_block(&mut self, structure: &mut Structure) {
        let block = self.pointers[self.used_pointers - 1];
        structure.block_map.mark_free(&mut structure.io, block);
        self.pointers[self.used_pointers - 1] = NULL_POINTER;
        self.used_pointers -= 1;
        self.allocated_size = Inode::calculate_allocated_size(self.used_pointers, structure.get_block_size());
    }
}

#[cfg(test)]
mod tests {
    use crate::structure::inode::Inode;
    use crate::structure::Structure;
    use crate::driver::file_drive::FileDrive;

    #[test]
    fn test_inode_to_bytes() {
        let inode = Inode {
            id: Some(42),
            pointers: [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12],
            size: 12 * 512,
            used_pointers: 12,
            allocated_size: 12 * 512,
        };

        let bytes = inode.to_bytes();
        assert_eq!(bytes.len(), Inode::size_on_disk());
    }

    #[test]
    fn test_inode_from_bytes() {
        let inode = Inode {
            id: Some(42),
            pointers: [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12],
            size: 12 * 512,
            used_pointers: 12,
            allocated_size: 12 * 512,
        };

        let bytes = inode.to_bytes();
        let new_inode = Inode::from_bytes(42, &bytes, 512);
        assert_eq!(new_inode.id, Some(42));
        assert_eq!(new_inode.size, 12 * 512);
        assert_eq!(new_inode.used_pointers, 12);
        assert_eq!(new_inode.pointers, [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12]);
        assert_eq!(new_inode.allocated_size, Inode::calculate_allocated_size(new_inode.used_pointers, 512));
    }

    #[test]
    fn test_inode_data() {
        let drive = FileDrive::new("./test-images/test_inode_data.img", 2048 * 512, 512);
        let mut structure = Structure::new(drive, 512);

        let mut inode = Inode::new();
        let data = vec![0; 512 * 12];
        inode.set_data(&mut structure, data.clone());
        let read_data = inode.get_data(&mut structure);

        assert_eq!(data, read_data);
    }
}
