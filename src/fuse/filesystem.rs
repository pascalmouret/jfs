use core::panic;
use fuser::{FileAttr, FileType, Filesystem, ReplyAttr, ReplyEntry, Request, TimeOrNow};
use libc::c_int;
use std::ffi::OsStr;
use std::fs::File;
use std::time::{Duration, SystemTime};

use crate::driver::file_drive::FileDrive;
use crate::ops::meta::{GroupId, InodeType, Metadata, UserId};
use crate::ops::JourneyFS;
use crate::structure::inode::{Inode, InodeId};
use crate::util::error::Error;
use crate::util::mode::{ModeBits, ModeBitsHelper};

const TTL: Duration = Duration::new(100, 0);

struct FuseDriver {
    size: u64,
    journey_fs: Option<JourneyFS>,
    mount_path: String,
    sector_size: usize,
    block_size: usize,
}

impl Filesystem for FuseDriver {
    fn init(&mut self, _req: &Request<'_>, _config: &mut fuser::KernelConfig) -> Result<(), c_int> {
        if self.journey_fs.is_some() {
            panic!("init should only be called once")
        }

        let drive = match File::open(self.mount_path.as_str()) {
            Ok(file) => FileDrive::open(file, self.sector_size),
            Err(_) => FileDrive::new(self.mount_path.as_str(), self.size, self.sector_size),
        };

        // TODO: should error if an already existing file does not match our parameters

        match JourneyFS::new(drive, _req.uid(), _req.gid(), self.block_size) {
            Ok(fs) => {
                self.journey_fs = Some(fs);
                Ok(())
            }
            Err(error) => Err(error.error_num),
        }
    }

    fn getattr(&mut self, _req: &Request<'_>, ino: u64, reply: ReplyAttr) {
        // TODO: really need error handling
        let inode = self.get_fs_ref().get_inode(ino as InodeId);
        match inode {
            Ok(inode) => reply.attr(&TTL, &self.inode_to_fileattr(inode)),
            Err(error) => reply.error(error.error_num),
        }
    }

    // TODO: figure out what the unused arguments do
    fn setattr(
        &mut self,
        _req: &Request<'_>,
        ino: u64,
        mode: Option<ModeBits>,
        uid: Option<UserId>,
        gid: Option<GroupId>,
        size: Option<u64>,
        atime: Option<TimeOrNow>,
        mtime: Option<TimeOrNow>,
        ctime: Option<SystemTime>,
        _fh: Option<u64>,
        _crtime: Option<SystemTime>,
        _chgtime: Option<SystemTime>,
        _bkuptime: Option<SystemTime>,
        flags: Option<u32>,
        reply: ReplyAttr,
    ) {
        let result = self.get_fs_ref().get_inode(ino as InodeId);

        match result {
            Err(error) => reply.error(error.error_num),
            Ok(mut inode) => {
                // TODO: resize
                if let Some(_) = size {
                    reply.error(libc::EOPNOTSUPP);
                    return;
                }

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
                if let Some(atime) = atime {
                    inode.meta.accessed_at = FuseDriver::time_or_now_to_system_time(atime);
                }
                if let Some(mtime) = mtime {
                    inode.meta.modified_at = FuseDriver::time_or_now_to_system_time(mtime);
                }
                if let Some(ctime) = ctime {
                    inode.meta.changed_at = ctime;
                }

                match self.get_mut_fs_ref().write_inode(&mut inode) {
                    Ok(_) => reply.attr(&TTL, &FuseDriver::inode_to_fileattr(&self, inode)),
                    Err(error) => reply.error(error.error_num),
                }
            }
        }
    }

    fn mkdir(
        &mut self,
        req: &Request<'_>,
        parent: InodeId,
        name: &OsStr,
        mode: ModeBits,
        _umask: u32,
        reply: ReplyEntry,
    ) {
        // TODO: apply umask if necessary
        let permissions = mode.get_permissions();
        let result = self.get_mut_fs_ref().mkdir(
            parent,
            &name.try_into().unwrap(),
            req.uid(),
            req.gid(),
            permissions,
        );

        match result {
            Err(error) => reply.error(error.error_num),
            Ok(directory) => {
                reply.entry(
                    &Duration::new(100, 0),
                    &self.inode_to_fileattr(directory.inode),
                    0,
                );
            }
        }
    }
}

impl FuseDriver {
    fn new(
        mount_path: &str,
        size: u64,
        block_size: usize,
        sector_size: usize,
    ) -> Result<FuseDriver, Error> {
        return Ok(FuseDriver {
            mount_path: String::from(mount_path),
            size,
            journey_fs: None,
            block_size,
            sector_size,
        });
    }

    fn get_fs_ref(&self) -> &JourneyFS {
        self.journey_fs
            .as_ref()
            .expect("init should have been called")
    }

    fn get_mut_fs_ref(&mut self) -> &mut JourneyFS {
        self.journey_fs
            .as_mut()
            .expect("init should have been called")
    }

    fn inode_to_fileattr(&self, inode: Inode<Metadata>) -> FileAttr {
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
            // If the block size is not know something is seriously wrong and we
            // should panic
            blksize: self.get_fs_ref().get_block_size().unwrap() as u32,
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

    fn time_or_now_to_system_time(time_or_now: TimeOrNow) -> SystemTime {
        match time_or_now {
            TimeOrNow::SpecificTime(system_time) => system_time,
            TimeOrNow::Now => SystemTime::now(),
        }
    }
}
