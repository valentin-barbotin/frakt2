use super::prelude::*;

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct Complex {
    pub re: f64,
    pub im: f64,
}

impl Complex {
    pub fn new(re: f64, im: f64) -> Self {
        Self { re, im }
    }

    pub fn sqrt_mag(self) -> f64 {
        self.re * self.re + self.im * self.im
    }

    pub fn complex_sin(self) -> Complex {
        Complex {
            re: self.re.sin() * self.re.cosh(),
            im: self.im.cos() * self.im.sinh(),
        }
    }
}

impl std::ops::Add for Complex {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        Complex {
            re: self.re + rhs.re,
            im: self.im + rhs.im,
        }
    }
}

impl std::ops::Mul for Complex {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self {
        Complex {
            re: self.re * rhs.re - self.im * rhs.im,
            im: self.re * rhs.im + self.im * rhs.re,
        }
    }
}
