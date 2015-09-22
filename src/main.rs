extern crate cgmath;
extern crate image;
extern crate rand;

use cgmath::{
    Point,
    EuclideanVector,
    Vector,
    dot,
};
use rand::Rng;
use std::f64;

use camera::*;

pub mod camera;

type Color  = cgmath::Vector3<f64>;
type Point3 = cgmath::Point3<f64>;
type Ray3   = cgmath::Ray3<f64>;
type Sphere = cgmath::Sphere<f64>;
type Vec3   = cgmath::Vector3<f64>;

struct Intersection<'a> {
    r: &'a Ray3,
    t: f64,
    n: Vec3,
}

impl<'a> Intersection<'a> {
    fn p(&self) -> Point3 {
        at(&self.r, self.t)
    }
}

trait Intersect {
    fn intersect<'a>(&self, r: &'a Ray3) -> Option<Intersection<'a>>;
}

fn at(r: &Ray3, t: f64) -> Point3 {
    return r.origin.add_v(&r.direction.mul_s(t));
}

impl Intersect for Sphere {
    fn intersect<'a>(&self, r: &'a Ray3) -> Option<Intersection<'a>> {
        let relcenter = r.origin.sub_p(&self.center);
        let a = dot(r.direction, r.direction);
        let b = 2.0 * dot(r.direction, relcenter);
        let c = dot(relcenter, relcenter) - self.radius * self.radius;
        let sqdisc = b * b - 4.0 * a * c;
        if sqdisc < 0.0 {
            return None;
        }
        let disc = sqdisc.sqrt();
        let t1 = (-b - disc) / (2.0 * a);
        let t2 = (-b + disc) / (2.0 * a);
        if t2 < 0.0 {
            return None;
        }
        let t = if t1 > 0.0 { t1 } else { t2 };
        let p = at(r, t);
        let n = p.sub_p(&self.center).normalize();
        Some(Intersection { r: &r, t: t, n: n })
    }
}

struct Geometry {
    light: bool,
    geom: Sphere,
}

fn sphere(light: bool, center: Point3, radius: f64) -> Geometry {
    Geometry {
        light: light,
        geom: Sphere { center: center, radius: radius }
    }
}

struct Scene {
    bg: Color,
    geoms: Vec<Geometry>,
}

/*
fn reflect(dir: &Vec3, n: &Vec3) -> Vec3 {
    dir.sub_v(&n.mul_s(2.0 * dot(*dir, *n)))
}

fn mirror<'a>(isx: &Intersection<'a>) -> Ray3 {
    let dir = reflect(&isx.r.direction, &isx.n);
    Ray3::new(isx.p().add_v(&dir.mul_s(0.0001)), dir)
}
*/

fn diffuse<'a>(isx: &Intersection<'a>) -> Ray3 {
    let mut rng = rand::weak_rng();

    let u1 = rng.next_f64();
    let u2 = rng.next_f64();
    let r = u1.sqrt();
    let theta = 2.0 * f64::consts::PI * u2;
    let x = r * theta.cos();
    let y = r * theta.sin();
    let z = f64::max(0.0, 1.0 - u1).sqrt();

    let nor = isx.n;
    let tan = isx.n.cross(&Vec3::unit_x());
    let bin = nor.cross(&tan);

    let dir = tan.mul_s(x) + bin.mul_s(y) + nor.mul_s(z);
    Ray3::new(isx.p().add_v(&dir.mul_s(0.0001)), dir)
}

impl Scene {
    fn intersect<'a>(&'a self, r: &'a Ray3) -> Option<(&'a Geometry, Intersection<'a>)> {
        let tmin = std::f64::MAX;
        let mut ret: Option<(&'a Geometry, Intersection<'a>)> = None;
        for ref g in &self.geoms {
            match g.geom.intersect(r) {
                Some(ix) => if ix.t < tmin { ret = Some((g, ix)); },
                None => {}
            }
        }
        ret
    }

    fn trace(&self, r: &Ray3, depth: u32) -> Color {
        if depth <= 0 {
            return self.bg;
        }

        let scatter = diffuse;

        match self.intersect(r) {
            Some((g, isx)) => {
                if g.light {
                    LIGHT_COLOR
                } else {
                    MAT_COLOR * self.trace(&scatter(&isx), depth - 1)
                }
            },
            None => self.bg,
        }
    }

    fn multitrace(&self, r: &Ray3, depth: u32, iters: u32) -> Color {
        //(0..iters).map(|_| self.trace(r, depth)).sum().div_s(iters as f64)
        let mut sum = Vec3::new(0.0, 0.0, 0.0);
        for _ in 0..iters {
            sum = sum.add_v(&self.trace(r, depth));
        }
        sum.div_s(iters as f64)
    }
}

fn clamp(n: f64, min: f64, max: f64) -> f64 {
    n.min(max).max(min)
}

fn clamp01(n: f64) -> f64 {
    clamp(n, 0.0, 1.0)
}

fn to_color(c: Vec3) -> image::Rgb<u8> {
    image::Rgb([
        (clamp01(c.x) * 255.0) as u8,
        (clamp01(c.y) * 255.0) as u8,
        (clamp01(c.z) * 255.0) as u8,
    ])
}

fn render(s: &Scene, cam: &Camera) {
    let mut image = image::ImageBuffer::new(cam.width as u32, cam.height as u32);

    for i in 0..cam.width {
        for j in 0..cam.height {
            let ray = cam.ray(i, j);
            let c = s.multitrace(&ray, MAX_DEPTH, ITERS);
            image.put_pixel(i as u32, j as u32, to_color(c));
        }
    }

    image.save("render.png").ok().expect("Failed to save rendered image");
}

const   MAT_COLOR: Vec3 = Vec3 { x: 0.95, y: 0.95, z: 0.95 };
const LIGHT_COLOR: Vec3 = Vec3 { x: 2.00, y: 2.00, z: 2.00 };
const MAX_DEPTH: u32 = 5;
const ITERS: u32 = 25;
const BIG: f64 = 100_000.0;
const BOXRAD: f64 = 2000.0;
const WIDTH: u32 = 200;
const HEIGHT: u32 = 200;

fn main() {
    let s = Scene {
        bg: Color::new(0.3, 0.3, 0.3),
        geoms: vec![
            sphere(true,  Point3::new(0.0, 0.0, 2.0), 0.5),
            sphere(false, Point3::new(0.0, 0.0, 0.0), 1.0),
            sphere(false, Point3::new( BIG + BOXRAD, 0.0, 0.0), BIG),
            //sphere(false, Point3::new(-BIG - BOXRAD, 0.0, 0.0), BIG),
            //sphere(false, Point3::new(0.0,  BIG + BOXRAD, 0.0), BIG),
            ////sphere(false, Point3::new(0.0, -BIG - BOXRAD, 0.0), BIG),
            //sphere(false, Point3::new(0.0, 0.0,  BIG + BOXRAD), BIG),
            //sphere(false, Point3::new(0.0, 0.0, -BIG - BOXRAD), BIG),
        ],
    };

    let c = Camera::new(
        Point3::new(0.0, -10.0, 0.0),
        Point3::new(0.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, 1.0),
        WIDTH, HEIGHT, 0.5
        );

    render(&s, &c);
}
