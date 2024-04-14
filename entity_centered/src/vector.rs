use std::{
    hash::Hash,
    ops::{Add, Mul, Neg, Sub},
};

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Vector3d {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[derive(Debug, PartialEq, Hash, Eq, Clone, Copy)]
pub struct VectorInt3d {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl Hash for Vector3d {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        let (x, y, z) = (
            (self.x / 10.).floor() as i32,
            (self.y / 10.).floor() as i32,
            (self.z / 10.).floor() as i32,
        );
        x.hash(state);
        y.hash(state);
        z.hash(state);
    }
}

impl Add for Vector3d {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}

impl Mul<Vector3d> for f32 {
    type Output = Vector3d;

    fn mul(self, rhs: Vector3d) -> Self::Output {
        Self::Output {
            x: rhs.x * self,
            y: rhs.y * self,
            z: rhs.z * self,
        }
    }
}
impl Mul<f32> for Vector3d {
    type Output = Vector3d;

    fn mul(self, rhs: f32) -> Self::Output {
        Self::Output {
            x: self.x * rhs,
            y: self.y * rhs,
            z: self.z * rhs,
        }
    }
}

impl Neg for Vector3d {
    type Output = Vector3d;

    fn neg(self) -> Self::Output {
        -1. * self
    }
}

impl Sub for Vector3d {
    type Output = Vector3d;

    fn sub(self, rhs: Self) -> Self::Output {
        self + (-rhs)
    }
}

impl Vector3d {
    pub fn get_random_unitary() -> Self {
        let x = fastrand::f32() - 0.5;
        let y = fastrand::f32() - 0.5;
        let z = fastrand::f32() - 0.5;
        let vec = Self { x, y, z };
        let length = vec.dot(&vec).sqrt();
        1. / length * Self { x, y, z }
    }

    pub fn dot(&self, other: &Self) -> f32 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    pub fn distance(&self, other: &Self) -> f32 {
        self.distance_pow2(other).sqrt()
    }

    pub fn distance_pow2(&self, other: &Self) -> f32 {
        let vec = *self - *other;
        vec.dot(&vec)
    }

    pub fn into_vectorint(self) -> VectorInt3d {
        VectorInt3d {
            x: self.x.floor() as i32,
            y: self.y.floor() as i32,
            z: self.z.floor() as i32,
        }
    }
}

pub const VECTOR_ZERO: Vector3d = Vector3d {
    x: 0.,
    y: 0.,
    z: 0.,
};

pub fn generate_random_position() -> Vector3d {
    let vec = Vector3d::get_random_unitary();
    let d = fastrand::f32() * 900.0 - 450.0;
    let random_pos = d * vec;
    assert!(random_pos.distance(&VECTOR_ZERO) <= 500.);
    random_pos
}
