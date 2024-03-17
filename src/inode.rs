pub struct Inode {
    pub first_block: u64,
    pub last_block: u64,
    pub data: Vec<u8>,
}
