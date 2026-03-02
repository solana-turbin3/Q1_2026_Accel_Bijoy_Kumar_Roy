use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Zeroable, Pod, Copy, Clone)]
pub struct Fundraiser {
    pub maker: [u8; 32],
    pub mint_to_raise: [u8; 32],
    pub amount_to_raise: [u8; 8],
    pub current_amount: [u8; 8],
    pub time_started: [i8; 8],
    pub duration: u8,
    pub bump: u8,
}

impl Fundraiser {
    pub const SIZE: usize = core::mem::size_of::<Fundraiser>();
}
