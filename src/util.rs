pub const EPSILON: f64 = 1e-5;

pub fn close_eq(x: f64, y: f64) -> bool {
    (x - y).abs() < EPSILON
}

pub fn clamp(x: f64, min: f64, max: f64) -> f64 {
    if x < min {
        min
    } else if x > max {
        max
    } else {
        x
    }
}
