use rand::Rng;
use serde::{Deserialize, Serialize};
use std::cmp::PartialEq;
use std::f32::consts;
use std::ops::{Add, Div, Mul, Neg, Sub};
use rand_distr::StandardNormal;

#[cfg(test)]
use assert_approx_eq::assert_approx_eq;

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
pub struct Point3D {
    x: f32,
    y: f32,
    z: f32,
}
const BEAM_FALLOFF: f32 = 80.0;
const SPREAD_FACTOR: f32 = 4.0;
impl Point3D {
    pub const fn new(x: f32, y: f32, z: f32) -> Point3D {
        Point3D { x, y, z }
    }

    pub fn random(min: f32, max: f32) -> Point3D {
        let mut rng = rand::thread_rng();
        Point3D::new(
            rng.gen_range(min..max),
            rng.gen_range(min..max),
            rng.gen_range(min..max),
        )
    }

    pub fn normal_in_beam(total_beams: i64, beam: i64, base_strength: f32) -> (Point3D, f32) {
        let mut rng = rand::thread_rng();
        let value = rng.gen_range(-0.5..0.5);
        let phi = ((beam as f32 +1.0 + value) / (total_beams as f32)) *2.0 * consts::PI;
        //println!("Point at angle {}={}, {} ({} / {})",phi,phi/(2.0*std::f32::consts::PI)* 360.0,value,beam,total_beams);
        let psi_choice:f32 = rng.sample(StandardNormal);
        let psi = consts::PI/2.0 *(1.0 + (psi_choice.clamp(-SPREAD_FACTOR,SPREAD_FACTOR)/SPREAD_FACTOR).abs());
        //let psi = rng.gen_range(consts::PI/2.0..consts::PI);//aimed down
        (
            Point3D::new(phi.sin() * psi.sin(), psi.cos(), phi.cos() * psi.sin()),
            (base_strength - BEAM_FALLOFF) + BEAM_FALLOFF * (1.0 - (2.0 * value.powi(2).abs())),
        )
    }

    pub fn random_in_unit_sphere() -> Point3D {
        let mut rng = rand::thread_rng();
        let phi = rng.gen_range(-consts::PI..consts::PI);
        let psi = rng.gen_range(-consts::PI..consts::PI);
        Point3D::new(phi.sin() * psi.cos(), psi.sin(), phi.cos() * psi.cos())
    }
    pub fn random_in_hemi_sphere(dir: &Point3D) -> Point3D {
        let new = Point3D::random_in_unit_sphere();
        if new.distance(&dir) < consts::SQRT_2 {
            return new;
        } else {
            return -new;
        }
    }

    pub fn x(&self) -> f32 {
        self.x
    }

    pub fn y(&self) -> f32 {
        self.y
    }

    pub fn z(&self) -> f32 {
        self.z
    }

    pub fn distance(&self, other: &Point3D) -> f32 {
        let dx = self.x - other.x();
        let dy = self.y - other.y();
        let dz = self.z - other.z();
        (dx * dx + dy * dy + dz * dz).sqrt()
    }

    pub fn length_squared(&self) -> f32 {
        self.x * self.x + self.y * self.y + self.z * self.z
    }

    pub fn length(&self) -> f32 {
        self.distance(&Point3D::new(0.0, 0.0, 0.0))
    }

    pub fn unit_vector(&self) -> Point3D {
        let length = self.length();
        Point3D::new(self.x / length, self.y / length, self.z / length)
    }

    pub fn dot(&self, other: &Point3D) -> f32 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    pub fn cross(&self, other: &Point3D) -> Point3D {
        Point3D::new(
            self.y * other.z - self.z * other.y,
            self.z * other.x - self.x * other.z,
            self.x * other.y - self.y * other.x,
        )
    }

    pub fn near_zero(&self) -> bool {
        self.x.abs() < f32::EPSILON && self.y.abs() < f32::EPSILON && self.z.abs() < f32::EPSILON
    }
}

impl Add for Point3D {
    type Output = Point3D;

    fn add(self, other: Point3D) -> Point3D {
        Point3D {
            x: self.x + other.x(),
            y: self.y + other.y(),
            z: self.z + other.z(),
        }
    }
}

impl Sub for Point3D {
    type Output = Point3D;

    fn sub(self, other: Point3D) -> Point3D {
        Point3D {
            x: self.x - other.x(),
            y: self.y - other.y(),
            z: self.z - other.z(),
        }
    }
}

impl Neg for Point3D {
    type Output = Point3D;

