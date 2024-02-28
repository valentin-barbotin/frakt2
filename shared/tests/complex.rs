#[cfg(test)]
mod tests {
    use complex::Complex;

    #[test]
    fn test_complex_operations() {
        let a = Complex::new(1.0, 2.0);
        let b = Complex::new(3.0, 4.0);

        // Test addition
        let add_result = a + b;
        assert_eq!(add_result.re, 4.0);
        assert_eq!(add_result.im, 6.0);

        // Test multiplication
        let mul_result = a * b;
        assert_eq!(mul_result.re, -5.0); // (1*3 - 2*4)
        assert_eq!(mul_result.im, 10.0); // (1*4 + 2*3)

        // Test magnitude
        let mag_result = a.sqrt_mag();
        assert_eq!(mag_result, 5.0); // sqrt(1^2 + 2^2)
    }
}

