use crate::color::Color;
use crate::light::Light;
use crate::material::*;
use crate::object::*;
use crate::ray::*;
use crate::tuple::Tuple;
use crate::util::*;

pub struct World {
    pub obj_pool: ObjPool,
    pub light: Light,
}

impl World {
    pub fn new(obj_pool: ObjPool, light: Light) -> Self {
        World { obj_pool, light }
    }

    pub fn color_at(&self, ray: &Ray, depth: u8) -> Color {
        let xs = self.obj_pool.intersect(ray);

        let hit = xs.iter().filter(|x| x.t > 0.0).nth(0);

        match hit {
            None => Color::new(0.0, 0.0, 0.0),
            Some(x) => {
                let comps = prepare_computations(x, &ray, &self.obj_pool, &xs);
                self.shade_hit(&comps, depth)
            }
        }
    }

    pub fn shade_hit(&self, comps: &Computations, depth: u8) -> Color {
        let in_shadow = self.is_shadowed(comps.over_point);
        let material = &self.obj_pool.material[comps.object.id];
        let color = if let Some(pattern) = &material.pattern {
            let object_point = self.obj_pool.world_to_object(comps.object, comps.point);
            pattern.color_at_object(object_point)
        } else {
            material.color
        };
        let surface = lighting(
            color,
            material,
            &self.light,
            comps.point,
            comps.eyev,
            comps.normalv,
            in_shadow,
        );
        let reflected = self.reflected_color(&comps, depth);
        let refracted = self.refracted_color(&comps, depth);

        if material.reflective > 0.0 && material.transparency > 0.0 {
            let reflectance = schlick(comps.eyev, comps.normalv, comps.n1, comps.n2);
            surface + reflected * reflectance + refracted * (1.0 - reflectance)
        } else {
            surface + reflected + refracted
        }
    }

    pub fn reflected_color(
        &self,
        comps: &Computations,
        depth: u8) -> Color {
        let material = &self.obj_pool.material[comps.object.id];

        if depth == 0 || close_eq(material.reflective, 0.0) {
            return Color::new(0.0, 0.0, 0.0);
        }

        let reflected_ray = Ray::new(comps.over_point, comps.reflectv);
        let color = self.color_at(&reflected_ray, depth - 1);

        color * material.reflective
    }

    pub fn refracted_color(
        &self,
        comps: &Computations,
        depth: u8
    ) -> Color {
        let material = &self.obj_pool.material[comps.object.id];

        if depth == 0 || close_eq(material.transparency, 0.0) {
            return Color::new(0.0, 0.0, 0.0);
        }

        // testing for "total internal reflection" using Snell's law and some trig.
        let n_ratio = comps.n1 / comps.n2;
        let cos_i = comps.eyev.dot(comps.normalv);
        let sin2_t = n_ratio * n_ratio * (1.0 - cos_i * cos_i);
        if sin2_t > 1.0 {
            return Color::new(0.0, 0.0, 0.0);
        }

        let cos_t = (1.0 - sin2_t).sqrt();
        let direction = comps.normalv * (n_ratio * cos_i - cos_t) - comps.eyev * n_ratio;

        let refracted_ray = Ray::new(comps.under_point, direction);

        self.color_at(&refracted_ray, depth - 1) * material.transparency
    }

    pub fn is_shadowed(&self, point: Tuple) -> bool {
        let lightv = self.light.position - point;
        let distance_to_light = lightv.magnitude();
        let lightv = lightv.normalize();
        let shadow_ray = Ray::new(point, lightv);
        let xs = self.obj_pool.intersect(&shadow_ray);

        xs.iter().any(|x| x.t > 0.0 && x.t < distance_to_light)
    }
}

pub struct Computations {
    pub t: f64,
    pub object: Obj,
    pub point: Tuple,
    pub over_point: Tuple,
    pub under_point: Tuple,
    pub eyev: Tuple,
    pub normalv: Tuple,
    pub inside: bool,
    pub reflectv: Tuple,
    pub n1: f64, // refractive index exited
    pub n2: f64, // refractive index entered
}

pub fn prepare_computations(
    x: &Intersection,
    ray: &Ray,
    object_pool: &ObjPool,
    intersections: &[Intersection],
) -> Computations {
    let point = ray.position(x.t);
    let eyev = -ray.direction;
    let mut normalv = object_pool.normal_at(x.obj, point);
    let inside = normalv.dot(eyev) < 0.0;
    if inside {
        normalv = -normalv;
    }
    let over_point = point + normalv * EPSILON;
    let under_point = point - normalv * EPSILON;
    let reflectv = ray.direction.reflect(normalv);
    let (n1, n2) = {
        let mut n1 = 1.0;
        let mut n2 = 1.0;

        let mut containers: Vec<Obj> = Vec::new();
        for x1 in intersections.iter() {
            if x1.t == x.t {
                n1 = containers
                    .last()
                    .map(|o| object_pool.material[o.id].refractive_index)
                    .unwrap_or(1.0);
            }

            if let Some(index) = containers.iter().position(|o| *o == x1.obj) {
                containers.remove(index);
            } else {
                containers.push(x1.obj);
            }

            if x1.t == x.t {
                n2 = containers
                    .last()
                    .map(|o| object_pool.material[o.id].refractive_index)
                    .unwrap_or(1.0);
                break;
            }
        }

        (n1, n2)
    };
    Computations {
        t: x.t,
        object: x.obj,
        point,
        over_point,
        under_point,
        eyev,
        normalv,
        inside,
        reflectv,
        n1,
        n2,
    }
}

pub fn lighting(
    color: Color,
    material: &Material,
    light: &Light,
    point: Tuple,
    eye_v: Tuple,
    normal_v: Tuple,
    in_shadow: bool,
) -> Color {
    if in_shadow {
        return color * material.ambient;
    }

    let effective_color = color * light.intensity;

    let light_v = (light.position - point).normalize();

    let ambient = effective_color * material.ambient;

    let light_dot_normal = light_v.dot(normal_v);
    if light_dot_normal < 0.0 {
        ambient
    } else {
        let diffuse = effective_color * material.diffuse * light_dot_normal;

        let reflect_v = (-light_v).reflect(normal_v);
        let reflect_dot_eye = reflect_v.dot(eye_v);

        let specular = if reflect_dot_eye <= 0.0 {
            Color::new(0.0, 0.0, 0.0)
        } else {
            let factor = reflect_dot_eye.powf(material.shininess);
            light.intensity * material.specular * factor
        };

        ambient + diffuse + specular
    }
}

/// Schlick approximation of the Fresnel effect.
pub fn schlick(eyev: Tuple, normalv: Tuple, n1: f64, n2: f64) -> f64 {
    let mut cos = eyev.dot(normalv);

    if n1 > n2 {
        let n = n1 / n2;
        let sin2_t = n * n * (1.0 - cos * cos);
        if sin2_t > 1.0 {
            return 1.0;
        }

        let cos_t = (1.0 - sin2_t).sqrt();
        cos = cos_t;
    }

    let r0 = (n1 - n2) / (n1 + n2);
    let r0 = r0 * r0;

    let x = 1.0 - cos;

    r0 + (1.0 - r0) * x * x * x * x * x
}
