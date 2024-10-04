use crate::ops::meta::{InodeType, Metadata};
use crate::structure::inode::{Inode};
use crate::structure::Structure;

pub struct File {
    pub inode: Inode<Metadata>,
}

impl File {
    pub fn new(structure: &mut Structure<Metadata>, permissions: u16) -> File {
        let meta = Metadata::new(InodeType::File, permissions, 1, 0);
        let inode = structure.create_inode(meta);
        File {
            inode,
        }
    }

    pub fn from_inode(inode: Inode<Metadata>) -> File {
        File {
            inode,
        }
    }

    pub fn set_data(&mut self, structure: &mut Structure<Metadata>, data: Vec<u8>) {
        self.inode.set_data(structure, data);
    }

    pub fn get_data(&self, structure: &Structure<Metadata>) -> Vec<u8> {
        self.inode.get_data(structure)
    }
}
