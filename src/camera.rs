use crate::canvas::*;
use crate::matrix::*;
use crate::ray::*;
use crate::tuple::*;
use crate::world::*;

pub struct Camera {
    hsize: usize,
    vsize: usize,
    field_of_view: f64,
    transform: Matrix<4>,
    transform_inverse: Matrix<4>,
    half_width: f64,
    half_height: f64,
    pixel_size: f64,
}

impl Camera {
    pub fn new(hsize: usize, vsize: usize, field_of_view: f64) -> Self {
        let transform = Matrix::identity();
        let transform_inverse = transform;

        let half_view = (field_of_view / 2.0).tan();
        let aspect_ratio = (hsize as f64) / (vsize as f64);
        let (half_width, half_height) = if aspect_ratio >= 1.0 {
            (half_view, half_view / aspect_ratio)
        } else {
            (half_view * aspect_ratio, half_view)
        };
        let pixel_size = (2.0 * half_width) / (hsize as f64);

        Camera {
            hsize,
            vsize,
            field_of_view,
            transform,
            transform_inverse,
            half_width,
            half_height,
            pixel_size,
        }
    }

    pub fn set_transform(&mut self, transform: Matrix<4>) {
        self.transform = transform;
        self.transform_inverse = transform.inverse();
    }

    pub fn ray_for_pixel(&self, x: usize, y: usize) -> Ray {
        let px = x as f64;
        let py = y as f64;

        let xoffset = (px + 0.5) * self.pixel_size;
        let yoffset = (py + 0.5) * self.pixel_size;

        let world_x = self.half_width - xoffset;
        let world_y = self.half_height - yoffset;

        let pixel = self.transform_inverse * Tuple::point(world_x, world_y, -1.0);
        let origin = self.transform_inverse * Tuple::point(0.0, 0.0, 0.0);
        let direction = (pixel - origin).normalize();

        Ray::new(origin, direction)
    }

    pub fn render(&self, world: &World) -> Canvas {
        let mut image = Canvas::new(self.hsize, self.vsize);
        for y in 0..self.vsize {
            for x in 0..self.hsize {
                let ray = self.ray_for_pixel(x, y);
                let color = world.color_at(&ray, 5);
                image.write_pixel(x, y, color);
            }
        }
        image
    }
}
