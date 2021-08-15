use crate::material::*;
use crate::matrix::*;
use crate::ray::*;
use crate::shape::*;
use crate::tuple::*;

use std::fmt;

/// Handle to an object in an object pool.
pub type Obj = usize;

/// Constructive Solid Geometry operations.
#[derive(Copy, Clone, Debug)]
pub enum CsgOp {
    Union,
    Intersection,
    Difference,
}

#[derive(Debug)]
enum ObjTag {
    /// A primitive shape.
    Shape(Shape),

    /// An object composed of other objects.
    Group,

    /// A Constructive Solid Geomtery object.
    Csg(CsgOp),
}

#[derive(Copy, Clone)]
pub struct Intersection {
    pub t: f64,
    pub obj: Obj,
}

impl Intersection {
    pub fn new_shape(t: f64, obj: Obj) -> Self {
        Intersection { t, obj }
    }
}

pub struct ObjPool {
    tag: Vec<ObjTag>,
    pub transform_inverse: Vec<Matrix<4>>,
    pub material: Vec<Material>,
    parent: Vec<Option<Obj>>,
    left: Vec<Option<Obj>>,
    right: Vec<Option<Obj>>,
}

impl ObjPool {
    pub fn new() -> Self {
        ObjPool {
            tag: Vec::new(),
            transform_inverse: Vec::new(),
            material: Vec::new(),
            parent: Vec::new(),
            left: Vec::new(),
            right: Vec::new(),
        }
    }

    fn next_id(&self) -> usize {
        self.tag.len()
    }

    fn add(&mut self, tag: ObjTag, transform: Matrix<4>, material: Material) -> Obj {
        let id = self.next_id();

        self.tag.push(tag);
        self.transform_inverse.push(transform.inverse());
        self.material.push(material);
        self.parent.push(None);
        self.left.push(None);
        self.right.push(None);

        id
    }

    pub fn add_shape(&mut self, shape: Shape, transform: Matrix<4>, material: Material) -> Obj {
        self.add(ObjTag::Shape(shape), transform, material)
    }

    pub fn add_group(&mut self, transform: Matrix<4>) -> Obj {
        self.add(ObjTag::Group, transform, Material::new())
    }

    pub fn add_child(&mut self, parent: Obj, child: Obj) {
        self.parent[child] = Some(parent);

        if let Some(first_child) = self.left[parent] {
            // Parent has children. Find next available sibling attribute in children.
            let mut c = first_child;
            while let Some(sibling) = self.right[c] {
                c = sibling;
            }
            self.right[c] = Some(child);
        } else {
            // First child. Set parent's child attribute.
            self.left[parent] = Some(child);
        }
    }

    pub fn add_csg(&mut self, op: CsgOp, transform: Matrix<4>, left: Obj, right: Obj) -> Obj {
        let csg = self.add(ObjTag::Csg(op), transform, Material::new());
        self.parent[left] = Some(csg);
        self.parent[right] = Some(csg);
        self.left[csg] = Some(left);
        self.right[csg] = Some(right);
        csg
    }

    pub fn intersect(&self, ray: &Ray) -> Vec<Intersection> {
        fn intersect_rec(obj_pool: &ObjPool, root: Obj, ray: &Ray, xs: &mut Vec<Intersection>) {
            let ray = ray.transform(obj_pool.transform_inverse[root]);
            match &obj_pool.tag[root] {
                ObjTag::Shape(shape) => shape.intersects(ray, root, xs),
                ObjTag::Group => {
                    let mut child = obj_pool.left[root];
                    while let Some(c) = child {
                        intersect_rec(obj_pool, c, &ray, xs);
                        child = obj_pool.right[c];
                    }
                }
                ObjTag::Csg(op) => {
                    let left = obj_pool.left[root].unwrap();
                    let right = obj_pool.right[root].unwrap();

                    let mut csg_xs = Vec::new();
                    intersect_rec(obj_pool, left, &ray, &mut csg_xs);
                    intersect_rec(obj_pool, right, &ray, &mut csg_xs);
                    csg_xs.sort_by(|x1, x2| x1.t.partial_cmp(&x2.t).unwrap());

                    let mut in_left = false;
                    let mut in_right = false;
                    for x in csg_xs.iter() {
                        let left_hit = obj_pool.includes(x.obj, left);
                        let include_x = match op {
                            CsgOp::Union => left_hit && !in_right || !left_hit && !in_left,
                            CsgOp::Intersection => left_hit && in_right || !left_hit && in_left,
                            CsgOp::Difference => left_hit && !in_right || !left_hit && in_left,
                        };
                        if include_x {
                            xs.push(*x);
                        }
                        if left_hit {
                            in_left = !in_left;
                        } else {
                            in_right = !in_right;
                        }
                    }
                }
            }
        }

        let mut xs = Vec::new();

        for id in 0..self.next_id() {
            if self.parent[id].is_none() {
                let root = id;
                intersect_rec(self, root, ray, &mut xs);
            }
        }

        xs.sort_by(|x1, x2| x1.t.partial_cmp(&x2.t).unwrap());

        xs
    }

