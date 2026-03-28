use std::{
    fmt::Display,
    ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign},
};

use bevy::reflect::Reflect;

use crate::world::{CHUNK_SIZE, TILE_SIZE};

#[derive(Debug, Default, Clone, Copy, PartialEq, PartialOrd, Reflect)]
pub struct Hybrid {
    pub chunk: i32,
    pub tile: f32,
}

impl Hybrid {
    pub const ZERO: Self = Self {
        chunk: 0,
        tile: 0.0,
    };

    pub const ONE: Self = Self {
        chunk: 0,
        tile: 1.0,
    };

    pub const NEG_ONE: Self = Self {
        chunk: 0,
        tile: -1.0,
    };

    pub fn new(chunk: i32, tile: f32) -> Self {
        Self { chunk, tile }.normalize()
    }

    pub fn from_chunk(chunk: i32) -> Self {
        Self { chunk, tile: 0.0 }
    }

    pub fn from_tile(tile: f32) -> Self {
        Self { chunk: 0, tile }.normalize()
    }

    #[inline]
    fn normalize(self) -> Self {
        let chunk_offset = self.tile.div_euclid(TILE_SIZE as f32) as i32;
        let tile = self.tile.rem_euclid(TILE_SIZE as f32);
        Self {
            chunk: self.chunk + chunk_offset,
            tile,
        }
    }

    pub fn to_f32(self) -> f32 {
        self.into()
    }

    pub fn min(self, other: Self) -> Self {
        if self <= other { self } else { other }
    }

    pub fn max(self, other: Self) -> Self {
        if self >= other { self } else { other }
    }

    pub fn is_positive(&self) -> bool {
        self.chunk >= 0 && self.tile > 0.0
    }

    pub fn is_negative(&self) -> bool {
        self.chunk <= 0
    }

    pub fn signum(&self) -> f32 {
        if self.is_positive() {
            1.0
        } else if self.is_negative() {
            -1.0
        } else {
            0.0
        }
    }

    pub fn clamp(self, min: Self, max: Self) -> Self {
        self.max(min).min(max)
    }

    pub fn round(self) -> Self {
        Self::new(self.chunk, self.tile.round())
    }

    pub fn floor(self) -> Self {
        Self::new(self.chunk, self.tile.floor())
    }

    pub fn ceil(self) -> Self {
        Self::new(self.chunk, self.tile.ceil())
    }

    pub fn abs(self) -> Self {
        if self.chunk < 0 || (self.chunk == 0 && self.tile < 0.0) {
            -self
        } else {
            self
        }
    }
}

impl Display for Hybrid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f32::from(*self).fmt(f)
    }
}

impl From<f32> for Hybrid {
    fn from(value: f32) -> Self {
        Self::from_tile(value)
    }
}

impl From<Hybrid> for f32 {
    fn from(value: Hybrid) -> Self {
        (value.chunk * CHUNK_SIZE) as f32 + value.tile
    }
}

impl From<i32> for Hybrid {
    fn from(value: i32) -> Self {
        let chunk = value.div_euclid(TILE_SIZE);
        let tile = value.rem_euclid(TILE_SIZE) as f32;
        Self { chunk, tile }
    }
}

impl From<Hybrid> for i32 {
    fn from(value: Hybrid) -> Self {
        (value.chunk * CHUNK_SIZE) + value.tile as i32
    }
}

impl From<i64> for Hybrid {
    fn from(value: i64) -> Self {
        let chunk = value.div_euclid(TILE_SIZE as i64) as i32;
        let tile = value.rem_euclid(TILE_SIZE as i64) as f32;
        Self { chunk, tile }
    }
}

impl From<Hybrid> for i64 {
    fn from(value: Hybrid) -> Self {
        (value.chunk as i64 * CHUNK_SIZE as i64) + value.tile as i64
    }
}

// add

impl Add<Hybrid> for Hybrid {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Self::new(self.chunk + rhs.chunk, self.tile + rhs.tile)
    }
}

impl Add<f32> for Hybrid {
    type Output = Self;
    fn add(self, rhs: f32) -> Self {
        Self::new(self.chunk, self.tile + rhs)
    }
}

impl Add<Hybrid> for f32 {
    type Output = Hybrid;
    fn add(self, rhs: Hybrid) -> Hybrid {
        rhs + self
    }
}

impl Add<i32> for Hybrid {
    type Output = Self;
    fn add(self, rhs: i32) -> Self {
        Self::new(self.chunk, self.tile + rhs as f32)
    }
}

impl Add<Hybrid> for i32 {
    type Output = Hybrid;
    fn add(self, rhs: Hybrid) -> Hybrid {
        rhs + self
    }
}

