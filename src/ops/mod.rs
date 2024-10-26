use crate::driver::DeviceDriver;
use crate::io::IO;
use crate::ops::directory::Directory;
use crate::ops::meta::{GroupId, Metadata, UserId};
use crate::structure::inode::{Inode, INODE_ID};
use crate::structure::Structure;
use crate::util::error::Error;
use std::ffi::OsString;

mod directory;
mod file;
pub mod meta;

pub struct JourneyFS {
    structure: Structure<Metadata>,
    root: Directory,
}

impl JourneyFS {
    // TODO: "new" should probably not mount existing filesystems
    pub fn new<D: DeviceDriver + 'static>(
        device: D,
        user_id: UserId,
        group_id: GroupId,
        block_size: usize,
    ) -> Result<JourneyFS, Error> {
        let mut io = IO::new(device, block_size);

        if Structure::<Metadata>::is_initialized(&mut io) {
            let structure = Structure::mount(io);
            let root = Directory::from_inode(structure.get_root_inode());
            Ok(JourneyFS { structure, root })
        } else {
            let mut structure = Structure::new(io, block_size);
            let root = Directory::new(&mut structure, user_id, group_id, 0o755);
            Ok(JourneyFS { structure, root })
        }
    }

    pub fn get_block_size(&self) -> Result<usize, Error> {
        Ok(self.structure.get_block_size())
    }

    pub fn mkdir(
        &mut self,
        parent: INODE_ID,
        name: &OsString,
        user_id: UserId,
        group_id: GroupId,
        permissions: u16,
    ) -> Result<Directory, Error> {
        let parent_inode = self.structure.read_inode(parent);
        let mut parent_directory = Directory::from_inode(parent_inode);
        Ok(parent_directory.add_directory(
            &mut self.structure,
            name,
            user_id,
            group_id,
            permissions,
        ))
    }

    pub fn get_inode(&self, id: INODE_ID) -> Result<Inode<Metadata>, Error> {
        Ok(self.structure.read_inode(id))
    }

    pub fn write_inode(&mut self, inode: &mut Inode<Metadata>) -> Result<(), Error> {
        Ok(self.structure.write_inode(inode))
    }
}
