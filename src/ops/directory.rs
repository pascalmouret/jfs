use std::mem::size_of;
use crate::consts::FILE_NAME_LENGTH;
use crate::ops::meta::{InodeType, Metadata};
use crate::structure::inode::{ByteSerializable, Inode, INODE_ID};
use crate::structure::Structure;

#[derive(Debug, PartialEq)]
struct Entry {
    name: String,
    id: INODE_ID,
}

type EntryList = Vec<Entry>;

impl ByteSerializable for EntryList {
    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::<u8>::new();
        for entry in self {
            bytes.extend_from_slice(&entry.id.to_le_bytes());
            bytes.extend_from_slice(&(entry.name.len() as u8).to_le_bytes());
            bytes.extend_from_slice(entry.name.as_bytes());
        }
        bytes
    }

    fn from_bytes(bytes: &[u8]) -> Self {
        let mut entries = Vec::<Entry>::new();
        let mut data = bytes;
        while data.len() > 0 {
            let (id_bytes, remainder) = data.split_at(size_of::<INODE_ID>());
            let (name_length_bytes, remainder) = remainder.split_at(size_of::<u8>());
            let (name_bytes, remainder) = remainder.split_at(name_length_bytes[0] as usize);

            let id = INODE_ID::from_le_bytes(id_bytes.try_into().unwrap());
            let name = String::from_utf8(name_bytes.to_vec()).unwrap();
            entries.push(Entry { name, id });

            data = remainder;
        }
        entries
    }
}

struct Directory {
    inode: Inode<Metadata>,
}

impl Directory {
    pub fn new(structure: &mut Structure<Metadata>) -> Directory {
        let meta = Metadata { inode_type: InodeType::Directory };
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

    pub fn add_entry(&mut self, structure: &mut Structure<Metadata>, name: &String, id: INODE_ID) {
        if name.len() > FILE_NAME_LENGTH {
            panic!("Name too long");
        }

        let mut entries = self.get_entries(structure);
        entries.push(Entry { name: name.clone(), id });
        self.inode.set_data(structure, entries.to_bytes());
    }
}

#[cfg(test)]
mod tests {
    use crate::driver::file_drive::FileDrive;
    use super::*;

    #[test]
    fn test_entry_list_to_bytes() {
        let entries = vec![
            Entry { name: "file1".to_string(), id: 1 },
            Entry { name: "file2".to_string(), id: 2 },
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
            Entry { name: "file1".to_string(), id: 1 },
            Entry { name: "file2".to_string(), id: 2 },
        ];
        let bytes = entries.to_bytes();
        assert_eq!(entries, EntryList::from_bytes(&bytes));
    }

    #[test]
    fn test_directory_new() {
        let drive = FileDrive::new("./test-images/test_directory_new.img", 2048 * 1024 * 5, 512);
        let mut structure = Structure::<Metadata>::new(drive, 512);
        let mut directory = Directory::new(&mut structure);
        let entries = directory.get_entries(&structure);
        assert_eq!(entries.len(), 0);
    }

    #[test]
    fn test_directory_add_entry() {
        let drive = FileDrive::new("./test-images/test_directory_add_entry.img", 2048 * 1024 * 5, 512);
        let mut structure = Structure::<Metadata>::new(drive, 1024);
        let mut directory = Directory::new(&mut structure);
        directory.add_entry(&mut structure, &"file1".to_string(), 1);
        let entries = directory.get_entries(&structure);
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].name, "file1");
        assert_eq!(entries[0].id, 1);
    }
}
