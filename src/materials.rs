use jpeg_decoder::Decoder;
use palette::Srgb;
use rand::Rng;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use std::fs::File;
use std::io::BufReader;

use crate::point3d::Point3D;
use crate::ray::HitRecord;
use crate::ray::Ray;

pub trait Scatterable {
    fn scatter(&self, ray: &Ray, hit_record: &HitRecord) -> Option<(Option<Ray>, Srgb)>;
}

// https://docs.rs/serde_with/1.9.4/serde_with/macro.serde_conv.html
serde_with::serde_conv!(
    SrgbAsArray,
    Srgb,
    |srgb: &Srgb| [srgb.red, srgb.green, srgb.blue],
    |value: [f32; 3]| -> Result<_, std::convert::Infallible> {
        Ok(Srgb::new(value[0], value[1], value[2]))
    }
);

// TODO: replace this with the more elegant implementation in config.rs
serde_with::serde_conv!(
    TexturePixelsAsPath,
    Vec<u8>,
    |_pixels: &Vec<u8>| "/tmp/texture.jpg",
    |value: &str| -> Result<_, std::convert::Infallible> { Ok(load_texture_image(value).0) }
);

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum Material {
    Lambertian(Lambertian),
    Metal(Metal),
    Glass(Glass),
    Texture(Texture),
    Light(Light),
}

impl Scatterable for Material {
    fn scatter(&self, ray: &Ray, hit_record: &HitRecord) -> Option<(Option<Ray>, Srgb)> {
        match self {
            Material::Lambertian(l) => l.scatter(ray, hit_record),
            Material::Metal(m) => m.scatter(ray, hit_record),
            Material::Glass(g) => g.scatter(ray, hit_record),
            Material::Texture(t) => t.scatter(ray, hit_record),
            Material::Light(l) => l.scatter(ray, hit_record),
        }
    }
}

#[serde_with::serde_as]
#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
pub struct Light {
    #[serde_as(as = "SrgbAsArray")]
    pub color: Srgb,
    pub strength: f32,
    pub beams: i64,
    pub frequency: i64,
}

impl Light {
    pub fn new(color: Srgb, strength: f32, beams: i64, frequency: i64) -> Light {
        Light {
            color,
            strength,
            beams,
            frequency,
        }
    }
}

impl Scatterable for Light {
    fn scatter(&self, _ray: &Ray, _hit_record: &HitRecord) -> Option<(Option<Ray>, Srgb)> {
        Some((None, self.color))
    }
}

#[serde_with::serde_as]
#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
pub struct Lambertian {
    #[serde_as(as = "SrgbAsArray")]
    pub albedo: Srgb,
}

impl Lambertian {
    pub fn new(albedo: Srgb) -> Lambertian {
        Lambertian { albedo }
    }
}

impl Scatterable for Lambertian {
    fn scatter(&self, ray: &Ray, hit_record: &HitRecord) -> Option<(Option<Ray>, Srgb)> {
        let scatter_direction = Point3D::random_in_hemi_sphere(&hit_record.normal);
        let target = hit_record.point + scatter_direction;
        let scattered = Ray::new(
            hit_record.point,
            target - hit_record.point,
            ray.strength,
            hit_record.t,
            ray.frequency,
        );
        let attenuation = self.albedo;
        Some((Some(scattered), attenuation))
    }
}

#[serde_with::serde_as]
#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
pub struct Metal {
    #[serde_as(as = "SrgbAsArray")]
    pub albedo: Srgb,
    pub fuzz: f32,
    pub dampening: f32,
}

impl Metal {
    pub fn new(albedo: Srgb, fuzz: f32, dampening: f32) -> Metal {
        Metal {
            albedo,
            fuzz,
            dampening,
        }
    }
}

fn reflect(v: &Point3D, n: &Point3D) -> Point3D {
    *v - *n * (2.0 * v.dot(n))
}

