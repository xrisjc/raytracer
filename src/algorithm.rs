pub fn vector_subtract<const N: usize>(a: &[f64], b: &[f64]) -> [f64; N] {
    let mut c = [0.0; N];
    for ((a, b), c) in a.iter().zip(b).zip(&mut c) {
        *c = *a - *b;
    }
    c
}

/// Dot product of two vectors.
pub fn dot_product(a: &[f64], b: &[f64]) -> f64 {
    let mut c = 0.0;
    for (a, b) in a.iter().zip(b) {
        c += *a * *b;
    }
    c
}

pub fn cross_product(a: &[f64], b: &[f64]) -> [f64; 4] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
        0.0,
    ]
}
