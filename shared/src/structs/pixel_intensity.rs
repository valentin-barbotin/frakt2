use super::prelude::*;

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct PixelIntensity {
    pub zn: f32,
    pub count: f32,
}

impl PixelIntensity {
    pub fn new(zn: f32, count: f32) -> Self {
        Self { zn, count }
    }
}
