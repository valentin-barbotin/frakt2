use super::prelude::{
    iterated_sin_z::IteratedSinZ, julia::Julia, mandelbrot::Mandelbrot,
    newton_raphson_z3::NewtonRaphsonZ3, newton_raphson_z4::NewtonRaphsonZ4, *,
};

#[derive(Serialize, Deserialize, Debug)]
pub enum FractalDescriptor {
    Julia(Julia),
    Mandelbrot(Mandelbrot),
    IteratedSinZ(IteratedSinZ),
    NewtonRaphsonZ3(NewtonRaphsonZ3),
    NewtonRaphsonZ4(NewtonRaphsonZ4),
}
