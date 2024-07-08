use serde::{Deserialize, Serialize};

use crate::materials::Material;
use crate::point3d::Point3D;
use crate::ray::HitRecord;
use crate::ray::Hittable;
use crate::ray::Ray;

#[cfg(test)]
use crate::materials::Glass;
#[cfg(test)]
use crate::materials::Lambertian;
#[cfg(test)]
use crate::materials::Texture;
#[cfg(test)]
use palette::Srgb;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Cube {
    pub origin: Point3D,
    pub dim_x: f32,
    pub dim_y: f32,
    pub dim_z: f32,
    pub material: Material,
    pub id: i64,
}

impl Cube {
    pub fn new(origin: Point3D, dim_x: f32, dim_y: f32, dim_z: f32, material: Material,id:i64) -> Cube {
        Cube {
            origin,
            dim_x,
            dim_y,
            dim_z,
            material,
            id
        }
    }
}

fn u_v_from_cube_hit_point(hit_point_on_cube: Point3D) -> (f32, f32) {
    let n = hit_point_on_cube.unit_vector();
    let x = n.x();
    let y = n.y();
    let z = n.z();
    let u = (x.atan2(z) / (2.0 * std::f32::consts::PI)) + 0.5;
    let v = y * 0.5 + 0.5;
    (u, v)
}

impl Cube {
    fn get_hit_for_cube(&self, ray: &Ray) -> Option<(Point3D, f32, Point3D)> {
        let mut results = Vec::new();
        for x_dir in [-1.0,1.0] {
            let dist = ray.origin.x() - (self.origin.x() - x_dir * self.dim_x);
            let t = -(dist / ray.direction.x());
            let intersect = ray.at(t);
            if (intersect.y() - self.origin.y()).abs() < self.dim_y
                && (intersect.z() - self.origin.z()).abs() < self.dim_z
            {
                let normal = Point3D::new(-x_dir, 0.0, 0.0);
                if self.origin.y() < 0.2 {
                    //dbg!("x", intersect,ray.origin,dist,self.origin);
                }
                results.push((intersect, t, normal));
            }
        }
        for y_dir in [-1.0, 1.0] {
            let dist =  ray.origin.y() - (self.origin.y() - y_dir * self.dim_y);
            let t = -(dist / ray.direction.y());
            let intersect = ray.at(t);
            if ((intersect.x() - self.origin.x()).abs() < self.dim_x)
                && ((intersect.z() - self.origin.z()).abs() < self.dim_z)
            {
                let normal = Point3D::new(0.0, y_dir, 0.0);
                if self.origin.y() < 0.2 {
                    //dbg!("y", intersect,normal);
                }
                results.push((intersect, t, normal));
            }
        }
        for z_dir in [-1.0,1.0] {
            let dist = ray.origin.z() - (self.origin.z() - z_dir * self.dim_z);
            let t = -(dist / ray.direction.z());
            let intersect = ray.at(t);
            if (intersect.y() - self.origin.y()).abs() < self.dim_y
                && (intersect.x() - self.origin.x()).abs() < self.dim_x
            {
                if self.origin.y() < 0.2 {
                    //dbg!("z", intersect);
                }
                let normal = Point3D::new(0.0, 0.0, z_dir);
                results.push((intersect, t, normal));
            }
        }
        if results.len() == 0 {
            return None;
        }
        if self.origin.x() > 0.1 && self.origin.y() > 0.1 {

        //println!("{:?}",results);
        }
        results.sort_by(|x, y| x.1.partial_cmp(&y.1).unwrap_or(std::cmp::Ordering::Equal));
        let (intersect, t, normal) = results.first().expect("results is non-empty");
        Some((*intersect, *t, *normal))
    }
}

impl Hittable for Cube {
    fn hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitRecord> {
        if let Some((hit_loc, ray_t, normal)) = self.get_hit_for_cube(ray) {
            //let ray_t = ((hit_loc.x() - ray.origin.x()) / ray.direction.x()).abs();
            if ray_t < t_max && ray_t > t_min {
                let (u, v) = u_v_from_cube_hit_point(hit_loc - self.origin);
                return Some(HitRecord {
                    t: ray_t,
                    point: hit_loc,
                    normal: normal,
                    front_face: ray.direction.dot(&normal) > 0.0,
                    material: &self.material,
                    u,
                    v,
                });
            }
        }
        None
    }
}
