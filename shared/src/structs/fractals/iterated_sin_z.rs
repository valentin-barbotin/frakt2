use super::super::prelude::*;

#[derive(Serialize, Deserialize, Debug)]
pub struct IteratedSinZ {
    pub c: Complex,
}

impl IteratedSinZ {
    pub fn generate(&self, max: u16, x: f64, y: f64) -> (f64, f64) {
        let mut z = Complex::new(x, y);

        let mut i = 0;
        while i < max && z.sqrt_mag() < 50.0 {
            z = z.complex_sin() * self.c;
            i += 1;
        }

        return (z.sqrt_mag(), i as f64);
    }
}