impl Scatterable for Metal {
    fn scatter(&self, ray: &Ray, hit_record: &HitRecord) -> Option<(Option<Ray>, Srgb)> {
        let reflected = reflect(&ray.direction, &hit_record.normal);
        //println!("Reflecting with strength {}", ray.strength_at(hit_record.t));
        let scattered = Ray::new(
            hit_record.point,
            reflected + Point3D::random_in_unit_sphere() * self.fuzz,
            ray.strength - self.dampening,
            hit_record.t,
            ray.frequency,
        );
        let attenuation = self.albedo;
        if scattered.direction.dot(&hit_record.normal) > 0.0 {
            Some((Some(scattered), attenuation))
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
pub struct Glass {
    pub index_of_refraction: f32,
}

impl Glass {
    pub fn new(index_of_refraction: f32) -> Glass {
        Glass {
            index_of_refraction,
        }
    }
}

fn refract(uv: &Point3D, n: &Point3D, etai_over_etat: f32) -> Point3D {
    let cos_theta = ((-*uv).dot(n)).min(1.0);
    let r_out_perp = (*uv + *n * cos_theta) * etai_over_etat;
    let r_out_parallel = *n * (-1.0 * (1.0 - r_out_perp.length_squared()).abs().sqrt());
    r_out_perp + r_out_parallel
}

fn reflectance(cosine: f32, ref_idx: f32) -> f32 {
    let mut r0 = (1.0 - ref_idx) / (1.0 + ref_idx);
    r0 = r0 * r0;
    r0 + (1.0 - r0) * (1.0 - cosine).powi(5)
}

#[test]
fn test_refract() {
    let uv = Point3D::new(1.0, 1.0, 0.0);
    let n = Point3D::new(-1.0, 0.0, 0.0);
    let etai_over_etat = 1.0;
    let expected = Point3D::new(0.0, 1.0, 0.0);
    let actual = refract(&uv, &n, etai_over_etat);
    assert_eq!(actual, expected);
}

#[test]
fn test_reflectance() {
    let cosine = 0.0;
    let ref_idx = 1.5;
    let expected = 1.0;
    let actual = reflectance(cosine, ref_idx);
    assert_eq!(actual, expected);
}

impl Scatterable for Glass {
    fn scatter(&self, ray: &Ray, hit_record: &HitRecord) -> Option<(Option<Ray>, Srgb)> {
        let mut rng = rand::thread_rng();
        let attenuation = Srgb::new(1.0f32, 1.0f32, 1.0f32);
        let refraction_ratio = if hit_record.front_face {
            1.0 / self.index_of_refraction
        } else {
            self.index_of_refraction
        };
        let unit_direction = ray.direction.unit_vector();
        let cos_theta = (-unit_direction).dot(&hit_record.normal).min(1.0);
        let sin_theta = (1.0 - cos_theta * cos_theta).sqrt();
        let cannot_refract = refraction_ratio * sin_theta > 1.0;
        if cannot_refract || reflectance(cos_theta, refraction_ratio) > rng.gen::<f32>() {
            let reflected = reflect(&unit_direction, &hit_record.normal);
            let scattered = Ray::new(
                hit_record.point,
                reflected,
                ray.strength,
                hit_record.t,
                ray.frequency,
            );
            Some((Some(scattered), attenuation))
        } else {
            let direction = refract(&unit_direction, &hit_record.normal, refraction_ratio);
            let scattered = Ray::new(
                hit_record.point,
                direction,
                ray.strength,
                hit_record.t,
                ray.frequency,
            );
            Some((Some(scattered), attenuation))
        }
    }
}

#[serde_with::serde_as]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Texture {
    #[serde_as(as = "SrgbAsArray")]
    pub albedo: Srgb,
    #[serde_as(as = "TexturePixelsAsPath")]
    pub pixels: Vec<u8>,
    width: u64,
    height: u64,
    h_offset: f32,
}

fn load_texture_image(path: &str) -> (Vec<u8>, u64, u64) {
    let file = File::open(path).expect(path);
    let mut decoder = Decoder::new(BufReader::new(file));
    let pixels = decoder.decode().expect("failed to decode image");
    let metadata = decoder.info().unwrap();
    (pixels, metadata.width as u64, metadata.height as u64)
}

impl Texture {
    pub fn new(albedo: Srgb, texture_path: &str, rot: f32) -> Texture {
        let file = File::open(texture_path).expect("failed to open texture file");
        let mut decoder = Decoder::new(BufReader::new(file));
        let pixels = decoder.decode().expect("failed to decode image");
        let metadata = decoder.info().unwrap();
        Texture {
            albedo,
            pixels,
            width: metadata.width as u64,
            height: metadata.height as u64,
            h_offset: rot,
        }
    }

    pub fn get_albedo(&self, u: f32, v: f32) -> Srgb {
        let mut rot = u + self.h_offset;
        if rot > 1.0 {
            rot -= 1.0;
        }
        let uu = rot * (self.width) as f32;
        let vv = (1.0 - v) * (self.height - 1) as f32;
        let base_pixel =
            (3 * ((vv.floor() as u64) * self.width + (uu.floor() as u64))) as usize;
        let pixel_r = self.pixels[base_pixel];
        let pixel_g = self.pixels[base_pixel + 1];
        let pixel_b = self.pixels[base_pixel + 2];
        Srgb::new(
            pixel_r as f32 / 255.0,
            pixel_g as f32 / 255.0,
            pixel_b as f32 / 255.0,
        )
    }
}

impl Scatterable for Texture {
    fn scatter(&self, ray: &Ray, hit_record: &HitRecord) -> Option<(Option<Ray>, Srgb)> {
        let mut scatter_direction = hit_record.normal + Point3D::random_in_unit_sphere();
        if scatter_direction.near_zero() {
            scatter_direction = hit_record.normal;
        }
        let target = hit_record.point + scatter_direction;
        let scattered = Ray::new(
            hit_record.point,
            target - hit_record.point,
            ray.strength,
            hit_record.t,
            ray.frequency,
        );
        let attenuation = self.get_albedo(hit_record.u, hit_record.v);
        Some((Some(scattered), attenuation))
    }
}

