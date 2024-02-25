use super::prelude::*;

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct FragmentResult {
    pub id: U8Data,
    pub resolution: Resolution,
    pub range: Range,
    pub pixels: PixelData,
}

impl FragmentResult {
    pub fn new(id: U8Data, resolution: Resolution, range: Range, pixels: PixelData) -> Self {
        FragmentResult {
            id,
            resolution,
            range,
            pixels,
        }
    }
}
