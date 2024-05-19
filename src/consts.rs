pub(crate) const SUPERBLOCK_SIZE: usize = 1024;
pub(crate) const BLOCKS_PER_INODE_MAP: usize = 2048;
pub(crate) const DIRECT_POINTERS: usize = 12;
pub(crate) const FILE_NAME_LENGTH: usize = 255;

pub type BlockPointer = u64;
pub type InodePointer = u64;
pub type DirectPointers = [BlockPointer; DIRECT_POINTERS];
pub type FileName = [u8; FILE_NAME_LENGTH];
