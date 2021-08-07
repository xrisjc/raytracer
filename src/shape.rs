use crate::object::{Intersection, Obj};
use crate::ray::*;
use crate::tuple::*;
use crate::util::*;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Shape {
    /// The XZ plane.
    Plane,

    /// A unit sphere.
    Sphere,

    /// Axis aligned bounding box.
    Cube,

    Cylinder {
        y_min: f64,
        y_max: f64,
        closed: bool,
    },

    Cone {
        y_min: f64,
        y_max: f64,
        closed: bool,
    },
}

impl Shape {
    pub fn intersects(&self, ray: Ray, id: Obj, xs: &mut Vec<Intersection>) {
        match self {
            Shape::Plane => {
                if ray.direction.y().abs() < EPSILON {
                    // ray is parallel to the plane
                } else {
                    let t = -ray.origin.y() / ray.direction.y();
                    xs.push(Intersection::new_shape(t, id));
                }
            }

            Shape::Sphere => {
                let sphere_to_ray = ray.origin - Tuple::point(0.0, 0.0, 0.0);
                let a = ray.direction.dot(ray.direction);
                let b = 2.0 * ray.direction.dot(sphere_to_ray);
                let c = sphere_to_ray.dot(sphere_to_ray) - 1.0;
                let discriminant = b * b - 4.0 * a * c;
                if discriminant < 0.0 {
                    // no intersections
                } else {
                    let t1 = (-b - discriminant.sqrt()) / (2.0 * a);
                    let t2 = (-b + discriminant.sqrt()) / (2.0 * a);
                    xs.push(Intersection::new_shape(t1, id));
                    xs.push(Intersection::new_shape(t2, id));
                }
            }

            Shape::Cube => {
                fn check_axis(origin: f64, direction: f64) -> (f64, f64) {
                    let tmin_numerator = -1.0 - origin;
                    let tmax_numerator = 1.0 - origin;
                    let (tmin, tmax) = if direction.abs() >= EPSILON {
                        (tmin_numerator / direction, tmax_numerator / direction)
                    } else {
                        (
                            tmin_numerator * f64::INFINITY,
                            tmax_numerator * f64::INFINITY,
                        )
                    };
                    if tmin > tmax {
                        (tmax, tmin)
                    } else {
                        (tmin, tmax)
                    }
                }

                let (xtmin, xtmax) = check_axis(ray.origin.x(), ray.direction.x());
                let (ytmin, ytmax) = check_axis(ray.origin.y(), ray.direction.y());
                let (ztmin, ztmax) = check_axis(ray.origin.z(), ray.direction.z());

                let tmin = xtmin.max(ytmin).max(ztmin);
                let tmax = xtmax.min(ytmax).min(ztmax);

                if tmin > tmax {
                    // no intersections
                } else {
                    xs.push(Intersection::new_shape(tmin, id));
                    xs.push(Intersection::new_shape(tmax, id));
                }
            }

            Shape::Cylinder {
                y_min,
                y_max,
                closed,
            } => {
                let a =
                    ray.direction.x() * ray.direction.x() + ray.direction.z() * ray.direction.z();
                if !close_eq(a, 0.0) {
                    let b = 2.0 * ray.origin.x() * ray.direction.x()
                        + 2.0 * ray.origin.z() * ray.direction.z();
                    let c = ray.origin.x() * ray.origin.x() + ray.origin.z() * ray.origin.z() - 1.0;
                    let disc = b * b - 4.0 * a * c;

                    if disc >= 0.0 {
                        let t0 = (-b - disc.sqrt()) / (2.0 * a);
                        let t1 = (-b + disc.sqrt()) / (2.0 * a);
                        let (t0, t1) = if t0 > t1 { (t1, t0) } else { (t0, t1) };

                        let y0 = ray.origin.y() + t0 * ray.direction.y();
                        if *y_min < y0 && y0 < *y_max {
                            xs.push(Intersection::new_shape(t0, id));
                        }

                        let y1 = ray.origin.y() + t1 * ray.direction.y();
                        if *y_min < y1 && y1 < *y_max {
                            xs.push(Intersection::new_shape(t1, id));
                        }
                    }
                }

                if *closed && !close_eq(ray.direction.y(), 0.0) {
                    fn check_cap(ray: Ray, t: f64) -> bool {
                        let x = ray.origin.x() + t * ray.direction.x();
                        let z = ray.origin.z() + t * ray.direction.z();
                        x * x + z * z <= 1.0
                    }
                    let t = (*y_min - ray.origin.y()) / ray.direction.y();
                    if check_cap(ray, t) {
                        xs.push(Intersection::new_shape(t, id));
                    }
                    let t = (*y_max - ray.origin.y()) / ray.direction.y();
                    if check_cap(ray, t) {
                        xs.push(Intersection::new_shape(t, id));
                    }
                }
            }

            Shape::Cone {
                y_min,
                y_max,
                closed,
            } => {
                let d = ray.direction;
                let o = ray.origin;

                let a = d.x() * d.x() - d.y() * d.y() + d.z() * d.z();
                let b = 2.0 * o.x() * d.x() - 2.0 * o.y() * d.y() + 2.0 * o.z() * d.z();

                let a_is_zero = close_eq(a, 0.0);
                let b_is_zero = close_eq(b, 0.0);

                if a_is_zero && b_is_zero {
                    // no intersections
                } else if a_is_zero {
                    let c = o.x() * o.x() - o.y() * o.y() + o.z() * o.z();
                    let t = -c / (2.0 * b);
                    xs.push(Intersection::new_shape(t, id));
                } else {
                    let c = o.x() * o.x() - o.y() * o.y() + o.z() * o.z();
                    let disc = b * b - 4.0 * a * c;
                    if disc >= 0.0 {
                        let t0 = (-b - disc.sqrt()) / (2.0 * a);
                        let t1 = (-b + disc.sqrt()) / (2.0 * a);
                        let (t0, t1) = if t0 > t1 { (t1, t0) } else { (t0, t1) };
                        let y0 = ray.origin.y() + t0 * ray.direction.y();
                        if *y_min < y0 && y0 < *y_max {
                            xs.push(Intersection::new_shape(t0, id));
                        }

                        let y1 = ray.origin.y() + t1 * ray.direction.y();
                        if *y_min < y1 && y1 < *y_max {
                            xs.push(Intersection::new_shape(t1, id));
                        }
                    }
                }

                if *closed && !close_eq(ray.direction.y(), 0.0) {
                    fn check_cap(ray: Ray, t: f64, y: f64) -> bool {
                        let x = ray.origin.x() + t * ray.direction.x();
                        let z = ray.origin.z() + t * ray.direction.z();
                        x * x + z * z <= y.abs()
                    }

                    let t = (*y_min - ray.origin.y()) / ray.direction.y();
                    if check_cap(ray, t, *y_min) {
                        xs.push(Intersection::new_shape(t, id));
                    }
                    let t = (*y_max - ray.origin.y()) / ray.direction.y();
                    if check_cap(ray, t, *y_max) {
                        xs.push(Intersection::new_shape(t, id));
                    }
                }
            }
        }
    }

