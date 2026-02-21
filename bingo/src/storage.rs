use std::marker::PhantomData;

use crate::serializer::Serializer;

pub struct Storage<T, S> {
    data: Option<Vec<u8>>,
    _serializer: S,
    _type_holder: PhantomData<T>,
}

#[derive(Debug)]
pub enum ConversionError<E1, E2> {
    ReadOldFormat(E1),
    WriteNewFormat(E2),
}

impl<T, S> Storage<T, S>
where
    S: Serializer<T>,
{
    pub fn new(serializer: S) -> Self {
        Self {
            data: None,
            _serializer: serializer,
            _type_holder: PhantomData,
        }
    }

    pub fn save(&mut self, data: &T) -> Result<(), S::Error> {
        let bytes = S::to_bytes(data)?;
        self.data = Some(bytes);
        Ok(())
    }

    pub fn load(&self) -> Result<T, S::Error> {
        let bytes = self.data.as_ref().expect("No data stored!");
        S::from_bytes(bytes)
    }

    pub fn convert<S2>(
        &mut self,
        serializer: S2,
    ) -> Result<Storage<T, S2>, ConversionError<S::Error, S2::Error>>
    where
        S2: Serializer<T>,
    {
        let mut out = Storage::new(serializer);

        if let Some(bytes) = &self.data {
            let data = S::from_bytes(bytes).map_err(ConversionError::ReadOldFormat)?;
            out.save(&data).map_err(ConversionError::WriteNewFormat)?;
        }

        Ok(out)
    }
}
