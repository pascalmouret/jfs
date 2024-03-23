use std::mem::size_of;
use crate::consts::{BlockPointer, DirectPointers, InodePointer};

#[repr(u8)]
#[derive(PartialEq, Debug, Copy, Clone)]
pub enum InodeKind {
    Directory,
    File,
}

#[derive(PartialEq, Debug, Copy, Clone)]
#[repr(C, packed)]
pub struct Inode {
    pub kind: InodeKind,
    pub id: InodePointer,
    pointers: DirectPointers,
}

impl Inode {
    pub fn new(tpe: InodeKind, id: InodePointer, pointers: [BlockPointer; 12]) -> Inode {
        Inode { kind: tpe, pointers, id }
    }

    pub fn to_bytes(&self) -> &[u8; size_of::<Inode>()] {
        unsafe { &*(self as *const Inode as *const [u8; size_of::<Inode>()]) }
    }

    pub fn from_bytes(bytes: [u8; size_of::<Inode>()]) -> Inode {
        unsafe { *(bytes.as_ptr() as *const Inode) }
    }
}
