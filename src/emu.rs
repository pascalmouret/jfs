use std::fs::File;
use std::os::unix::fs::FileExt;

pub struct HardDrive {
    file: File,
    pub bytes: u64,
    pub sector_size: usize,
}

impl HardDrive {
    pub fn new(name: &str, bytes: u64, sector_size: usize) -> HardDrive {
        let file = File::create_new(name).unwrap();
        file.set_len(bytes).unwrap();
        HardDrive { file, bytes, sector_size }
    }

    pub fn open(name: &str, sector_size: usize) -> HardDrive {
        let file = File::open(name).unwrap();
        let bytes = file.metadata().unwrap().len();
        HardDrive { file, bytes, sector_size }
    }

    pub fn read_sector(&self, index: u64) -> Vec<u8> {
        let mut buffer = vec![0; self.sector_size];
        self.file.read_at(&mut buffer, index * self.sector_size as u64).unwrap();
        buffer
    }

    pub fn write_sector(&self, index: u64, sector: &Vec<u8>) {
        if sector.len() != self.sector_size {
            panic!("Sector size mismatch - expected {}, got {}", self.sector_size, sector.len());
        }
        self.file.write_at(&sector, index * self.sector_size as u64).unwrap();
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    #[test]
    fn test_hard_drive() {
        {
            let drive = super::HardDrive::new("test_drive.img", 1024 * 512, 512);

            let sector0 = vec![0x42; 512];
            let sector1 = vec![0x1; 512];
            let sector512 = vec![0x8; 512];
            let sector1023 = vec![0x52; 512];

            drive.write_sector(0, &sector0);
            drive.write_sector(1, &sector1);
            drive.write_sector(512, &sector512);
            drive.write_sector(1023, &sector1023);

            assert_eq!(drive.read_sector(0), sector0);
            assert_eq!(drive.read_sector(1), sector1);
            assert_eq!(drive.read_sector(512), sector512);
            assert_eq!(drive.read_sector(1023), sector1023);
            assert_eq!(drive.read_sector(2), vec![0; 512]);
            assert_eq!(drive.read_sector(511), vec![0; 512]);

            let mut buffer = vec![0x42, 0x43, 0x44, 0x45, 0x46, 0x47, 0x48, 0x49];
            buffer.append(&mut vec![0; 504]);
            drive.write_sector(0, &buffer);

            assert_eq!(drive.read_sector(0), buffer);
        }

        fs::remove_file("test_drive.img").unwrap();
    }
}
