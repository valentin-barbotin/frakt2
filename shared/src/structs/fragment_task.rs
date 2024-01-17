use super::prelude::{mandelbrot::Mandelbrot, *, iterated_sin_z::IteratedSinZ, julia::Julia};
use image::{ImageBuffer, Rgb};
use log::{debug, error, info, trace, warn, LevelFilter};
use std::{io::{Read, Write}, str::FromStr};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct FragmentTask {
    pub id: U8Data,
    pub fractal: FractalDescriptor,
    pub max_iteration: u16,
    pub resolution: Resolution,
    pub range: Range,
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
        let Range { min, max } = self.range;

        let mut image_buffer: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::new(width, height);

        let pixel_data = PixelData::new(self.id.count, width * height);

        let result = FragmentResult::new(self.id, self.resolution, self.range, pixel_data);

        // exemple
        let mut data = Vec::new();

        for (x, y, _pixel) in image_buffer.enumerate_pixels_mut() {
            // Mapping des coordonnées dans la Range de la Resolution
            let mapped_x = min.x + (x as f64 / self.resolution.nx as f64) * (max.x - min.x);
            let mapped_y = min.y + (y as f64 / self.resolution.ny as f64) * (max.y - min.y);

            let zn: f64;
            let count: f64;

            match &self.fractal {
                FractalDescriptor::Julia(julia) => {
                    (zn, count) = julia.generate(self.max_iteration, mapped_x, mapped_y);
                },
                FractalDescriptor::Mandelbrot(_) => {
                    (zn, count) = Mandelbrot::generate(self.max_iteration, mapped_x, mapped_y);
                }
                FractalDescriptor::IteratedSinZ(iterated_sin_z) => {
                    (zn, count) = iterated_sin_z.generate(self.max_iteration, mapped_x, mapped_y);
                },
                FractalDescriptor::NewtonRaphsonZ3(_) => todo!(),
                FractalDescriptor::NewtonRaphsonZ4(_) => todo!(),
            }

            let pixel_intensity =
                PixelIntensity::new(zn as f32, (count as f32) / self.max_iteration as f32);

            if data.write_all(&pixel_intensity.zn.to_be_bytes()).is_err() {
                error!("Error: Failed to write pixel intensity to data");
            }

            if data.write_all(&pixel_intensity.count.to_be_bytes()).is_err() {
                error!("Error: Failed to write pixel intensity to data");
            }
        }

        (result, data)
    }
}