impl AddAssign<Hybrid> for Hybrid {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl AddAssign<f32> for Hybrid {
    fn add_assign(&mut self, rhs: f32) {
        *self = *self + rhs;
    }
}

impl AddAssign<i32> for Hybrid {
    fn add_assign(&mut self, rhs: i32) {
        *self = *self + rhs;
    }
}

// sub

impl Sub<Hybrid> for Hybrid {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        Self::new(self.chunk - rhs.chunk, self.tile - rhs.tile)
    }
}

impl Sub<f32> for Hybrid {
    type Output = Self;
    fn sub(self, rhs: f32) -> Self {
        Self::new(self.chunk, self.tile - rhs)
    }
}

impl Sub<Hybrid> for f32 {
    type Output = Hybrid;
    fn sub(self, rhs: Hybrid) -> Hybrid {
        Hybrid::from_tile(self) - rhs
    }
}

impl Sub<i32> for Hybrid {
    type Output = Self;
    fn sub(self, rhs: i32) -> Self {
        Self::new(self.chunk, self.tile - rhs as f32)
    }
}

impl Sub<Hybrid> for i32 {
    type Output = Hybrid;
    fn sub(self, rhs: Hybrid) -> Hybrid {
        Hybrid::from(self) - rhs
    }
}

impl SubAssign<Hybrid> for Hybrid {
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs;
    }
}

impl SubAssign<f32> for Hybrid {
    fn sub_assign(&mut self, rhs: f32) {
        *self = *self - rhs;
    }
}

impl SubAssign<i32> for Hybrid {
    fn sub_assign(&mut self, rhs: i32) {
        *self = *self - rhs;
    }
}

// mul

impl Mul<Hybrid> for Hybrid {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self {
        (f32::from(self) * f32::from(rhs)).into()
    }
}

impl Mul<f32> for Hybrid {
    type Output = Self;
    fn mul(self, rhs: f32) -> Self {
        (f32::from(self) * rhs).into()
    }
}

impl Mul<Hybrid> for f32 {
    type Output = Hybrid;
    fn mul(self, rhs: Hybrid) -> Hybrid {
        rhs * self
    }
}

impl Mul<i32> for Hybrid {
    type Output = Self;
    fn mul(self, rhs: i32) -> Self {
        Self::new(self.chunk * rhs, self.tile * rhs as f32)
    }
}

impl Mul<Hybrid> for i32 {
    type Output = Hybrid;
    fn mul(self, rhs: Hybrid) -> Hybrid {
        rhs * self
    }
}

impl MulAssign<Hybrid> for Hybrid {
    fn mul_assign(&mut self, rhs: Self) {
        *self = *self * rhs;
    }
}

impl MulAssign<f32> for Hybrid {
    fn mul_assign(&mut self, rhs: f32) {
        *self = *self * rhs;
    }
}

impl MulAssign<i32> for Hybrid {
    fn mul_assign(&mut self, rhs: i32) {
        *self = *self * rhs;
    }
}

// div

impl Div<Hybrid> for Hybrid {
    type Output = Self;
    fn div(self, rhs: Self) -> Self {
        (f32::from(self) / f32::from(rhs)).into()
    }
}

impl Div<f32> for Hybrid {
    type Output = Self;
    fn div(self, rhs: f32) -> Self {
        (f32::from(self) / rhs).into()
    }
}

impl Div<Hybrid> for f32 {
    type Output = Hybrid;
    fn div(self, rhs: Hybrid) -> Hybrid {
        (self / f32::from(rhs)).into()
    }
}

impl Div<i32> for Hybrid {
    type Output = Self;
    fn div(self, rhs: i32) -> Self {
        Self::new(self.chunk / rhs, self.tile / rhs as f32)
    }
}

impl DivAssign<Hybrid> for Hybrid {
    fn div_assign(&mut self, rhs: Self) {
        *self = *self / rhs;
    }
}

impl DivAssign<f32> for Hybrid {
    fn div_assign(&mut self, rhs: f32) {
        *self = *self / rhs;
    }
}

impl DivAssign<i32> for Hybrid {
    fn div_assign(&mut self, rhs: i32) {
        *self = *self / rhs;
    }
}

// unary and cmp

impl Neg for Hybrid {
    type Output = Self;
    fn neg(self) -> Self::Output {
        Self::new(-self.chunk, -self.tile)
    }
}

impl PartialEq<i32> for Hybrid {
    fn eq(&self, other: &i32) -> bool {
        *self == Hybrid::from(*other)
    }
}

impl PartialEq<f32> for Hybrid {
    fn eq(&self, other: &f32) -> bool {
        *self == Hybrid::from(*other)
    }
}

impl PartialOrd<i32> for Hybrid {
    fn partial_cmp(&self, other: &i32) -> Option<std::cmp::Ordering> {
        self.partial_cmp(&Self::from(*other))
    }
}

impl PartialOrd<f32> for Hybrid {
    fn partial_cmp(&self, other: &f32) -> Option<std::cmp::Ordering> {
        self.partial_cmp(&Self::from(*other))
    }
}
