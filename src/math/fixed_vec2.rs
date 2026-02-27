use std::{
    fmt::Display,
    ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Rem, RemAssign, Sub, SubAssign},
};

use bevy::math::{IVec2, Vec2};

use crate::math::i32f32::I32F32;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FixedVec2 {
    pub x: I32F32,
    pub y: I32F32,
}

impl FixedVec2 {
    pub const fn new(x: I32F32, y: I32F32) -> Self {
        Self { x, y }
    }

    pub const fn splat(value: I32F32) -> Self {
        Self::new(value, value)
    }

    pub fn trunc(self) -> Self {
        Self::new(self.x.trunc(), self.y.trunc())
    }

    pub fn floor(self) -> Self {
        Self::new(self.x.floor(), self.y.floor())
    }

    pub fn ceil(self) -> Self {
        Self::new(self.x.ceil(), self.y.ceil())
    }

    pub fn round(self) -> Self {
        Self::new(self.x.round(), self.y.round())
    }

    pub fn div_euclid(self, rhs: Self) -> Self {
        Self::new(self.x.div_euclid(rhs.x), self.y.div_euclid(rhs.y))
    }

    pub fn rem_euclid(self, rhs: Self) -> Self {
        Self::new(self.x.rem_euclid(rhs.x), self.y.rem_euclid(rhs.y))
    }

    pub fn div_euclid_int(self, rhs: i32) -> Self {
        Self::new(self.x.div_euclid_int(rhs), self.y.div_euclid_int(rhs))
    }

    pub fn rem_euclid_int(self, rhs: i32) -> Self {
        Self::new(self.x.rem_euclid_int(rhs), self.y.rem_euclid_int(rhs))
    }
}

impl From<IVec2> for FixedVec2 {
    fn from(value: IVec2) -> Self {
        Self::new(value.x.into(), value.y.into())
    }
}

impl From<FixedVec2> for IVec2 {
    fn from(value: FixedVec2) -> Self {
        IVec2::new(value.x.into(), value.y.into())
    }
}

impl From<Vec2> for FixedVec2 {
    fn from(value: Vec2) -> Self {
        Self::new(value.x.into(), value.y.into())
    }
}

impl From<FixedVec2> for Vec2 {
    fn from(value: FixedVec2) -> Self {
        Vec2::new(value.x.into(), value.y.into())
    }
}

impl Display for FixedVec2 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(precision) = f.precision() {
            write!(f, "({:.precision$}, {:.precision$})", self.x, self.y)
        } else {
            write!(f, "({}, {})", self.x, self.y)
        }
    }
}

impl Add for FixedVec2 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self::new(self.x + rhs.x, self.y + rhs.y)
    }
}

impl Add<IVec2> for FixedVec2 {
    type Output = Self;

    fn add(self, rhs: IVec2) -> Self::Output {
        Self::new(self.x + rhs.x, self.y + rhs.y)
    }
}

impl Add<Vec2> for FixedVec2 {
    type Output = Self;

    fn add(self, rhs: Vec2) -> Self::Output {
        Self::new(self.x + rhs.x, self.y + rhs.y)
    }
}

impl Add<I32F32> for FixedVec2 {
    type Output = Self;

    fn add(self, rhs: I32F32) -> Self::Output {
        Self::new(self.x + rhs, self.y + rhs)
    }
}

impl Add<i32> for FixedVec2 {
    type Output = Self;

    fn add(self, rhs: i32) -> Self::Output {
        Self::new(self.x + rhs, self.y + rhs)
    }
}

impl Sub for FixedVec2 {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self::new(self.x - rhs.x, self.y - rhs.y)
    }
}

impl Sub<IVec2> for FixedVec2 {
    type Output = Self;

    fn sub(self, rhs: IVec2) -> Self::Output {
        Self::new(self.x - rhs.x, self.y - rhs.y)
    }
}

impl Sub<Vec2> for FixedVec2 {
    type Output = Self;

    fn sub(self, rhs: Vec2) -> Self::Output {
        Self::new(self.x - rhs.x, self.y - rhs.y)
    }
}

impl Sub<I32F32> for FixedVec2 {
    type Output = Self;

    fn sub(self, rhs: I32F32) -> Self::Output {
        Self::new(self.x - rhs, self.y - rhs)
    }
}

impl Sub<i32> for FixedVec2 {
    type Output = Self;

    fn sub(self, rhs: i32) -> Self::Output {
        Self::new(self.x - rhs, self.y - rhs)
    }
}

impl Mul<I32F32> for FixedVec2 {
    type Output = Self;

    fn mul(self, rhs: I32F32) -> Self::Output {
        Self::new(self.x * rhs, self.y * rhs)
    }
}

impl Mul<i32> for FixedVec2 {
    type Output = Self;

    fn mul(self, rhs: i32) -> Self::Output {
        Self::new(self.x * rhs, self.y * rhs)
    }
}

impl Div<I32F32> for FixedVec2 {
    type Output = Self;

    fn div(self, rhs: I32F32) -> Self::Output {
        Self::new(self.x / rhs, self.y / rhs)
    }
}

impl Div<i32> for FixedVec2 {
    type Output = Self;

    fn div(self, rhs: i32) -> Self::Output {
        Self::new(self.x / rhs, self.y / rhs)
    }
}

impl Rem<i32> for FixedVec2 {
    type Output = Self;

    fn rem(self, rhs: i32) -> Self::Output {
        Self::new(self.x % rhs, self.y % rhs)
    }
}

impl AddAssign for FixedVec2 {
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

impl AddAssign<IVec2> for FixedVec2 {
    fn add_assign(&mut self, rhs: IVec2) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

impl AddAssign<Vec2> for FixedVec2 {
    fn add_assign(&mut self, rhs: Vec2) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

impl SubAssign for FixedVec2 {
    fn sub_assign(&mut self, rhs: Self) {
        self.x -= rhs.x;
        self.y -= rhs.y;
    }
}

impl SubAssign<IVec2> for FixedVec2 {
    fn sub_assign(&mut self, rhs: IVec2) {
        self.x -= rhs.x;
        self.y -= rhs.y;
    }
}

impl SubAssign<Vec2> for FixedVec2 {
    fn sub_assign(&mut self, rhs: Vec2) {
        self.x -= rhs.x;
        self.y -= rhs.y;
    }
}

impl MulAssign<I32F32> for FixedVec2 {
    fn mul_assign(&mut self, rhs: I32F32) {
        self.x *= rhs;
        self.y *= rhs;
    }
}

impl MulAssign<i32> for FixedVec2 {
    fn mul_assign(&mut self, rhs: i32) {
        self.x *= rhs;
        self.y *= rhs;
    }
}

impl DivAssign<I32F32> for FixedVec2 {
    fn div_assign(&mut self, rhs: I32F32) {
        self.x /= rhs;
        self.y /= rhs;
    }
}

impl DivAssign<i32> for FixedVec2 {
    fn div_assign(&mut self, rhs: i32) {
        self.x /= rhs;
        self.y /= rhs;
    }
}

impl RemAssign<i32> for FixedVec2 {
    fn rem_assign(&mut self, rhs: i32) {
        self.x %= rhs;
        self.y %= rhs;
    }
}
