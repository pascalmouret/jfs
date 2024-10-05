use std::ffi::OsStr;
use std::time::Duration;
use fuser::{FileAttr, FileType, Filesystem, ReplyEntry, Request};
use libc::c_int;

use crate::driver::file_drive::FileDrive;
use crate::ops::JourneyFS;
use crate::structure::inode::{Inode, INODE_ID};
use crate::ops::meta::{InodeType, Metadata};
use crate::util::mode::{ModeBits, ModeBitsHelper};

struct FuseDriver {
    journey_fs: JourneyFS,
}

impl Filesystem for FuseDriver {
    fn init(&mut self, _req: &Request<'_>, _config: &mut fuser::KernelConfig) -> Result<(), c_int> {
        let device = FileDrive::new("./test-images/test_drive.img", 20 * 1024 * 1024, 512);
        self.journey_fs = JourneyFS::new(device, _req.uid(), _req.gid(), 512);
        Ok(())
    }

    fn mkdir(
        &mut self,
        _req: &Request<'_>,
        parent: INODE_ID,
        name: &OsStr,
        mode: ModeBits,
        umask: u32,
        reply: ReplyEntry,
    ) {
        // TODO: apply umask if necessary
        let permissions = mode.get_permissions();
        let directory = self.journey_fs.mkdir(parent, &name.try_into().unwrap(), _req.uid(), _req.gid(), permissions);
        reply.entry(&Duration::new(100, 0), &self.inode_to_fileattr(directory.inode), 0);
    }
}

impl FuseDriver {
    fn inode_to_fileattr(&self,  inode: Inode<Metadata>) -> FileAttr {
        FileAttr {
            ino: inode.id.unwrap() as u64,
            size: inode.size,
            blocks: inode.used_pointers as u64,
            atime: inode.meta.accessed_at,
            mtime: inode.meta.modified_at,
            ctime: inode.meta.changed_at,
            crtime: inode.meta.created_at,
            kind: match inode.meta.inode_type {
                InodeType::File => FileType::RegularFile,
                InodeType::Directory => FileType::Directory,
            },
            perm: inode.meta.permissions,
            nlink: inode.meta.nlinks,
            uid: inode.meta.user_id,
            gid: inode.meta.group_id,
            rdev: inode.meta.rdev,
            flags: inode.meta.flags,
            blksize: self.journey_fs.get_block_size() as u32,
        }
    }
}
