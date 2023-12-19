use super::prelude::*;

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub enum FractalDescriptor {
    Julia(Julia),
    Mandelbrot(Mandelbrot),
    IteratedSinZ(IteratedSinZ),
    NewtonRaphsonZ3(NewtonRaphsonZ3),
    NewtonRaphsonZ4(NewtonRaphsonZ4),
}
