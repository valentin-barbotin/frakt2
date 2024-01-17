use super::prelude::*;

#[derive(Serialize, Deserialize, Debug, Clone, Copy, Default)]
pub struct Resolution {
    pub nx: u16,
    pub ny: u16,
}

impl Resolution {
    pub fn new(nx: u16, ny: u16) -> Self {
        Self { nx, ny }
    }
}