    fn neg(self) -> Point3D {
        Point3D {
            x: -self.x,
            y: -self.y,
            z: -self.z,
        }
    }
}

impl Mul<Point3D> for Point3D {
    type Output = Point3D;

    fn mul(self, other: Point3D) -> Point3D {
        Point3D {
            x: self.x * other.x(),
            y: self.y * other.y(),
            z: self.z * other.z(),
        }
    }
}

impl Mul<f32> for Point3D {
    type Output = Point3D;

    fn mul(self, other: f32) -> Point3D {
        Point3D {
            x: self.x * other,
            y: self.y * other,
            z: self.z * other,
        }
    }
}

impl Div<Point3D> for Point3D {
    type Output = Point3D;

    fn div(self, other: Point3D) -> Point3D {
        Point3D {
            x: self.x / other.x(),
            y: self.y / other.y(),
            z: self.z / other.z(),
        }
    }
}

impl Div<f32> for Point3D {
    type Output = Point3D;

    fn div(self, other: f32) -> Point3D {
        Point3D {
            x: self.x / other,
            y: self.y / other,
            z: self.z / other,
        }
    }
}

impl PartialEq for Point3D {
    fn eq(&self, other: &Point3D) -> bool {
        self.x == other.x() && self.y == other.y() && self.z == other.z()
    }
}

#[test]
fn test_gen() {
    let p = Point3D {
        x: 0.1,
        y: 0.2,
        z: 0.3,
    };
    assert_eq!(p.x(), 0.1);
    assert_eq!(p.y(), 0.2);
    assert_eq!(p.z(), 0.3);

    let q = Point3D::new(0.2, 0.3, 0.4);
    assert_eq!(q.x(), 0.2);
    assert_eq!(q.y(), 0.3);
    assert_eq!(q.z(), 0.4);
}

#[test]
fn test_add() {
    let p = Point3D::new(0.1, 0.2, 0.3);
    let q = Point3D::new(0.2, 0.3, 0.4);
    let r = p + q;
    assert_approx_eq!(r.x(), 0.3);
    assert_approx_eq!(r.y(), 0.5);
    assert_approx_eq!(r.z(), 0.7);
}

#[test]
fn test_sub() {
    let p = Point3D::new(0.1, 0.2, 0.3);
    let q = Point3D::new(0.2, 0.3, 0.4);
    let r = p - q;
    assert_approx_eq!(r.x(), -0.1);
    assert_approx_eq!(r.y(), -0.1);
    assert_approx_eq!(r.z(), -0.1);
}

#[test]
fn test_neg() {
    let p = Point3D::new(0.1, 0.2, 0.3);
    let q = -p;
    assert_approx_eq!(q.x(), -0.1);
    assert_approx_eq!(q.y(), -0.2);
    assert_approx_eq!(q.z(), -0.3);
}

#[test]
fn test_mul() {
    let p = Point3D::new(0.1, 0.2, 0.3);
    let q = Point3D::new(0.2, 0.3, 0.4);
    let r = p * q;
    assert_approx_eq!(r.x(), 0.02);
    assert_approx_eq!(r.y(), 0.06);
    assert_approx_eq!(r.z(), 0.12);
}

#[test]
fn test_div() {
    let p = Point3D::new(0.1, 0.2, 0.3);
    let q = Point3D::new(0.2, 0.3, 0.4);
    let r = p / q;
    assert_approx_eq!(r.x(), 0.5);
    assert_approx_eq!(r.y(), 0.6666666666666666);
    assert_approx_eq!(r.z(), 0.3 / 0.4);
}

#[test]
fn test_dot() {
    let p = Point3D::new(0.1, 0.2, 0.3);
    let q = Point3D::new(0.2, 0.3, 0.4);
    assert_approx_eq!(p.dot(&q), 0.2);
}

#[test]
fn test_length_squared() {
    let p = Point3D::new(0.1, 0.2, 0.3);
    assert_approx_eq!(p.length_squared(), 0.14);
}

#[test]
fn test_random() {
    let p = Point3D::random(-1.0, 1.0);
    assert!(p.x() >= -1.0 && p.x() <= 1.0);
    assert!(p.y() >= -1.0 && p.y() <= 1.0);
    assert!(p.z() >= -1.0 && p.z() <= 1.0);
}

#[test]
fn test_near_zero() {
    let p = Point3D::new(0.1, 0.2, 0.3);
    assert!(!p.near_zero());
    let p = Point3D::new(0.0, 0.0, 0.0);
    assert!(p.near_zero());
}
