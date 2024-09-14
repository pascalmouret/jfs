use crate::structure::inode::ByteSerializable;

pub enum InodeType {
    File,
    Directory,
}

pub struct Metadata {
    pub inode_type: InodeType,
}

impl ByteSerializable for Metadata {
    fn to_bytes(&self) -> Vec<u8> {
        match self.inode_type {
            InodeType::File => vec![0],
            InodeType::Directory => vec![1],
        }
    }

    fn from_bytes(bytes: &[u8]) -> Self {
        let inode_type = match bytes[0] {
            0 => InodeType::File,
            1 => InodeType::Directory,
            _ => panic!("Invalid inode type"),
        };

        Metadata { inode_type }
    }
}
