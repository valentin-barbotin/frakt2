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

impl FractalDescriptor {
    // TODO: Using a centralized function would be 
    // a better approach to call the generation of a Fractal
    // ISSUE: the payload can be different on each Fractal
    pub fn generate(&self) {
        match self {
            FractalDescriptor::Julia(_) => todo!(),
            FractalDescriptor::Mandelbrot(_) => todo!(),
            FractalDescriptor::IteratedSinZ(_) => todo!(),
            FractalDescriptor::NewtonRaphsonZ3(_) => todo!(),
            FractalDescriptor::NewtonRaphsonZ4(_) => todo!(),
        }
    }
}
