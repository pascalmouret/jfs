pub trait ByteSerializable {
    fn to_bytes(&self) -> Vec<u8>;
    fn from_bytes(bytes: &[u8]) -> Self;
}

pub trait KnownSize: ByteSerializable {
    fn size_on_disk() -> usize;
}
