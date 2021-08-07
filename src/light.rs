use crate::color::*;
use crate::tuple::*;

pub struct Light {
    pub position: Tuple,
    pub intensity: Color,
}

impl Light {
    pub fn new(position: Tuple, intensity: Color) -> Self {
        Light {
            position,
            intensity,
        }
    }
}
