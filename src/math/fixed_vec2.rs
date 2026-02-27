use std::{
    fmt::Display,
    ops::{Add, Div, Mul, Rem, Sub},
};

use bevy::math::{I64Vec2, IVec2, Vec2};
use fixed::{traits::ToFixed, types::I32F32};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FixedVec2 {
    pub x: I32F32,
    pub y: I32F32,
}

impl FixedVec2 {
    pub fn new(x: I32F32, y: I32F32) -> Self {
        Self { x, y }
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

    pub fn div_euclid_int(self, rhs: i64) -> Self {
        Self::new(self.x.div_euclid_int(rhs), self.y.div_euclid_int(rhs))
    }

    pub fn rem_euclid_int(self, rhs: i64) -> Self {
        Self::new(self.x.rem_euclid_int(rhs), self.y.rem_euclid_int(rhs))
    }
}

impl From<Vec2> for FixedVec2 {
    fn from(value: Vec2) -> Self {
        Self::new(value.x.to_fixed(), value.y.to_fixed())
    }
}

impl From<FixedVec2> for Vec2 {
    fn from(value: FixedVec2) -> Self {
        Vec2::new(value.x.to_num(), value.y.to_num())
    }
}

impl From<IVec2> for FixedVec2 {
    fn from(value: IVec2) -> Self {
        Self::new(value.x.to_fixed(), value.y.to_fixed())
    }
}

impl From<FixedVec2> for IVec2 {
    fn from(value: FixedVec2) -> Self {
        IVec2::new(value.x.to_num(), value.y.to_num())
    }
}

impl From<I64Vec2> for FixedVec2 {
    fn from(value: I64Vec2) -> Self {
        Self::new(value.x.to_fixed(), value.y.to_fixed())
    }
}

impl From<FixedVec2> for I64Vec2 {
    fn from(value: FixedVec2) -> Self {
        I64Vec2::new(value.x.to_num(), value.y.to_num())
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
        Self::new(self.x + I32F32::from(rhs.x), self.y + I32F32::from(rhs.y))
    }
}

impl Add<Vec2> for FixedVec2 {
    type Output = Self;

    fn add(self, rhs: Vec2) -> Self::Output {
        Self::new(
            self.x + I32F32::from_num(rhs.x),
            self.y + I32F32::from_num(rhs.y),
        )
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
        Self::new(self.x - I32F32::from(rhs.x), self.y - I32F32::from(rhs.y))
    }
}

impl Sub<Vec2> for FixedVec2 {
    type Output = Self;

    fn sub(self, rhs: Vec2) -> Self::Output {
        Self::new(
            self.x - I32F32::from_num(rhs.x),
            self.y - I32F32::from_num(rhs.y),
        )
    }
}

impl Mul<I32F32> for FixedVec2 {
    type Output = Self;

    fn mul(self, rhs: I32F32) -> Self::Output {
        Self::new(self.x * rhs, self.y * rhs)
    }
}

impl Mul<i64> for FixedVec2 {
    type Output = Self;

    fn mul(self, rhs: i64) -> Self::Output {
        Self::new(self.x * rhs, self.y * rhs)
    }
}

impl Div<I32F32> for FixedVec2 {
    type Output = Self;

    fn div(self, rhs: I32F32) -> Self::Output {
        Self::new(self.x / rhs, self.y / rhs)
    }
}

impl Div<i64> for FixedVec2 {
    type Output = Self;

    fn div(self, rhs: i64) -> Self::Output {
        Self::new(self.x / rhs, self.y / rhs)
    }
}

impl Rem<i64> for FixedVec2 {
    type Output = Self;

    fn rem(self, rhs: i64) -> Self::Output {
        Self::new(self.x % rhs, self.y % rhs)
    }
}

use std::ops::{AddAssign, DivAssign, MulAssign, RemAssign, SubAssign};

impl AddAssign for FixedVec2 {
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

impl AddAssign<IVec2> for FixedVec2 {
    fn add_assign(&mut self, rhs: IVec2) {
        self.x += I32F32::from(rhs.x);
        self.y += I32F32::from(rhs.y);
    }
}

impl AddAssign<Vec2> for FixedVec2 {
    fn add_assign(&mut self, rhs: Vec2) {
        self.x += I32F32::from_num(rhs.x);
        self.y += I32F32::from_num(rhs.y);
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
        self.x -= I32F32::from(rhs.x);
        self.y -= I32F32::from(rhs.y);
    }
}

impl SubAssign<Vec2> for FixedVec2 {
    fn sub_assign(&mut self, rhs: Vec2) {
        self.x -= I32F32::from_num(rhs.x);
        self.y -= I32F32::from_num(rhs.y);
    }
}

impl MulAssign<I32F32> for FixedVec2 {
    fn mul_assign(&mut self, rhs: I32F32) {
        self.x *= rhs;
        self.y *= rhs;
    }
}

impl MulAssign<i64> for FixedVec2 {
    fn mul_assign(&mut self, rhs: i64) {
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

impl DivAssign<i64> for FixedVec2 {
    fn div_assign(&mut self, rhs: i64) {
        self.x /= rhs;
        self.y /= rhs;
    }
}

impl RemAssign<i64> for FixedVec2 {
    fn rem_assign(&mut self, rhs: i64) {
        self.x %= rhs;
        self.y %= rhs;
    }
}
