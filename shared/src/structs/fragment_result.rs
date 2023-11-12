use super::prelude::*;

#[derive(Serialize, Deserialize, Debug)]
pub struct FragmentResult {
    id: U8Data,
    resolution: Resolution,
    range: Range,
    pixels: PixelData,
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
