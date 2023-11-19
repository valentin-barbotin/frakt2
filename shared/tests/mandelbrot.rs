#[cfg(test)]
mod tests {
    use shared::structs::prelude::mandelbrot::Mandelbrot;

    #[test]
    fn test_mandelbrot_generate() {
        let max_iterations = 1000;

        // Point inside the Mandelbrot set
        {
            let (_zn, count) = Mandelbrot::generate(max_iterations, 0.0, 0.0);
            assert!((count as u16) == max_iterations); // Should reach max iterations
        }

        // Point outside the Mandelbrot set
        {
            let (_zn, count) = Mandelbrot::generate(max_iterations, 2.0, 2.0);
            assert!((count as u16) < max_iterations); // Should not reach max iterations
        }
    }
}
