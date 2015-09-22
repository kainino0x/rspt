use cgmath::{
    Ray,
    Ray3,
    Point,
    Point3,
    EuclideanVector,
    Vector,
    Vector3,
};


#[derive(Clone, Copy)]
pub struct Camera {
    eye: Point3<f64>,
    view: Vector3<f64>,
    up: Vector3<f64>,
    right: Vector3<f64>,
    pub width: u32,
    pub height: u32,
    fovy: f64
}

impl Camera {
    fn ray_clip_space(&self, x: f64, y: f64) -> Ray3<f64> {
        let scaled_up = self.up.mul_s(self.fovy.tan());
        let scaled_right = self.right.mul_s(self.fovy.tan() *
                                    (self.width as f64 / self.height as f64));
        let d = self.view.add_v(&scaled_up.mul_s(y)).add_v(&scaled_right.mul_s(x));
        Ray::new(self.eye, d.normalize())
    }

    pub fn ray(&self, x: u32, y: u32) -> Ray3<f64>  {
        let new_x =       (x as f64) * 2.0 / (self.width as f64) - 1.0;
        let new_y = 1.0 - (y as f64) * 2.0 / (self.height as f64);
        self.ray_clip_space(new_x, new_y)
    }

    pub fn new(eye: Point3<f64>, center: Point3<f64>, up: Vector3<f64>,
               width: u32, height: u32, fovy: f64) -> Camera {
        let norm_up = up.normalize();
        let norm_view = center.sub_p(&eye).normalize();
        let norm_right = norm_view.cross(&norm_up);
        let perp_up = norm_right.cross(&norm_view);
        Camera { eye: eye, view: norm_view, up: perp_up, right: norm_right,
            width: width, height: height, fovy: fovy }

    }
}
