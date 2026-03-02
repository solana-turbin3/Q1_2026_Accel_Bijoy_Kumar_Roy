use {pinocchio::error::ProgramError, thiserror::Error};

#[derive(Clone, Debug, Eq, Error, PartialEq)]
pub enum PinocchioError {
    // 0
    /// Lamport balance below rent-exempt threshold.
    #[error("Fundraiser duration is active")]
    FundraiserNotEnded,
    #[error("Fundraiser target met")]
    TargetMet,
    #[error("Fundraiser target not met")]
    TargetNotMet,
}

impl From<PinocchioError> for ProgramError {
    fn from(e: PinocchioError) -> Self {
        ProgramError::Custom(e as u32)
    }
}

impl TryFrom<u32> for PinocchioError {
    type Error = ProgramError;
    fn try_from(error: u32) -> Result<Self, Self::Error> {
        match error {
            0 => Ok(PinocchioError::FundraiserNotEnded),
            1 => Ok(PinocchioError::TargetMet),
            _ => Err(ProgramError::InvalidArgument),
        }
    }
}
