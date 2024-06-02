use std::mem::size_of;

pub type INODE_ID = u64;

pub(crate) trait Inode {
    fn id(&self) -> Option<INODE_ID>;
    fn set_id(&mut self, id: INODE_ID);
    fn to_bytes(&self) -> Vec<u8>;
    fn from_bytes(bytes: &Vec<u8>) -> Self;
    fn disk_size() -> usize;
}
