pub(crate) mod file_drive;

pub trait DeviceDriver {
    #[deprecated]
    fn get_size(&self) -> u64;
    fn get_sector_count(&self) -> u64;
    fn get_sector_size(&self) -> usize;
    fn read_sector(&self, index: u64) -> Vec<u8>;
    fn write_sector(&mut self, index: u64, data: &Vec<u8>);
}
