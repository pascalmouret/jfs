const KILO_BYTE: u64 = 1024;
const MEGA_BYTE: u64 = KILO_BYTE * 1024;
const GIGA_BYTE: u64 = MEGA_BYTE * 1024;
const TERRA_BYTE: u64 = GIGA_BYTE * 1024;

pub fn pretty_size_from_bytes(bytes: u64) -> String {
    if bytes < KILO_BYTE {
        format!("{} B", bytes)
    } else if bytes < MEGA_BYTE {
        format!("{:.2} KB", bytes as f64 / KILO_BYTE as f64)
    } else if bytes < GIGA_BYTE {
        format!("{:.2} MB", bytes as f64 / MEGA_BYTE as f64)
    } else if bytes < TERRA_BYTE {
        format!("{:.2} GB", bytes as f64 / GIGA_BYTE as f64)
    } else {
        format!("{:.2} TB", bytes as f64 / TERRA_BYTE as f64)
    }
}
