use serde::{Deserialize, Serialize};

use crate::serializer::Serializer;

pub struct Json;

impl<T> Serializer<T> for Json
where
    T: Serialize + for<'a> Deserialize<'a>,
{
    type Error = serde_json::Error;

    fn to_bytes(data: &T) -> Result<Vec<u8>, Self::Error> {
        serde_json::to_vec(data)
    }

    fn from_bytes(bytes: &[u8]) -> Result<T, Self::Error> {
        serde_json::from_slice(bytes)
    }
}
