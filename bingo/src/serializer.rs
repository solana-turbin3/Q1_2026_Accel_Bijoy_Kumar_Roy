pub trait Serializer<T> {
    type Error;

    fn to_bytes(data: &T) -> Result<Vec<u8>, Self::Error>;
    fn from_bytes(bytes: &[u8]) -> Result<T, Self::Error>;
}
