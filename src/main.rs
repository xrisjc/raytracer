mod algorithm;
mod camera;
mod canvas;
mod color;
mod light;
mod material;
mod matrix;
mod object;
mod pattern;
mod ray;
mod shape;
mod tuple;
mod util;
mod world;

use camera::*;
use color::*;
use light::*;
use material::*;
use matrix::*;
use object::*;
use pattern::PatternBuilder;
use shape::*;
use tuple::*;
use world::*;

use std::f64::consts::PI;
use std::fs;
use std::time::Instant;

fn main() {
    let result = render_scene();
    if let Result::Err(e) = result {
        eprintln!("error: {}", e);
    }
}

fn render_scene() -> Result<(), Box<dyn std::error::Error>> {
    let (obj_pool, camera, light) = _csg_scene();
    println!("{}", obj_pool);
    let world = World::new(obj_pool, light);

    let render_start = Instant::now();
    let image = camera.render(&world);
    let render_end = Instant::now();
    display_benchmark("render", render_start, render_end);

    fs::write("out.ppm", image.to_ppm()?)?;

    Ok(())
}

fn display_benchmark(label: &str, start: Instant, end: Instant) {
    let duration = end.duration_since(start);
    let secs = duration.as_secs();
    let millis = duration.as_millis() % 1000u128;
    println!("{}: {}s {}ms", label, secs, millis);
}

fn _hex_scene() -> (ObjPool, Camera, Light) {
    fn hexagon_corner(obj_pool: &mut ObjPool) -> Obj {
        let shape = Shape::Sphere;
        let transform = Matrix::translation(0.0, 0.0, -1.0) * Matrix::scaling(0.25, 0.25, 0.25);
        let material = Material::new();
        obj_pool.add_shape(shape, transform, material)
    }

    fn hexagon_edge(obj_pool: &mut ObjPool) -> Obj {
        let shape = Shape::Cylinder {
            y_min: 0.0,
            y_max: 1.0,
            closed: false,
        };
        let transform = Matrix::translation(0.0, 0.0, -1.0)
            * Matrix::rotation_y(-PI / 6.0)
            * Matrix::rotation_z(-PI / 2.0)
            * Matrix::scaling(0.25, 1.0, 0.25);
        let material = Material::new();
        obj_pool.add_shape(shape, transform, material)
    }

    fn hexagon_side(obj_pool: &mut ObjPool, transform: Matrix<4>) -> Obj {
        let side = obj_pool.add_group(transform);

        let corner = hexagon_corner(obj_pool);
        let edge = hexagon_edge(obj_pool);

        obj_pool.add_child(side, corner);
        obj_pool.add_child(side, edge);

        side
    }

    fn hexagon(obj_pool: &mut ObjPool, transform: Matrix<4>) -> Obj {
        let hex = obj_pool.add_group(transform);

        for n in 0..6 {
            let n = n as f64;
            let transform = Matrix::rotation_y(n * PI / 3.0);
            let side = hexagon_side(obj_pool, transform);
            obj_pool.add_child(hex, side);
        }

        hex
    }

    let light = Light::new(Tuple::point(-10.0, 10.0, -10.0), Color::new(1.0, 1.0, 1.0));

    let camera = {
        let (h, w) = (1280, 960);
        let mut camera = Camera::new(w, h, 50.0 * PI / 180.0);
        let from = Tuple::point(0.0, 0.0, -10.0);
        let to = Tuple::point(0.0, 0.0, 0.0);
        let up = Tuple::vector(0.0, 1.0, 0.0);
        camera.set_transform(Matrix::view_transform(from, to, up));
        camera
    };

    let obj_pool = {
        let mut obj_pool = ObjPool::new();

        let transform = Matrix::rotation_x(-PI / 2.0) * Matrix::scaling(1.0, 1.0, 2.0);
        let _hex = hexagon(&mut obj_pool, transform);

        obj_pool
    };

    (obj_pool, camera, light)
}

