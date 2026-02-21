use borsh::{BorshDeserialize, BorshSerialize};

use crate::serializer::Serializer;

pub struct Borsh;

impl<T> Serializer<T> for Borsh
where
    T: BorshSerialize + BorshDeserialize,
{
    type Error = std::io::Error;

    fn to_bytes(data: &T) -> Result<Vec<u8>, Self::Error> {
        borsh::to_vec(data)
    }

    fn from_bytes(bytes: &[u8]) -> Result<T, Self::Error> {
        T::try_from_slice(bytes)
    }
}
