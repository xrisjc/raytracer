use crate::algorithm::dot_product;
use crate::tuple::Tuple;
use crate::util::*;
use std::ops::Mul;

#[derive(Copy, Clone, Debug)]
pub struct Matrix<const N: usize> {
    elements: [[f64; N]; N],
}

impl<const N: usize> Matrix<N> {
    fn new(elements: [[f64; N]; N]) -> Self {
        Self { elements }
    }

    pub fn identity() -> Self {
        Matrix::new(identity_elements())
    }

    fn element(&self, row: usize, column: usize) -> f64 {
        self.elements[row][column]
    }

    pub fn transpose(&self) -> Self {
        let mut elements = zero_elements();
        for r in 0..N {
            for c in 0..N {
                elements[c][r] = self.elements[r][c];
            }
        }
        Matrix::new(elements)
    }
}

impl Matrix<2> {
    fn determinant(&self) -> f64 {
        let a = self.elements[0][0];
        let b = self.elements[0][1];
        let c = self.elements[1][0];
        let d = self.elements[1][1];

        a * d - b * c
    }
}

impl Matrix<3> {
    fn submatrix(&self, row: usize, column: usize) -> Matrix<2> {
        let mut elements = zero_elements();
        let mut r_sub = 0;
        for r in 0..3 {
            let mut c_sub = 0;

            if r == row {
                continue;
            }

            for c in 0..3 {
                if c == column {
                    continue;
                }

                elements[r_sub][c_sub] = self.elements[r][c];

                c_sub += 1;
            }

            r_sub += 1;
        }

        Matrix::new(elements)
    }

    fn minor(&self, row: usize, column: usize) -> f64 {
        self.submatrix(row, column).determinant()
    }

    fn cofactor(&self, row: usize, column: usize) -> f64 {
        let mut c = self.minor(row, column);
        if (row + column) % 2 == 1 {
            c = -c;
        }
        c
    }

    fn determinant(&self) -> f64 {
        let mut d = 0.0;
        for c in 0..3 {
            d += self.elements[0][c] * self.cofactor(0, c);
        }
        d
    }
}

impl Matrix<4> {
    pub fn translation(x: f64, y: f64, z: f64) -> Self {
        let mut elements = identity_elements();
        elements[0][3] = x;
        elements[1][3] = y;
        elements[2][3] = z;
        Matrix::new(elements)
    }

    pub fn scaling(x: f64, y: f64, z: f64) -> Self {
        let mut elements = identity_elements();
        elements[0][0] = x;
        elements[1][1] = y;
        elements[2][2] = z;
        Matrix::new(elements)
    }

    pub fn rotation_x(r: f64) -> Self {
        let cos_r = r.cos();
        let sin_r = r.sin();
        let mut elements = identity_elements();
        elements[1][1] = cos_r;
        elements[1][2] = -sin_r;
        elements[2][1] = sin_r;
        elements[2][2] = cos_r;
        Matrix::new(elements)
    }

    pub fn rotation_y(r: f64) -> Self {
        let cos_r = r.cos();
        let sin_r = r.sin();
        let mut elements = identity_elements();
        elements[0][0] = cos_r;
        elements[0][2] = sin_r;
        elements[2][0] = -sin_r;
        elements[2][2] = cos_r;
        Matrix::new(elements)
    }

    pub fn rotation_z(r: f64) -> Self {
        let cos_r = r.cos();
        let sin_r = r.sin();
        let mut elements = identity_elements();
        elements[0][0] = cos_r;
        elements[0][1] = -sin_r;
        elements[1][0] = sin_r;
        elements[1][1] = cos_r;
        Matrix::new(elements)
    }

    pub fn shearing(x_y: f64, x_z: f64, y_x: f64, y_z: f64, z_x: f64, z_y: f64) -> Self {
        let mut elements = identity_elements();
        elements[0][1] = x_y;
        elements[0][2] = x_z;
        elements[1][0] = y_x;
        elements[1][2] = y_z;
        elements[2][0] = z_x;
        elements[2][1] = z_y;
        Matrix::new(elements)
    }

    pub fn view_transform(from: Tuple, to: Tuple, up: Tuple) -> Self {
        let forward = (to - from).normalize();
        let up = up.normalize();
        let left = forward.cross(up);
        // Computing true_up allows for the up vector to not be exactly an up vector.
        let true_up = left.cross(forward);
        let orientation = Matrix::new([
            [left.x(), left.y(), left.z(), 0.0],
            [true_up.x(), true_up.y(), true_up.z(), 0.0],
            [-forward.x(), -forward.y(), -forward.z(), 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ]);
        orientation * Matrix::translation(-from.x(), -from.y(), -from.z())
    }

    fn submatrix(&self, row: usize, column: usize) -> Matrix<3> {
        let mut elements = zero_elements();
        let mut r_sub = 0;
        for r in 0..4 {
            let mut c_sub = 0;

            if r == row {
                continue;
            }

            for c in 0..4 {
                if c == column {
                    continue;
                }

                elements[r_sub][c_sub] = self.elements[r][c];

                c_sub += 1;
            }

            r_sub += 1;
        }

        Matrix::new(elements)
    }

    fn minor(&self, row: usize, column: usize) -> f64 {
        self.submatrix(row, column).determinant()
    }

    fn cofactor(&self, row: usize, column: usize) -> f64 {
        let mut c = self.minor(row, column);
        if (row + column) % 2 == 1 {
            c = -c;
        }
        c
    }

    fn determinant(&self) -> f64 {
        let mut d = 0.0;
        for c in 0..4 {
            d += self.elements[0][c] * self.cofactor(0, c);
        }
        d
    }

    pub fn inverse(&self) -> Matrix<4> {
        let d = self.determinant();
        let mut elements = zero_elements();
        for r in 0..4 {
            for c in 0..4 {
                elements[c][r] = self.cofactor(r, c) / d;
            }
        }

        Matrix::new(elements)
    }
}

impl<const N: usize> PartialEq for Matrix<N> {
    fn eq(&self, other: &Self) -> bool {
        for r in 0..N {
            for c in 0..N {
                let lhs = self.elements[r][c];
                let rhs = other.elements[r][c];
                if !close_eq(lhs, rhs) {
                    return false;
                }
            }
        }
        return true;
    }
}

impl<const N: usize> Mul for Matrix<N> {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self {
        let mut p = zero_elements();
        for r in 0..N {
            for c in 0..N {
                for i in 0..N {
                    p[r][c] += self.elements[r][i] * rhs.elements[i][c];
                }
            }
        }
        Matrix::new(p)
    }
}

impl<const N: usize> Mul<Tuple> for Matrix<N> {
    type Output = Tuple;
    fn mul(self, rhs: Tuple) -> Tuple {
        let x = dot_product(&self.elements[0], &rhs.elms);
        let y = dot_product(&self.elements[1], &rhs.elms);
        let z = dot_product(&self.elements[2], &rhs.elms);
        let w = dot_product(&self.elements[3], &rhs.elms);

        Tuple::new(x, y, z, w)
    }
}

fn zero_elements<const R: usize, const C: usize>() -> [[f64; R]; C] {
    [[0.0; R]; C]
}

fn identity_elements<const N: usize>() -> [[f64; N]; N] {
    let mut elements = zero_elements();
    for i in 0..N {
        elements[i][i] = 1.0;
    }
    elements
}