fn _shield_scene() -> (ObjPool, Camera, Light) {
    let light = Light::new(Tuple::point(-10.0, 10.0, -10.0), Color::new(1.0, 1.0, 1.0));

    let (h, w) = (1280, 960);
    let mut camera = Camera::new(w, h, 50.0 * PI / 180.0);
    let from = Tuple::point(0.0, 1.0, -10.0);
    let to = Tuple::point(0.0, 1.0, 0.0);
    let up = Tuple::vector(0.0, 1.0, 0.0);
    camera.set_transform(Matrix::view_transform(from, to, up));

    let white = Color::new_u8(255, 255, 255);
    let black = Color::new_u8(0, 0, 0);
    let light_steel_blue = Color::new_u8(0xb0, 0xc4, 0xde);
    let sky_blue = Color::new_u8(0x87, 0xce, 0xeb);

    let mut obj_pool = ObjPool::new();

    {
        let shape = Shape::Sphere;
        let scale = 2000.0;
        let transform = Matrix::scaling(scale, scale, scale);
        let mut material = Material::new();
        material.color = sky_blue;
        material.pattern = PatternBuilder::new()
            .gradient(light_steel_blue, sky_blue)
            .transform(Matrix::rotation_z(PI / 2.0))
            .build();
        material.specular = 0.0;
        let _dome = obj_pool.add_shape(shape, transform, material);
    }

    {
        let shape = Shape::Plane;
        let transform = Matrix::identity();
        let mut material = Material::new();
        material.pattern = PatternBuilder::new().checkers(black, white).build();
        material.reflective = 0.1;
        let _ground = obj_pool.add_shape(shape, transform, material);
    }

    {
        let shape = Shape::Cone {
            y_min: 0.0,
            y_max: 3.0,
            closed: false,
        };
        let transform = Matrix::translation(0.0, 3.0, 0.0)
            * Matrix::rotation_y(PI / 2.0)
            * Matrix::rotation_z(PI / 2.0);
        let mut material = Material::new();
        material.color = Color::new(0.25, 0.25, 0.25);
        material.ambient = 0.01;
        material.diffuse = 0.1;
        material.specular = 1.0;
        material.shininess = 300.0;
        material.reflective = 1.0;
        let _shield = obj_pool.add_shape(shape, transform, material);
    }

    (obj_pool, camera, light)
}


fn _csg_scene() -> (ObjPool, Camera, Light) {
    let light = Light::new(Tuple::point(-10.0, 10.0, -10.0), Color::new(1.0, 1.0, 1.0));

    let (h, w) = (1280, 960);
    let (h, w) = (2048, 1536);
    let (w, h) = (3840, 2160);
    let mut camera = Camera::new(w, h, 50.0 * PI / 180.0);
    let from = Tuple::point(1.0, 4.0, -10.0);
    let to = Tuple::point(1.0, 0.0, 0.0);
    let up = Tuple::vector(0.0, 1.0, 0.0);
    camera.set_transform(Matrix::view_transform(from, to, up));

    let white_smoke = Color::new_u8(0xf5, 0xf5, 0xf5);
    let dim_gray = Color::new_u8(0x69, 0x69, 0x69);
    let light_steel_blue = Color::new_u8(0xb0, 0xc4, 0xde);
    let sky_blue = Color::new_u8(0x87, 0xce, 0xeb);
    let linen = Color::new_u8(0xfa, 0xf0, 0xe6);
    let honey_dew = Color::new_u8(0xf0, 0xff, 0xf0);
    let alice_blue = Color::new_u8(0xf0, 0xf8, 0xff);

    let mut obj_pool = ObjPool::new();

    {
        let shape = Shape::Sphere;
        let scale = 2000.0;
        let transform = Matrix::scaling(scale, scale, scale);
        let mut material = Material::new();
        material.color = sky_blue;
        material.pattern = PatternBuilder::new()
            .gradient(light_steel_blue, sky_blue)
            .transform(Matrix::rotation_z(PI / 2.0))
            .build();
        material.specular = 0.0;
        let _dome = obj_pool.add_shape(shape, transform, material);
    }

    {
        let shape = Shape::Plane;
        let transform = Matrix::identity();
        let mut material = Material::new();
        material.pattern = PatternBuilder::new().checkers(dim_gray, white_smoke).build();
        material.reflective = 0.1;
        let _ground = obj_pool.add_shape(shape, transform, material);
    }

    let s1 = {
        let shape = Shape::Sphere;
        let transform =  Matrix::scaling(1.25, 1.25, 1.25);
        let mut material = Material::new();
        material.color = honey_dew;
        obj_pool.add_shape(shape, transform, material)
    };

    let c1 = {
        let shape = Shape::Cube;
        let transform = Matrix::identity();
        let mut material = Material::new();
        material.color = alice_blue;
        obj_pool.add_shape(shape, transform, material)
    };

    let csg = {
        let op = CsgOp::Difference;
        let transform = Matrix::translation(0.0, 1.0, 0.0) * Matrix::rotation_y(PI/4.0);
        obj_pool.add_csg(op, transform, c1, s1)
    };

    (obj_pool, camera, light)
}
