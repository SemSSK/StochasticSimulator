use std::{
    hash::Hash,
    ops::{Add, Mul, Neg, Sub},
};

use glam::{IVec3, Vec3A};
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Vector3d {
    pub data: Vec3A,
}

#[derive(Debug, PartialEq, Hash, Eq, Clone, Copy)]
pub struct VectorInt3d {
    pub data: IVec3,
}

impl Add for Vector3d {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            data: self.data + rhs.data,
        }
    }
}

impl Mul<Vector3d> for f32 {
    type Output = Vector3d;

    fn mul(self, rhs: Vector3d) -> Self::Output {
        Self::Output {
            data: self * rhs.data,
        }
    }
}
impl Mul<f32> for Vector3d {
    type Output = Vector3d;

    fn mul(self, rhs: f32) -> Self::Output {
        Self::Output {
            data: rhs * self.data,
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
        let mut data = Vec3A::new(
            fastrand::f32() - 0.5,
            fastrand::f32() - 0.5,
            fastrand::f32() - 0.5,
        ) * 100.;
        data = data.normalize();
        Self { data }
    }

    pub fn dot(&self, other: &Self) -> f32 {
        self.data.dot(other.data)
    }

    pub fn distance(&self, other: &Self) -> f32 {
        self.data.distance(other.data)
    }

    pub fn distance_pow2(&self, other: &Self) -> f32 {
        let vec = *self - *other;
        vec.data.distance_squared(vec.data)
    }

    pub fn into_vectorint(self) -> VectorInt3d {
        let data = (self.data / 10.).as_ivec3();
        VectorInt3d { data }
    }
}

pub const VECTOR_ZERO: Vector3d = Vector3d { data: Vec3A::ZERO };

pub fn generate_random_position() -> Vector3d {
    let vec = Vector3d::get_random_unitary();
    let d = fastrand::f32() * 400.0;
    let random_pos = d * vec;
    assert!(random_pos.distance(&VECTOR_ZERO) <= 500.);
    random_pos
}
