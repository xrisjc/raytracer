use crate::algorithm::*;
use crate::util::*;
use std::ops::{Add, Div, Mul, Neg, Sub};

#[derive(Debug, Copy, Clone)]
pub struct Tuple {
    pub elms: [f64; 4],
}

impl Tuple {
    pub fn new(x: f64, y: f64, z: f64, w: f64) -> Self {
        Tuple { elms: [x, y, z, w] }
    }

    pub fn point(x: f64, y: f64, z: f64) -> Self {
        Tuple::new(x, y, z, 1.0)
    }

    pub fn vector(x: f64, y: f64, z: f64) -> Self {
        Tuple::new(x, y, z, 0.0)
    }

    pub fn from_slice(t: &[f64]) -> Self {
        Tuple::new(t[0], t[1], t[2], t[3])
    }

    pub fn x(&self) -> f64 {
        self.elms[0]
    }

    pub fn y(&self) -> f64 {
        self.elms[1]
    }

    pub fn z(&self) -> f64 {
        self.elms[2]
    }

    pub fn w(&self) -> f64 {
        self.elms[3]
    }

    pub fn set_w(&mut self, w: f64) {
        self.elms[3] = w;
    }

    pub fn iter(&self) -> impl Iterator<Item = &f64> {
        self.elms.iter().take(3)
    }

    pub fn magnitude(self) -> f64 {
        dot_product(&self.elms, &self.elms).sqrt()
    }

    pub fn normalize(self) -> Tuple {
        self / self.magnitude()
    }

    pub fn dot(self, rhs: Self) -> f64 {
        dot_product(&self.elms, &rhs.elms)
    }

    pub fn cross(self, rhs: Self) -> Self {
        let elms = cross_product(&self.elms, &rhs.elms);
        Tuple { elms }
    }

    pub fn reflect(self, normal: Tuple) -> Tuple {
        self - normal * 2.0 * self.dot(normal)
    }
}

impl PartialEq for Tuple {
    fn eq(&self, other: &Self) -> bool {
        self.elms
            .iter()
            .zip(&other.elms)
            .map(|(a, b)| close_eq(*a, *b))
            .all(|p| p)
    }
}

impl Add for Tuple {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        let mut elms = [0.0; 4];
        for ((a, b), c) in self.elms.iter().zip(&other.elms).zip(&mut elms) {
            *c = *a + *b;
        }
        Tuple { elms }
    }
}

impl Sub for Tuple {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        let elms = vector_subtract(&self.elms, &other.elms);
        Tuple { elms }
    }
}

impl Neg for Tuple {
    type Output = Self;
    fn neg(self) -> Self {
        let mut elms = [0.0; 4];
        for (a, b) in self.elms.iter().zip(&mut elms) {
            *b = -*a;
        }
        Tuple { elms }
    }
}

impl Mul<f64> for Tuple {
    type Output = Self;
    fn mul(self, rhs: f64) -> Self {
        let mut elms = [0.0; 4];
        for (a, b) in self.elms.iter().zip(&mut elms) {
            *b = *a * rhs;
        }
        Tuple { elms }
    }
}

impl Div<f64> for Tuple {
    type Output = Self;
    fn div(self, rhs: f64) -> Self {
        let mut elms = [0.0; 4];
        for (a, b) in self.elms.iter().zip(&mut elms) {
            *b = *a / rhs;
        }
        Tuple { elms }
    }
}