    pub fn normal_at(&self, obj: Obj, world_point: Tuple) -> Tuple {
        match &self.tag[obj] {
            ObjTag::Shape(shape) => {
                let object_point = self.world_to_object(obj, world_point);
                let object_normal = shape.normal_at(object_point);
                self.normal_to_world(obj, object_normal)
            }
            ObjTag::Group => panic!("cannot take a normal of a group object"),
            ObjTag::Csg(_) => unimplemented!(),
        }
    }

    pub fn world_to_object(&self, obj: Obj, point: Tuple) -> Tuple {
        let point = if let Some(parent) = self.parent[obj] {
            self.world_to_object(parent, point)
        } else {
            point
        };

        self.transform_inverse[obj] * point
    }

    fn normal_to_world(&self, obj: Obj, normal: Tuple) -> Tuple {
        let normal = {
            let mut n = self.transform_inverse[obj].transpose() * normal;
            n.set_w(0.0);
            n.normalize()
        };

        if let Some(parent) = self.parent[obj] {
            self.normal_to_world(parent, normal)
        } else {
            normal
        }
    }

    fn includes(&self, search_target: Obj, node: Obj) -> bool {
        if search_target == node {
            return true;
        }

        match &self.tag[node] {
            ObjTag::Shape(_) => false,
            ObjTag::Group => {
                let mut child = self.left[node];
                while let Some(c) = child {
                    if self.includes(search_target, c) {
                        return true;
                    }
                    child = self.right[c];
                }
                false
            }
            ObjTag::Csg(_) => {
                if let Some(left) = self.left[node] {
                    if self.includes(search_target, left) {
                        return true;
                    }
                }

                if let Some(right) = self.right[node] {
                    if self.includes(search_target, right) {
                        return true;
                    }
                }

                false
            }
        }
    }
}

impl fmt::Display for ObjPool {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fn write_rec(
            obj_pool: &ObjPool,
            f: &mut fmt::Formatter<'_>,
            root: Obj,
            depth: usize,
        ) -> fmt::Result {
            for _ in 0..depth {
                write!(f, "    ")?;
            }

            match &obj_pool.tag[root] {
                ObjTag::Shape(shape) => writeln!(f, "{:?}", shape)?,
                ObjTag::Group => {
                    writeln!(f, "Group")?;
                    let mut child = obj_pool.left[root];
                    while let Some(c) = child {
                        write_rec(obj_pool, f, c, depth + 1)?;
                        child = obj_pool.right[c];
                    }
                }
                ObjTag::Csg(op) => {
                    writeln!(f, "CSG({:?})", op)?;
                    if let Some(left) = obj_pool.left[root] {
                        write_rec(obj_pool, f, left, depth + 1)?;
                    }
                    if let Some(right) = obj_pool.right[root] {
                        write_rec(obj_pool, f, right, depth + 1)?;
                    }
                }
            }

            Ok(())
        }

        for id in 0..self.next_id() {
            if self.parent[id].is_none() {
                let root = id;
                write_rec(self, f, root, 0)?;
            }
        }

        Ok(())
    }
}
