use wincode::{SchemaRead, SchemaWrite, config::Configuration};

use crate::serializer::Serializer;

pub struct Wincode;

impl<T> Serializer<T> for Wincode
where
    T: SchemaWrite<Configuration, Src = T> + for<'a> SchemaRead<'a, Configuration, Dst = T>,
{
    type Error = wincode::Error;

    fn to_bytes(data: &T) -> Result<Vec<u8>, Self::Error> {
        wincode::serialize(data).map_err(Into::into)
    }

    fn from_bytes(bytes: &[u8]) -> Result<T, Self::Error> {
        wincode::deserialize(bytes).map_err(Into::into)
    }
}
