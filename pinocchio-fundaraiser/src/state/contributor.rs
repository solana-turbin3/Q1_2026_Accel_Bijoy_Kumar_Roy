use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Zeroable, Pod, Copy, Clone)]
pub struct Contributor {
    pub amount: [u8; 8],
    pub bump: u8,
}

impl Contributor {
    pub const SIZE: usize = core::mem::size_of::<Contributor>();
}
