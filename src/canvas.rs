use crate::color::Color;
use crate::util::*;
use std::error::Error;
use std::fmt::Write;

pub struct Canvas {
    pub width: usize,
    pub height: usize,
    pixels: Vec<Color>,
}

impl Canvas {
    pub fn new(width: usize, height: usize) -> Canvas {
        let length = width * height;
        let mut pixels = Vec::with_capacity(length);
        pixels.resize(length, Color::new(0.0, 0.0, 0.0));
        Canvas {
            width,
            height,
            pixels,
        }
    }

    pub fn write_pixel(&mut self, x: usize, y: usize, color: Color) {
        self.pixels[pixel_index(x, y, self.width)] = color;
    }

    pub fn pixel_at(&self, x: usize, y: usize) -> Color {
        self.pixels[pixel_index(x, y, self.width)]
    }

    pub fn to_ppm(&self) -> Result<String, Box<dyn Error>> {
        let mut ppm = String::new();
        write!(ppm, "P3\n{} {}\n255\n", self.width, self.height)?;
        for color in self.pixels.iter() {
            let red = clamp(color.red, 0.0, 1.0);
            let green = clamp(color.green, 0.0, 1.0);
            let blue = clamp(color.blue, 0.0, 1.0);
            let red = (255.0 * red) as u8;
            let green = (255.0 * green) as u8;
            let blue = (255.0 * blue) as u8;
            write!(ppm, "{} {} {}\n", red, green, blue)?;
        }
        write!(ppm, "\n")?;
        Ok(ppm)
    }
}

fn pixel_index(x: usize, y: usize, width: usize) -> usize {
    y * width + x
}
