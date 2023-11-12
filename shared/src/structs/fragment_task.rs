use super::prelude::*;
use image::{ImageBuffer, Rgb};
use log::{debug, error, info, trace, warn, LevelFilter};
use std::{
    io::{Read, Write},
};

#[derive(Serialize, Deserialize, Debug)]
pub struct FragmentTask {
    id: U8Data,
    fractal: FractalDescriptor,
    max_iteration: u16,
    resolution: Resolution,
    range: Range,
}

impl FragmentTask {
    pub fn new(
        id: U8Data,
        fractal: FractalDescriptor,
        max_iteration: u16,
        resolution: Resolution,
        range: Range,
    ) -> Self {
        FragmentTask {
            id,
            fractal,
            max_iteration,
            resolution,
            range,
        }
    }

    pub fn run(&self) -> (FragmentResult, Vec<u8>) {
        // u16 to u32
        let width = self.resolution.nx as u32;
        let height = self.resolution.ny as u32;

        let mut image_buffer: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::new(width, height);

        let pixel_data = PixelData::new(
            self.id.count,
            width * height,
        );

        let result = FragmentResult::new(self.id, self.resolution, self.range, pixel_data);

        // exemple
        let mut data = Vec::new();
        for (_x, _y, _pixel) in image_buffer.enumerate_pixels_mut() {
            let zn = 5.0f32; // example
            let count = 50.0f32; // example

            let pixel_intensity = PixelIntensity::new(zn, count / self.max_iteration as f32);

            if let Err(_) = data.write_all(&pixel_intensity.zn.to_be_bytes()) {
                error!("Error: Failed to write pixel intensity to data");
            }

            if let Err(_) = data.write_all(&pixel_intensity.count.to_be_bytes()) {
                error!("Error: Failed to write pixel intensity to data");
            }
        }

        (result, data)
    }
}
