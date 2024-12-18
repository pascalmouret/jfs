use crate::driver::DeviceDriver;
use std::fs::File;
use std::os::unix::fs::FileExt;

pub struct FileDrive {
    file: File,
    pub bytes: u64,
    pub sector_size: usize,
}

impl FileDrive {
    pub fn new(name: &str, bytes: u64, sector_size: usize) -> FileDrive {
        let file = File::create_new(name).unwrap();
        file.set_len(bytes).unwrap();
        FileDrive {
            file,
            bytes,
            sector_size,
        }
    }

    pub fn open(file: File, sector_size: usize) -> FileDrive {
        let bytes = file.metadata().unwrap().len();
        FileDrive {
            file,
            bytes,
            sector_size,
        }
    }

    pub fn open_path(path: &str, sector_size: usize) -> FileDrive {
        let file = File::open(path).unwrap();
        let bytes = file.metadata().unwrap().len();
        FileDrive {
            file,
            bytes,
            sector_size,
        }
    }
}

impl DeviceDriver for FileDrive {
    fn get_sector_count(&self) -> u64 {
        self.bytes / self.sector_size as u64
    }

    fn get_sector_size(&self) -> usize {
        self.sector_size
    }

    fn read_sector(&self, index: u64) -> Vec<u8> {
        let mut buffer = vec![0; self.sector_size];
        self.file
            .read_at(&mut buffer, index * self.sector_size as u64)
            .unwrap();
        buffer
    }

    fn write_sector(&mut self, index: u64, sector: &Vec<u8>) {
        if sector.len() != self.sector_size {
            panic!(
                "Sector size mismatch - expected {}, got {}",
                self.sector_size,
                sector.len()
            );
        }
        self.file
            .write_at(&sector, index * self.sector_size as u64)
            .unwrap();
    }
}

#[cfg(test)]
mod tests {
    use crate::driver::file_drive::FileDrive;
    use crate::driver::DeviceDriver;

    #[test]
    fn test_hard_drive() {
        let mut drive = FileDrive::new("./test-images/test_drive.img", 1024 * 512, 512);

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
}
