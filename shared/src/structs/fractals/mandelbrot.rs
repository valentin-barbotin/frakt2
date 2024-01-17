use super::super::prelude::*;

#[derive(Serialize, Deserialize, Debug, Clone, Copy, Default)]
pub struct Mandelbrot {}

impl Mandelbrot {
    pub fn generate(max: u16, x: f64, y: f64) -> (f64, f64) {
        let mut z = Complex::new(0.0, 0.0);
        let c = Complex::new(x, y);

        let mut i = 0;
        while i < max && z.sqrt_mag() < 4.0 {
            z = z * z + c;
            i += 1;
        }

        return (z.sqrt_mag(), i as f64);
    }
}