    pub fn normal_at(&self, object_point: Tuple) -> Tuple {
        match self {
            Shape::Plane => Tuple::vector(0.0, 1.0, 0.0),

            Shape::Sphere => object_point,

            Shape::Cube => {
                let x = object_point.x();
                let y = object_point.y();
                let z = object_point.z();
                let absx = x.abs();
                let absy = y.abs();
                let absz = z.abs();

                match absx.max(absy).max(absz) {
                    max if max == absx => Tuple::vector(x, 0.0, 0.0),
                    max if max == absy => Tuple::vector(0.0, y, 0.0),
                    _ => Tuple::vector(0.0, 0.0, z),
                }
            }

            Shape::Cylinder { y_min, y_max, .. } => {
                let dist =
                    object_point.x() * object_point.x() + object_point.z() * object_point.z();

                if dist < 1.0 && object_point.y() >= *y_max - EPSILON {
                    Tuple::vector(0.0, 1.0, 0.0)
                } else if dist < 1.0 && object_point.y() <= *y_min + EPSILON {
                    Tuple::vector(0.0, -1.0, 0.0)
                } else {
                    Tuple::vector(object_point.x(), 0.0, object_point.z())
                }
            }

            Shape::Cone { y_min, y_max, .. } => {
                let p = &object_point;

                let dist = p.x() * p.x() + p.z() * p.z();
                let max_dist = p.y().abs();

                if dist < max_dist && object_point.y() >= *y_max - EPSILON {
                    Tuple::vector(0.0, 1.0, 0.0)
                } else if dist < max_dist && object_point.y() <= *y_min + EPSILON {
                    Tuple::vector(0.0, -1.0, 0.0)
                } else {
                    let y = (p.x() * p.x() + p.z() * p.z()).sqrt();
                    let y = if p.y() > 0.0 { -y } else { y };
                    Tuple::vector(p.x(), y, p.z()).normalize()
                }
            }
        }
    }
}
