use crate::materials::Material;
use crate::point3d::Point3D;

#[derive(Debug, Clone, Copy)]
pub struct Ray {
    pub origin: Point3D,
    pub direction: Point3D,
    pub strength: f32,
    pub ray_time: f32,
    pub frequency: i64,
    dist_factor: f32,
}

impl Ray {
    fn free_space_fallof(&self, dist: f32) -> f32 {
        20.0 * (dist / self.dist_factor).log10()
    }
    pub fn new(
        origin: Point3D,
        direction: Point3D,
        strength: f32,
        ray_time: f32,
        frequency: i64,
    ) -> Ray {
        let dist_factor: f32 = {
            let wavelength: f32 = 299792458.0 / ((frequency * 1_000_000) as f32);
            wavelength / (4.0 * std::f32::consts::PI)
        };
        Ray {
            origin,
            direction,
            strength,
            ray_time,
            frequency,
            dist_factor,
        }
    }

    pub fn at(&self, t: f32) -> Point3D {
        self.origin + self.direction * t
    }
    pub fn strength_at(&self, t: f32) -> f32 {
        self.strength - self.free_space_fallof(self.ray_time + t)
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
