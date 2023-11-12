use super::prelude::*;

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct PixelData {
    pub offset: u32,
    pub count: u32,
}

impl PixelData {
    pub fn new(offset: u32, count: u32) -> Self {
        Self { offset, count }
    }
}
