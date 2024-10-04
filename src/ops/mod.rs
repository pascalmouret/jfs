use std::ffi::OsString;
use crate::driver::DeviceDriver;
use crate::io::IO;
use crate::ops::directory::Directory;
use crate::ops::meta::Metadata;
use crate::structure::inode::INODE_ID;
use crate::structure::Structure;

mod file;
pub mod meta;
mod directory;

pub struct JourneyFS {
    structure: Structure<Metadata>,
    root: Directory,
}

impl JourneyFS {
    pub fn new<D: DeviceDriver + 'static>(device: D, block_size: usize) -> JourneyFS {
        let mut io = IO::new(device, block_size);

        if Structure::<Metadata>::is_initialized(&mut io) {
            let structure = Structure::mount(io);
            let root = Directory::from_inode(structure.get_root_inode());
            JourneyFS {
                structure,
                root,
            }
        } else {
            let mut structure = Structure::new(io, block_size);
            let root = Directory::new(&mut structure, 0o755);
            JourneyFS {
                structure,
                root,
            }
        }
    }

    pub fn get_block_size(&self) -> usize {
        self.structure.get_block_size()
    }

    pub fn mkdir(&mut self, parent: INODE_ID, name: &OsString, permissions: u16) -> Directory {
        let parent_inode = self.structure.read_inode(parent);
        let mut parent_directory = Directory::from_inode(parent_inode);
        parent_directory.add_directory(&mut self.structure, name, permissions)
    }
}
