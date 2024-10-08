use std::ffi::OsStr;
use std::time::{Duration, SystemTime};
use fuser::{FileAttr, FileType, Filesystem, ReplyAttr, ReplyEntry, Request, TimeOrNow};
use libc::c_int;

use crate::driver::file_drive::FileDrive;
use crate::ops::JourneyFS;
use crate::structure::inode::{Inode, INODE_ID};
use crate::ops::meta::{GroupId, InodeType, Metadata, UserId};
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

    fn getattr(&mut self, _req: &Request<'_>, ino: u64, reply: ReplyAttr) {
        // TODO: really need error handling
        let inode = self.journey_fs.get_inode(ino as INODE_ID);
        reply.attr(&Duration::new(100, 0), &self.inode_to_fileattr(inode));
    }

    fn setattr(
        &mut self,
        _req: &Request<'_>,
        ino: u64,
        mode: Option<ModeBits>,
        uid: Option<UserId>,
        gid: Option<GroupId>,
        size: Option<u64>,
        _atime: Option<TimeOrNow>,
        _mtime: Option<TimeOrNow>,
        _ctime: Option<SystemTime>,
        fh: Option<u64>,
        _crtime: Option<SystemTime>,
        _chgtime: Option<SystemTime>,
        _bkuptime: Option<SystemTime>,
        flags: Option<u32>,
        reply: ReplyAttr,
    ) {
        let mut inode = self.journey_fs.get_inode(ino as INODE_ID);

        // TODO: resize
        // TODO: what is fh?

        if let Some(mode) = mode {
            // TODO: other mode stuff
            inode.meta.permissions = mode.get_permissions();
        }
        if let Some(uid) = uid {
            inode.meta.user_id = uid;
        }
        if let Some(gid) = gid {
            inode.meta.group_id = gid;
        }
        if let Some(flags) = flags {
            inode.meta.flags = flags;
        }
        if let Some(_atime) = _atime {
            inode.meta.accessed_at = _atime.try_into().unwrap();
        }
        if let Some(_mtime) = _mtime {
            inode.meta.modified_at = _mtime.try_into().unwrap();
        }
        if let Some(_ctime) = _ctime {
            inode.meta.changed_at = _ctime;
        }

        self.journey_fs.write_inode(inode);
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

    fn fileattr_to_metadata(&self, attr: FileAttr) -> Metadata {
        Metadata {
            inode_type: match attr.kind {
                FileType::RegularFile => InodeType::File,
                FileType::Directory => InodeType::Directory,
                _ => panic!("Unsupported file type"),
            },
            created_at: attr.crtime,
            modified_at: attr.mtime,
            changed_at: attr.ctime,
            accessed_at: attr.atime,
            permissions: attr.perm,
            nlinks: attr.nlink,
            user_id: attr.uid,
            group_id: attr.gid,
            rdev: attr.rdev,
            flags: attr.flags,
        }
    }
}
