use super::prelude::*;

#[derive(Serialize, Deserialize, Debug)]
pub struct Complex {
    re: f64,
    im: f64,
}

impl Complex {
    pub fn sqrt_mag(self) -> f64 {
        self.re * self.re + self.im * self.im
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
            im: self.re * rhs.im + self.im * rhs.re ,
        }
    }
}
