use std::time::{Duration, SystemTime};
use crate::util::serializable::{ByteSerializable, KnownSize};

pub enum InodeType {
    File,
    Directory,
}

pub type UserId = u32;
pub type GroupId = u32;

pub struct Metadata {
    pub inode_type: InodeType,
    pub created_at: SystemTime,
    pub modified_at: SystemTime,
    pub changed_at: SystemTime,
    pub accessed_at: SystemTime,
    pub permissions: u16,
    pub nlinks: u32, // what is this?
    pub user_id: UserId,
    pub group_id: GroupId,
    pub rdev: u32, // what is this?
    pub flags: u32,
}

impl Metadata {
    pub fn new(
        inode_type: InodeType,
        user_id: UserId,
        group_id: GroupId,
        permissions: u16,
        nlinks: u32,
        flags: u32,
    ) -> Metadata {
        let now = SystemTime::now();

        Metadata {
            inode_type,
            created_at: now,
            modified_at: now,
            changed_at: now,
            accessed_at: now,
            permissions,
            nlinks,
            user_id,
            group_id,
            rdev: 0,
            flags,
        }
    }
}

impl ByteSerializable for SystemTime {
    fn to_bytes(&self) -> Vec<u8> {
        let since_unix = self.duration_since(SystemTime::UNIX_EPOCH).unwrap();
        let mut result = Vec::<u8>::new();
        result.extend_from_slice(&since_unix.as_secs().to_le_bytes());
        result.extend_from_slice(&since_unix.subsec_nanos().to_le_bytes());
        result
    }

    fn from_bytes(bytes: &[u8]) -> Self {
        let (seconds, sub_nanos) = bytes.split_at(8);

        SystemTime::UNIX_EPOCH + Duration::new(
            u64::from_le_bytes(seconds.try_into().unwrap()),
            u32::from_le_bytes(sub_nanos.try_into().unwrap())
        )
    }
}

impl ByteSerializable for InodeType {
    fn to_bytes(&self) -> Vec<u8> {
        match self {
            InodeType::File => vec![0],
            InodeType::Directory => vec![1],
        }
    }

    fn from_bytes(bytes: &[u8]) -> Self {
        match bytes[0] {
            0 => InodeType::File,
            1 => InodeType::Directory,
            _ => panic!("Invalid inode type"),
        }
    }
}

impl KnownSize for Metadata {
    fn size_on_disk() -> usize {
        // TODO: this seems stupid
        71
    }
}

impl ByteSerializable for Metadata {
    fn to_bytes(&self) -> Vec<u8> {
        let mut result = Vec::<u8>::new();
        result.extend_from_slice(&self.inode_type.to_bytes());
        result.extend_from_slice(&self.created_at.to_bytes());
        result.extend_from_slice(&self.modified_at.to_bytes());
        result.extend_from_slice(&self.accessed_at.to_bytes());
        result.extend_from_slice(&self.changed_at.to_bytes());
        result.extend_from_slice(&self.permissions.to_le_bytes());
        result.extend_from_slice(&self.nlinks.to_le_bytes());
        result.extend_from_slice(&self.user_id.to_le_bytes());
        result.extend_from_slice(&self.group_id.to_le_bytes());
        result.extend_from_slice(&self.rdev.to_le_bytes());
        result.extend_from_slice(&self.flags.to_le_bytes());
        result
    }

    fn from_bytes(bytes: &[u8]) -> Self {
        let inode_type = InodeType::from_bytes(&bytes[0..1]);
        let created_at = SystemTime::from_bytes(&bytes[1..17]);
        let modified_at = SystemTime::from_bytes(&bytes[17..33]);
        let accessed_at = SystemTime::from_bytes(&bytes[33..49]);
        let changed_at = SystemTime::from_bytes(&bytes[49..65]);
        let permission = u16::from_le_bytes([bytes[65], bytes[66]]);
        let nlinks = u32::from_le_bytes([bytes[67], bytes[68], bytes[69], bytes[70]]);
        let user_id = u32::from_le_bytes([bytes[71], bytes[72], bytes[73], bytes[74]]);
        let group_id = u32::from_le_bytes([bytes[75], bytes[76], bytes[77], bytes[78]]);
        let rdev = u32::from_le_bytes([bytes[79], bytes[80], bytes[81], bytes[82]]);
        let flags = u32::from_le_bytes([bytes[83], bytes[84], bytes[85], bytes[86]]);

        Metadata {
            inode_type,
            created_at,
            modified_at,
            accessed_at,
            changed_at,
            permissions: permission,
            nlinks,
            user_id,
            group_id,
            rdev,
            flags,
        }
    }
}
