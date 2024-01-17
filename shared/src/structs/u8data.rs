use super::prelude::*;

#[derive(Serialize, Deserialize, Debug, Clone, Copy, Default)]
pub struct U8Data {
    pub offset: u32,
    pub count: u32,
}

impl U8Data {
    pub fn new(offset: u32, count: u32) -> Self {
        Self { offset, count }
    }
}
