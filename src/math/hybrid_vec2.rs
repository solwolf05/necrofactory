use std::{
    fmt::Display,
    ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign},
};

use bevy::{
    math::{I64Vec2, IVec2, Vec2},
    reflect::Reflect,
};

use crate::math::hybrid::Hybrid;

#[derive(Debug, Default, Clone, Copy, PartialEq, PartialOrd, Reflect)]
pub struct HybridVec2 {
    pub x: Hybrid,
    pub y: Hybrid,
}

impl HybridVec2 {
    pub const ZERO: Self = Self::splat(Hybrid::ZERO);

    pub const X: Self = Self::new(Hybrid::ONE, Hybrid::ZERO);

    pub const Y: Self = Self::new(Hybrid::ZERO, Hybrid::ONE);

    #[inline]
    pub const fn new(x: Hybrid, y: Hybrid) -> Self {
        Self { x, y }
    }

    #[inline]
    pub const fn splat(v: Hybrid) -> Self {
        Self { x: v, y: v }
    }

    pub fn from_chunk_tile(chunk: IVec2, tile: Vec2) -> Self {
        Self {
            x: Hybrid::new(chunk.x, tile.x),
            y: Hybrid::new(chunk.y, tile.y),
        }
    }

    pub fn from_chunk(chunk: IVec2) -> Self {
        Self {
            x: Hybrid::from_chunk(chunk.x),
            y: Hybrid::from_chunk(chunk.y),
        }
    }

    pub fn from_tile(tile: Vec2) -> Self {
        Self {
            x: Hybrid::from_tile(tile.x),
            y: Hybrid::from_tile(tile.y),
        }
    }

    pub fn with_x(self, x: Hybrid) -> Self {
        Self { x, y: self.y }
    }

    pub fn with_y(self, y: Hybrid) -> Self {
        Self { x: self.x, y: y }
    }

    pub fn max_element(self) -> Hybrid {
        self.x.max(self.y)
    }

    pub fn chunk(&self) -> IVec2 {
        IVec2::new(self.x.chunk, self.y.chunk)
    }

    pub fn tile(&self) -> Vec2 {
        Vec2::new(self.x.tile, self.y.tile)
    }

    pub fn clamp(self, min: Hybrid, max: Hybrid) -> Self {
        Self {
            x: self.x.clamp(min, max),
            y: self.y.clamp(min, max),
        }
    }

    pub fn round(self) -> Self {
        Self {
            x: self.x.round(),
            y: self.y.round(),
        }
    }

    // pub fn trunc(self) -> Self {
    //     Self {
    //         x: self.x.trunc(),
    //         y: self.y.trunc(),
    //     }
    // }

    pub fn floor(self) -> Self {
        Self {
            x: self.x.floor(),
            y: self.y.floor(),
        }
    }

    pub fn ceil(self) -> Self {
        Self {
            x: self.x.ceil(),
            y: self.y.ceil(),
        }
    }

    pub fn abs(self) -> Self {
        Self {
            x: self.x.abs(),
            y: self.y.abs(),
        }
    }

    pub fn to_vec2(self) -> Vec2 {
        Vec2::new(self.x.into(), self.y.into())
    }

    pub fn length_squared(self) -> f32 {
        let x = self.x.to_f32();
        let y = self.y.to_f32();
        x * x + y * y
    }

    pub fn length(self) -> f32 {
        self.length_squared().sqrt()
    }

    pub fn distance(self, other: Self) -> f32 {
        (self - other).length()
    }

    pub fn distance_squared(self, other: Self) -> f32 {
        (self - other).length_squared()
    }
}

impl Display for HybridVec2 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.to_vec2().fmt(f)
    }
}

impl From<Vec2> for HybridVec2 {
    fn from(value: Vec2) -> Self {
        Self {
            x: value.x.into(),
            y: value.y.into(),
        }
    }
}

