use crate::materials::Material;
use crate::point3d::Point3D;

#[cfg(test)]
use assert_approx_eq::assert_approx_eq;

#[derive(Debug, Clone, Copy)]
pub struct Ray {
    pub origin: Point3D,
    pub direction: Point3D,
    pub strength: f32,
    pub ray_time: f32,
    pub frequency: i64,
    dist_factor: f32
}

impl Ray {
    fn free_space_fallof(&self,dist: f32) -> f32{
        return 20.0*(dist/self.dist_factor).log10();
    }
    pub fn new(origin: Point3D, direction: Point3D,strength:f32,ray_time:f32, frequency: i64) -> Ray {
        let dist_factor: f32 = {
            let wavelength:f32 = 299792458.0/((frequency*1_000_000) as f32);
            wavelength/(4.0*std::f32::consts::PI)
        };
        Ray { origin, direction,strength ,ray_time, frequency, dist_factor}
    }

    pub fn at(&self, t: f32) -> Point3D {
        self.origin + self.direction * t
    }
    pub fn strength_at(&self, t: f32) -> f32 {
        return self.strength - self.free_space_fallof(self.ray_time + t);
    }
}

pub struct HitRecord<'material> {
    pub t: f32,
    pub point: Point3D,
    pub normal: Point3D,
    pub front_face: bool,
    pub material: &'material Material,
    pub u: f32,
    pub v: f32,
}

pub trait Hittable {
    fn hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitRecord>;
}

#[test]
fn test_ray() {
    let p = Point3D::new(0.1, 0.2, 0.3);
    let q = Point3D::new(0.2, 0.3, 0.4);

    let r = Ray::new(p, q);

    assert_approx_eq!(r.origin.x(), 0.1);
    assert_approx_eq!(r.origin.y(), 0.2);
    assert_approx_eq!(r.origin.z(), 0.3);
    assert_approx_eq!(r.direction.x(), 0.2);
    assert_approx_eq!(r.direction.y(), 0.3);
    assert_approx_eq!(r.direction.z(), 0.4);
}

#[test]
fn test_ray_at() {
    let p = Point3D::new(0.0, 0.0, 0.0);
    let q = Point3D::new(1.0, 2.0, 3.0);

    let r = Ray::new(p, q);
    let s = r.at(0.5);

    assert_approx_eq!(s.x(), 0.5);
    assert_approx_eq!(s.y(), 1.0);
    assert_approx_eq!(s.z(), 1.5);
}
