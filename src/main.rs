extern crate cgmath;
extern crate image;
extern crate rand;

use rand::Rng;
use std::f64;

use camera::*;
use defns::*;

pub mod camera;
pub mod defns;

struct Intersection<'a> {
    r: &'a Ry,
    t: f64,
    n: Vt,
}

impl<'a> Intersection<'a> {
    fn p(&self) -> Pt {
        at(&self.r, self.t)
    }
}

trait Intersect {
    fn intersect<'a>(&self, r: &'a Ry) -> Option<Intersection<'a>>;
}

fn at(r: &Ry, t: f64) -> Pt {
    return r.origin.add_v(&r.direction.mul_s(t));
}

impl Intersect for Sph {
    fn intersect<'a>(&self, r: &'a Ry) -> Option<Intersection<'a>> {
        let relcenter = r.origin.sub_p(&self.center);
        let b = r.direction.dot(&relcenter);
        let c = relcenter.dot(&relcenter) - self.radius * self.radius;
        let disc = b * b - c;
        if disc < 0.0 {
            return None;
        }
        let sqrtdisc = disc.sqrt();
        let t1 = -b - sqrtdisc;
        let t2 = -b + sqrtdisc;
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
    geom: Sph,
}

fn sphere(light: bool, center: Pt, radius: f64) -> Geometry {
    Geometry {
        light: light,
        geom: Sph { center: center, radius: radius }
    }
}

struct Scene {
    bg: Color,
    geoms: Vec<Geometry>,
}

/*
fn reflect(dir: &Vt, n: &Vt) -> Vt {
    dir.sub_v(&n.mul_s(2.0 * dot(*dir, *n)))
}

fn mirror<'a>(isx: &Intersection<'a>) -> Ry {
    let dir = reflect(&isx.r.direction, &isx.n);
    Ry::new(isx.p().add_v(&dir.mul_s(0.0001)), dir)
}
*/

fn diffuse<'a>(isx: &Intersection<'a>) -> Ry {
    let mut rng = rand::weak_rng();

    let u1 = rng.next_f64();
    let u2 = rng.next_f64();
    let r = u1.sqrt();
    let theta = 2.0 * f64::consts::PI * u2;
    let x = r * theta.cos();
    let y = r * theta.sin();
    let z = f64::max(0.0, 1.0 - u1).sqrt();

    let nor = isx.n;
    let tan = isx.n.cross(&Vt::unit_x()).normalize();
    let bin = nor.cross(&tan);

    let dir = (tan.mul_s(x) + bin.mul_s(y) + nor.mul_s(z)).normalize();
    Ry::new(isx.p().add_v(&dir.mul_s(0.00001)), dir)
}

impl Scene {
    fn intersect<'a>(&'a self, r: &'a Ry) -> Option<(&'a Geometry, Intersection<'a>)> {
        let mut tmin = std::f64::MAX;
        let mut ret: Option<(&'a Geometry, Intersection<'a>)> = None;
        for ref g in &self.geoms {
            match g.geom.intersect(r) {
                Some(ix) => if ix.t < tmin {
                    tmin = ix.t;
                    ret = Some((g, ix));
                },
                None => {}
            }
        }
        ret
    }

    fn trace(&self, r: &Ry, depth: u32) -> Color {
        if depth <= 0 {
            return self.bg;
        }

        let scatter = diffuse;

        match self.intersect(r) {
            Some((g, isx)) => {
                if g.light {
                    LIGHT_COLOR
                } else {
                    //let d = scatter(&isx);
                    //Vt::new(isx.n.x.abs(),
                    //        isx.n.y.abs(),
                    //        isx.n.z.abs())
                    MAT_COLOR * self.trace(&scatter(&isx), depth - 1)
                }
            },
            None => self.bg,
        }
    }

    fn multitrace(&self, r: &Ry, depth: u32, iters: u32) -> Color {
        //(0..iters).map(|_| self.trace(r, depth)).sum().div_s(iters as f64)
        let mut sum = Vt::new(0.0, 0.0, 0.0);
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

fn to_color(c: Vt) -> image::Rgb<u8> {
    image::Rgb([
        (clamp01(c.x) * 255.0) as u8,
        (clamp01(c.y) * 255.0) as u8,
        (clamp01(c.z) * 255.0) as u8,
    ])
}

fn render(s: &Scene, cam: &Camera) {
    let mut image = image::ImageBuffer::new(cam.dim.0, cam.dim.1);

    for i in 0..cam.dim.0 {
        for j in 0..cam.dim.1 {
            let ray = cam.ray(i, j);
            let c = s.multitrace(&ray, MAX_DEPTH, ITERS);
            image.put_pixel(i as u32, j as u32, to_color(c));
        }
    }

    image.save("render.png").ok().expect("Failed to save rendered image");
}

const   MAT_COLOR: Vt = Vt { x: 0.95, y: 0.95, z: 0.95 };
const LIGHT_COLOR: Vt = Vt { x: 5.00, y: 5.00, z: 5.00 };
const MAX_DEPTH: u32 = 5;
const ITERS: u32 = 100;
const DIM: (u32, u32) = (200, 200);

fn main() {
    let s = Scene {
        bg: Color::new(0.2, 0.2, 0.2),
        geoms: vec![
            sphere(true,  Pt::new(0.0, 0.0, 102.), 100.005),
            sphere(false, Pt::new(0.0, 0.0, 0.0), 1.0),
            sphere(false, Pt::new(1002., 0.0, 0.0), 1000.),
            sphere(false, Pt::new(-1002., 0.0, 0.0), 1000.),
            sphere(false, Pt::new(0.0, 1002., 0.0), 1000.),
            sphere(false, Pt::new(0.0, 0.0, 1002.), 1000.),
            sphere(false, Pt::new(0.0, 0.0, -1002.), 1000.),
        ],
    };

    let c = Camera::new(
        Pt::new(0.0, -6.0, 0.0),
        Pt::new(0.0, 0.0, 0.0),
        Vt::new(0.0, 0.0, 1.0),
        DIM, 0.5);

    render(&s, &c);
}
