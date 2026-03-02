pub mod checker;
pub mod contribute;
pub mod initialize;
pub mod refund;

pub use checker::*;
pub use contribute::*;
pub use initialize::*;
use pinocchio::error::ProgramError;
pub use refund::*;

pub enum FundraiserInstrctions {
    Initialize = 0,
    Contribute = 1,
    Refund = 2,
    Checker = 3,
}

impl TryFrom<&u8> for FundraiserInstrctions {
    type Error = ProgramError;

    fn try_from(value: &u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(FundraiserInstrctions::Initialize),
            1 => Ok(FundraiserInstrctions::Contribute),
            2 => Ok(FundraiserInstrctions::Refund),
            3 => Ok(FundraiserInstrctions::Checker),
            _ => Err(ProgramError::InvalidInstructionData),
        }
    }
}