impl From<HybridVec2> for Vec2 {
    fn from(value: HybridVec2) -> Self {
        Self {
            x: value.x.into(),
            y: value.y.into(),
        }
    }
}

impl From<IVec2> for HybridVec2 {
    fn from(value: IVec2) -> Self {
        Self {
            x: value.x.into(),
            y: value.y.into(),
        }
    }
}

impl From<HybridVec2> for IVec2 {
    fn from(value: HybridVec2) -> Self {
        Self {
            x: value.x.into(),
            y: value.y.into(),
        }
    }
}

impl From<I64Vec2> for HybridVec2 {
    fn from(value: I64Vec2) -> Self {
        Self {
            x: value.x.into(),
            y: value.y.into(),
        }
    }
}

impl From<HybridVec2> for I64Vec2 {
    fn from(value: HybridVec2) -> Self {
        Self {
            x: value.x.into(),
            y: value.y.into(),
        }
    }
}

// add

impl Add<HybridVec2> for HybridVec2 {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Self::new(self.x + rhs.x, self.y + rhs.y)
    }
}

impl Add<f32> for HybridVec2 {
    type Output = Self;
    fn add(self, rhs: f32) -> Self {
        Self::new(self.x + rhs, self.y + rhs)
    }
}

impl Add<HybridVec2> for f32 {
    type Output = HybridVec2;
    fn add(self, rhs: HybridVec2) -> HybridVec2 {
        rhs + self
    }
}

impl Add<Vec2> for HybridVec2 {
    type Output = Self;
    fn add(self, rhs: Vec2) -> Self {
        Self::new(self.x + rhs.x, self.y + rhs.y)
    }
}

impl Add<HybridVec2> for Vec2 {
    type Output = HybridVec2;
    fn add(self, rhs: HybridVec2) -> HybridVec2 {
        rhs + self
    }
}

impl Add<IVec2> for HybridVec2 {
    type Output = Self;
    fn add(self, rhs: IVec2) -> Self {
        Self::new(self.x + rhs.x, self.y + rhs.y)
    }
}

impl AddAssign<HybridVec2> for HybridVec2 {
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

impl AddAssign<f32> for HybridVec2 {
    fn add_assign(&mut self, rhs: f32) {
        self.x += rhs;
        self.y += rhs;
    }
}

// sub

impl Sub<HybridVec2> for HybridVec2 {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        Self::new(self.x - rhs.x, self.y - rhs.y)
    }
}

impl Sub<f32> for HybridVec2 {
    type Output = Self;
    fn sub(self, rhs: f32) -> Self {
        Self::new(self.x - rhs, self.y - rhs)
    }
}

impl Sub<HybridVec2> for f32 {
    type Output = HybridVec2;
    fn sub(self, rhs: HybridVec2) -> HybridVec2 {
        HybridVec2::new(Hybrid::from(self) - rhs.x, Hybrid::from(self) - rhs.y)
    }
}

impl Sub<Vec2> for HybridVec2 {
    type Output = Self;
    fn sub(self, rhs: Vec2) -> Self {
        Self::new(self.x - rhs.x, self.y - rhs.y)
    }
}

impl Sub<IVec2> for HybridVec2 {
    type Output = Self;
    fn sub(self, rhs: IVec2) -> Self {
        Self::new(self.x - rhs.x, self.y - rhs.y)
    }
}

impl SubAssign<HybridVec2> for HybridVec2 {
    fn sub_assign(&mut self, rhs: Self) {
        self.x -= rhs.x;
        self.y -= rhs.y;
    }
}

impl SubAssign<f32> for HybridVec2 {
    fn sub_assign(&mut self, rhs: f32) {
        self.x -= rhs;
        self.y -= rhs;
    }
}

// mul

impl Mul<HybridVec2> for HybridVec2 {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self {
        Self::new(self.x * rhs.x, self.y * rhs.y)
    }
}

