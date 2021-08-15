use crate::color::*;
use crate::material::Material;
use crate::tuple::*;

pub enum Light {
    Point { position: Tuple, intensity: Color },
    Directional { direction: Tuple, intensity: Color },
}

impl Light {
    pub fn new_point(position: Tuple, intensity: Color) -> Self {
        Light::Point {
            position,
            intensity,
        }
    }

    pub fn new_directional(direction: Tuple, intensity: Color) -> Self {
        let direction = direction.normalize();
        Light::Directional {
            direction,
            intensity,
        }
    }
}

/// A light source illuminating a point.
pub struct LightSource {
    /// Light source's intensity.
    pub intensity: Color,

    /// Direction vector to the light source.
    pub direction: Tuple,

    /// Distance from the point to the light source.
    pub distance: f64,
}

impl LightSource {
    /// Primary LightSource constructor.
    pub fn new(intensity: Color, direction: Tuple, distance: f64) -> Self {
        LightSource {
            intensity,
            direction,
            distance,
        }
    }
}

/// Illuminate a point using the Phong reflection model.
pub fn phong<'a, L>(material: &Material, light_sources: L, normal: &Tuple, viewer: &Tuple) -> Color
where
    L: Iterator<Item = LightSource>,
{
    let mut intensity = Color::new(1.0, 1.0, 1.0) * material.ambient;
    for light in light_sources {
        let light_dot_normal = light.direction.dot(*normal);
        if light_dot_normal > 0.0 {
            intensity = intensity + light.intensity * material.diffuse * light_dot_normal;
            let reflection = (-light.direction).reflect(*normal);
            let reflection_dot_viewer = reflection.dot(*viewer);
            if reflection_dot_viewer > 0.0 {
                intensity = intensity
                    + light.intensity
                        * material.specular
                        * reflection_dot_viewer.powf(material.shininess);
            }
        }
    }
    intensity
}
