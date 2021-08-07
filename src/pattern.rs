use crate::color::*;
use crate::matrix::*;
use crate::tuple::*;

pub trait PatternMap {
    fn color_at(&self, point: Tuple) -> Color;
}

pub struct Pattern {
    map: Box<dyn PatternMap>,
    transform_inverse: Matrix<4>,
}

impl Pattern {
    pub fn color_at_object(&self, object_point: Tuple) -> Color {
        let pattern_point = self.transform_inverse * object_point;
        self.map.color_at(pattern_point)
    }
}

pub struct PatternBuilder {
    map: Option<Box<dyn PatternMap>>,
    transform_inverse: Matrix<4>,
}

impl PatternBuilder {
    pub fn new() -> Self {
        PatternBuilder {
            map: None,
            transform_inverse: Matrix::identity(),
        }
    }

    pub fn build(self) -> Option<Pattern> {
        let map = self.map?;
        let transform_inverse = self.transform_inverse;
        Some(Pattern {
            map,
            transform_inverse,
        })
    }

    pub fn transform(mut self, transform: Matrix<4>) -> Self {
        self.transform_inverse = transform.inverse();
        self
    }

    pub fn stripes(mut self, a: Color, b: Color) -> Self {
        let map = Stripes { a, b };
        self.map = Some(Box::new(map));
        self
    }

    pub fn gradient(mut self, a: Color, b: Color) -> Self {
        let map = Gradient { a, b };
        self.map = Some(Box::new(map));
        self
    }

    pub fn rings(mut self, a: Color, b: Color) -> Self {
        let map = Rings { a, b };
        self.map = Some(Box::new(map));
        self
    }

    pub fn checkers(mut self, a: Color, b: Color) -> Self {
        let map = Checkers { a, b };
        self.map = Some(Box::new(map));
        self
    }
}

struct Stripes {
    a: Color,
    b: Color,
}

impl PatternMap for Stripes {
    fn color_at(&self, point: Tuple) -> Color {
        if point.x().floor() % 2.0 == 0.0 {
            self.a
        } else {
            self.b
        }
    }
}

struct Gradient {
    a: Color,
    b: Color,
}

impl PatternMap for Gradient {
    fn color_at(&self, point: Tuple) -> Color {
        self.a + (self.b - self.a) * (point.x() - point.x().floor())
    }
}

struct Rings {
    a: Color,
    b: Color,
}

impl PatternMap for Rings {
    fn color_at(&self, point: Tuple) -> Color {
        let x = point.x();
        let z = point.z();
        if (x * x + z * z).sqrt().floor() % 2.0 == 0.0 {
            self.a
        } else {
            self.b
        }
    }
}

struct Checkers {
    a: Color,
    b: Color,
}

impl PatternMap for Checkers {
    fn color_at(&self, point: Tuple) -> Color {
        if (point.x().floor() + point.z().floor()) % 2.0 == 0.0 {
            self.a
        } else {
            self.b
        }
    }
}
