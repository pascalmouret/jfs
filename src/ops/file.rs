use crate::ops::meta::Metadata;
use crate::structure::inode::{Inode, INODE_ID};
use crate::structure::Structure;

struct File {
    inode: Inode<Metadata>,
}

impl File {
    pub fn new(meta: Metadata, structure: &mut Structure<Metadata>) -> File {
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
