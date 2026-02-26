use std::{
    fmt::Display,
    ops::{Add, Div, Mul, Sub},
};

use bevy::math::Vec2;

use crate::math::hybrid::Hybrid;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct HybridVec2 {
    pub x: Hybrid,
    pub y: Hybrid,
}

impl HybridVec2 {
    pub fn new(x: Hybrid, y: Hybrid) -> Self {
        Self { x, y }
    }
}

impl From<Vec2> for HybridVec2 {
    fn from(value: Vec2) -> Self {
        Self::new(value.x.into(), value.y.into())
    }
}

impl From<HybridVec2> for Vec2 {
    fn from(value: HybridVec2) -> Self {
        Vec2::new(value.x.into(), value.y.into())
    }
}

impl Display for HybridVec2 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

impl Add for HybridVec2 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self::new(self.x + rhs.x, self.y + rhs.y)
    }
}

impl Sub for HybridVec2 {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self::new(self.x - rhs.x, self.y - rhs.y)
    }
}

impl Mul<Hybrid> for HybridVec2 {
    type Output = Self;

    fn mul(self, rhs: Hybrid) -> Self::Output {
        Self::new(self.x * rhs, self.y * rhs)
    }
}

impl Mul<i32> for HybridVec2 {
    type Output = Self;

    fn mul(self, rhs: i32) -> Self::Output {
        Self::new(self.x * rhs, self.y * rhs)
    }
}

impl Div<i32> for HybridVec2 {
    type Output = Self;

    fn div(self, rhs: i32) -> Self::Output {
        Self::new(self.x / rhs, self.y / rhs)
    }
}
