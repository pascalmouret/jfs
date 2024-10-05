pub type ModeBits = u32;

const PERMISSIONS_MASK: ModeBits = 0o777;
const IS_DIR_MASK: ModeBits = 0o40000;
const IS_FILE_MASK: ModeBits = 0o100000;

pub trait ModeBitsHelper {
    fn get_permissions(&self) -> u16;
    fn is_directory(&self) -> bool;
    fn is_file(&self) -> bool;
}

impl ModeBitsHelper for ModeBits {
    fn get_permissions(&self) -> u16 {
        (self & PERMISSIONS_MASK) as u16
    }

    fn is_directory(&self) -> bool {
        (self & IS_DIR_MASK) != 0
    }

    fn is_file(&self) -> bool {
        (self & IS_FILE_MASK) != 0
    }
}