impl Mul<f32> for HybridVec2 {
    type Output = Self;
    fn mul(self, rhs: f32) -> Self {
        Self::new(self.x * rhs, self.y * rhs)
    }
}

impl Mul<HybridVec2> for f32 {
    type Output = HybridVec2;
    fn mul(self, rhs: HybridVec2) -> HybridVec2 {
        rhs * self
    }
}

impl Mul<i32> for HybridVec2 {
    type Output = Self;
    fn mul(self, rhs: i32) -> Self {
        Self::new(self.x * rhs, self.y * rhs)
    }
}

impl Mul<HybridVec2> for i32 {
    type Output = HybridVec2;
    fn mul(self, rhs: HybridVec2) -> HybridVec2 {
        rhs * self
    }
}

impl Mul<Vec2> for HybridVec2 {
    type Output = Self;
    fn mul(self, rhs: Vec2) -> Self {
        Self::new(self.x * rhs.x, self.y * rhs.y)
    }
}

impl MulAssign<f32> for HybridVec2 {
    fn mul_assign(&mut self, rhs: f32) {
        self.x *= rhs;
        self.y *= rhs;
    }
}

// div

impl Div<HybridVec2> for HybridVec2 {
    type Output = Self;
    fn div(self, rhs: Self) -> Self {
        Self::new(self.x / rhs.x, self.y / rhs.y)
    }
}

impl Div<f32> for HybridVec2 {
    type Output = Self;
    fn div(self, rhs: f32) -> Self {
        Self::new(self.x / rhs, self.y / rhs)
    }
}

impl Div<HybridVec2> for f32 {
    type Output = HybridVec2;
    fn div(self, rhs: HybridVec2) -> HybridVec2 {
        let val = Hybrid::from(self);
        HybridVec2::new(val / rhs.x, val / rhs.y)
    }
}

impl Div<i32> for HybridVec2 {
    type Output = Self;
    fn div(self, rhs: i32) -> Self {
        Self::new(self.x / rhs, self.y / rhs)
    }
}

impl Div<Vec2> for HybridVec2 {
    type Output = Self;
    fn div(self, rhs: Vec2) -> Self {
        Self::new(self.x / rhs.x, self.y / rhs.y)
    }
}

impl DivAssign<f32> for HybridVec2 {
    fn div_assign(&mut self, rhs: f32) {
        self.x /= rhs;
        self.y /= rhs;
    }
}

// --- ADDITION (HybridVec2 <-> Vec2) ---

impl AddAssign<Vec2> for HybridVec2 {
    fn add_assign(&mut self, rhs: Vec2) {
        *self = *self + rhs;
    }
}

// --- SUBTRACTION (HybridVec2 <-> Vec2) ---

impl Sub<HybridVec2> for Vec2 {
    type Output = HybridVec2;
    fn sub(self, rhs: HybridVec2) -> HybridVec2 {
        HybridVec2::from(self) - rhs
    }
}

impl SubAssign<Vec2> for HybridVec2 {
    fn sub_assign(&mut self, rhs: Vec2) {
        *self = *self - rhs;
    }
}

// --- MULTIPLICATION (HybridVec2 <-> Vec2) ---

impl Mul<HybridVec2> for Vec2 {
    type Output = HybridVec2;
    fn mul(self, rhs: HybridVec2) -> HybridVec2 {
        rhs * self
    }
}

impl MulAssign<Vec2> for HybridVec2 {
    fn mul_assign(&mut self, rhs: Vec2) {
        *self = *self * rhs;
    }
}

// --- DIVISION (HybridVec2 <-> Vec2) ---

impl Div<HybridVec2> for Vec2 {
    type Output = HybridVec2;
    fn div(self, rhs: HybridVec2) -> HybridVec2 {
        HybridVec2::from(self) / rhs
    }
}

impl DivAssign<Vec2> for HybridVec2 {
    fn div_assign(&mut self, rhs: Vec2) {
        *self = *self / rhs;
    }
}
