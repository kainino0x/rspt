use defns::*;

#[derive(Clone, Copy)]
pub struct Camera { pub dim: (u32, u32), o: Pt, v: Vt, u: Vt, r: Vt }

impl Camera {
    pub fn ray(&self, x: u32, y: u32) -> Ry  {
        let xn = (x as f64) * 2.0 / (self.dim.0 as f64) - 1.0;
        let yn = (y as f64) * 2.0 / (self.dim.1 as f64) - 1.0;
        let d = self.v.add_v(&self.u.mul_s(-yn)).add_v(&self.r.mul_s(xn));
        Ry::new(self.o, d.normalize())
    }

    pub fn new(eye: Pt, ctr: Pt, up: Vt, dim: (u32, u32), fovy: f64) -> Camera {
        let aspect = dim.0 as f64 / dim.1 as f64;
        let v = ctr.sub_p(&eye).normalize();
        let r = v.cross(&up.normalize()).mul_s(fovy.tan() * aspect);
        let u = r.normalize().cross(&v).mul_s(fovy.tan());
        Camera { dim: dim, o: eye, v: v, u: u, r: r }
    }
}
