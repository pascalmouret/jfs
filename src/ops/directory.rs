use std::ffi::OsString;
use std::mem::size_of;
use crate::consts::FILE_NAME_LENGTH;
use crate::ops::file::File;
use crate::ops::meta::{GroupId, InodeType, Metadata, UserId};
use crate::structure::inode::{Inode, INODE_ID};
use crate::structure::Structure;
use crate::util::serializable::ByteSerializable;

#[derive(Debug, PartialEq)]
struct Entry {
    // TODO: use something better for this, or at least find a safe way to decode it
    name: OsString,
    id: INODE_ID,
}

type EntryList = Vec<Entry>;

impl ByteSerializable for EntryList {
    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::<u8>::new();
        for entry in self {
            bytes.extend_from_slice(&entry.id.to_le_bytes());
            let name_bytes = entry.name.as_encoded_bytes();
            bytes.extend_from_slice(&(name_bytes.len() as u8).to_le_bytes());
            bytes.extend_from_slice(name_bytes);
        }
        bytes
    }

    fn from_bytes(bytes: &[u8]) -> Self {
        let mut entries = Vec::<Entry>::new();
        let mut data = bytes;
        unsafe {
            while data.len() > 0 {
                let (id_bytes, remainder) = data.split_at(size_of::<INODE_ID>());
                let (name_length_bytes, remainder) = remainder.split_at(size_of::<u8>());
                let (name_bytes, remainder) = remainder.split_at(name_length_bytes[0] as usize);

                let id = INODE_ID::from_le_bytes(id_bytes.try_into().unwrap());
                let name = OsString::from_encoded_bytes_unchecked(name_bytes.to_vec());
                entries.push(Entry { name, id });

                data = remainder;
            }
        }
        entries
    }
}

pub struct Directory {
    pub inode: Inode<Metadata>,
}

impl Directory {
    pub fn new(structure: &mut Structure<Metadata>, user_id: UserId, group_id: GroupId, permissions: u16) -> Directory {
        let meta = Metadata::new(InodeType::Directory, user_id, group_id, permissions, 2, 0);
        let inode = structure.create_inode(meta);
        Directory {
            inode,
        }
    }

    pub fn from_inode(inode: Inode<Metadata>) -> Directory {
        Directory {
            inode,
        }
    }

    pub fn get_entries(&self, structure: &Structure<Metadata>) -> EntryList {
        let data = self.inode.get_data(structure);
        EntryList::from_bytes(&data)
    }

    fn add_entry(&mut self, structure: &mut Structure<Metadata>, name: &OsString, id: INODE_ID) {
        if name.len() > FILE_NAME_LENGTH {
            panic!("Name too long");
        }

        let mut entries = self.get_entries(structure);
        entries.push(Entry { name: name.clone(), id });
        self.inode.set_data(structure, entries.to_bytes());
    }

    pub fn add_directory(
        &mut self, structure: &mut Structure<Metadata>,
        name: &OsString,
        user_id: UserId,
        group_id: GroupId,
        permissions: u16,
    ) -> Directory {
        let directory = Directory::new(structure, user_id, group_id, permissions);
        self.add_entry(structure, name, directory.inode.id.unwrap());
        directory
    }

    pub fn add_file(
        &mut self,
        structure: &mut Structure<Metadata>,
        name: &OsString,
        user_id: UserId,
        group_id: GroupId,
        permissions: u16,
    ) -> File {
        let file = File::new(structure, user_id, group_id, permissions);
        self.add_entry(structure, name, file.inode.id.unwrap());
        file
    }
}

#[cfg(test)]
mod tests {
    use crate::driver::file_drive::FileDrive;
    use crate::io::IO;
    use super::*;

    #[test]
    fn test_entry_list_to_bytes() {
        let entries = vec![
            Entry { name: OsString::from("file1"), id: 1 },
            Entry { name: OsString::from("file2"), id: 2 },
        ];
        let bytes = entries.to_bytes();
        let expected = vec![
            1, 0, 0, 0, 0, 0, 0, 0, 5, 102, 105, 108, 101, 49, 2, 0, 0, 0, 0, 0, 0, 0, 5, 102, 105, 108, 101, 50,
        ];
        assert_eq!(bytes, expected);
    }

    #[test]
    fn test_entry_list_from_bytes() {
        let entries = vec![
            Entry { name: OsString::from("file1"), id: 1 },
            Entry { name: OsString::from("file2"), id: 2 },
        ];
        let bytes = entries.to_bytes();
        assert_eq!(entries, EntryList::from_bytes(&bytes));
    }

    #[test]
    fn test_directory_new() {
        let drive = FileDrive::new("./test-images/test_directory_new.img", 2048 * 1024 * 5, 512);
        let io = IO::new(drive, 512);
        let mut structure = Structure::<Metadata>::new(io, 512);
        let mut directory = Directory::new(&mut structure, 0, 0,0o755);
        let entries = directory.get_entries(&structure);
        assert_eq!(entries.len(), 0);
    }

    #[test]
    fn test_directory_add_entry() {
        let drive = FileDrive::new("./test-images/test_directory_add_entry.img", 2048 * 1024 * 5, 512);
        let io = IO::new(drive, 1024);
        let mut structure = Structure::<Metadata>::new(io, 1024);
        let mut directory = Directory::new(&mut structure, 0, 0, 0o755);
        directory.add_entry(&mut structure, &OsString::from("file1"), 1);
        let entries = directory.get_entries(&structure);
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].name, "file1");
        assert_eq!(entries[0].id, 1);
    }
}
