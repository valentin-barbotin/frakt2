use super::prelude::*;

#[derive(Serialize, Deserialize, Debug)]
pub enum FractalDescriptor {
    Julia(Julia),
    Mandelbrot(Mandelbrot),
    IteratedSinZ(IteratedSinZ),
    NewtonRaphsonZ3(NewtonRaphsonZ3),
    NewtonRaphsonZ4(NewtonRaphsonZ4),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Julia {
    pub c: Complex,
    pub divergence_threshold_square: f64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Mandelbrot {}

#[derive(Serialize, Deserialize, Debug)]
pub struct IteratedSinZ {
    pub c: Complex,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct NewtonRaphsonZ3 {
    pub c: Complex,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct NewtonRaphsonZ4 {
    pub c: Complex,
}
