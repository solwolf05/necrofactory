use std::{
    fmt::Display,
    i64,
    ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Rem, RemAssign, Sub, SubAssign},
};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct I32F32(i64);

impl I32F32 {
    pub const ZERO: Self = Self(0);

    pub const ONE: Self = Self(1 << 32);

    pub const HALF: Self = Self(1 << 31);

    pub const MAX: Self = Self(i64::MAX);

    pub const MIN: Self = Self(i64::MIN);

    pub const MIN_POSITIVE: Self = Self(1);

    pub const INT_MASK: i64 = !Self::FRAC_MASK;

    pub const FRAC_MASK: i64 = 0xFFFF_FFFF;

    pub fn int(&self) -> i32 {
        (self.0 >> 32) as i32
    }

    pub fn frac(&self) -> u32 {
        self.0 as u32
    }

    pub fn trunc(self) -> Self {
        if self.frac() == 0 {
            return self;
        }

        let truncated = self.floor();
        if self < 0 {
            truncated + Self::ONE
        } else {
            truncated
        }
    }

    pub fn floor(self) -> Self {
        Self(self.0 & Self::INT_MASK)
    }

    pub fn ceil(self) -> Self {
        if self.frac() == 0 {
            return self;
        }

        let truncated = self.trunc();
        if self > 0 {
            truncated + Self::ONE
        } else {
            truncated
        }
    }

    pub fn round(self) -> Self {
        if self >= 0 {
            (self + Self::HALF).trunc()
        } else {
            (self - Self::HALF).trunc()
        }
    }

    pub fn div_euclid(self, rhs: Self) -> Self {
        Self((((self.0 as i128) << 32).div_euclid(rhs.0 as i128)) as i64)
    }

    pub fn div_euclid_int(self, rhs: i32) -> Self {
        Self((((self.0 as i128) << 32).div_euclid(Self::from(rhs).0 as i128)) as i64)
    }

    pub fn rem_euclid(self, rhs: Self) -> Self {
        Self(((self.0 as i128).rem_euclid(rhs.0 as i128)) as i64)
    }

    pub fn rem_euclid_int(self, rhs: i32) -> Self {
        Self(((self.0 as i128).rem_euclid(Self::from(rhs).0 as i128)) as i64)
    }
}

impl From<i32> for I32F32 {
    fn from(value: i32) -> Self {
        Self((value as i64) << 32)
    }
}

impl From<I32F32> for i32 {
    fn from(value: I32F32) -> Self {
        (value.0 >> 32) as i32
    }
}

impl From<f32> for I32F32 {
    fn from(value: f32) -> Self {
        Self((value * Self::ONE.0 as f32).round() as i64)
    }
}

impl From<I32F32> for f32 {
    fn from(value: I32F32) -> Self {
        value.0 as f32 / I32F32::ONE.0 as f32
    }
}

impl Display for I32F32 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Todo
        f32::from(*self).fmt(f)
    }
}

impl Add for I32F32 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl Add<i32> for I32F32 {
    type Output = Self;

    fn add(self, rhs: i32) -> Self::Output {
        Self(self.0 + Self::from(rhs).0)
    }
}

impl Add<f32> for I32F32 {
    type Output = Self;

    fn add(self, rhs: f32) -> Self::Output {
        Self(self.0 + Self::from(rhs).0)
    }
}

impl Sub for I32F32 {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0 - rhs.0)
    }
}

impl Sub<i32> for I32F32 {
    type Output = Self;

    fn sub(self, rhs: i32) -> Self::Output {
        Self(self.0 - Self::from(rhs).0)
    }
}

impl Sub<f32> for I32F32 {
    type Output = Self;

    fn sub(self, rhs: f32) -> Self::Output {
        Self(self.0 - Self::from(rhs).0)
    }
}

impl Mul for I32F32 {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Self(((self.0 as i128 * rhs.0 as i128) >> 32) as i64)
    }
}

impl Mul<i32> for I32F32 {
    type Output = Self;

    fn mul(self, rhs: i32) -> Self::Output {
        Self(((self.0 as i128 * Self::from(rhs).0 as i128) >> 32) as i64)
    }
}

impl Mul<f32> for I32F32 {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self::Output {
        Self(((self.0 as i128 * Self::from(rhs).0 as i128) >> 32) as i64)
    }
}

impl Div for I32F32 {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        Self((((self.0 as i128) << 32) / rhs.0 as i128) as i64)
    }
}

impl Div<i32> for I32F32 {
    type Output = Self;

    fn div(self, rhs: i32) -> Self::Output {
        Self((((self.0 as i128) << 32) / (Self::from(rhs).0 as i128)) as i64)
    }
}

impl Div<f32> for I32F32 {
    type Output = Self;

    fn div(self, rhs: f32) -> Self::Output {
        Self((((self.0 as i128) << 32) / Self::from(rhs).0 as i128) as i64)
    }
}

impl Rem for I32F32 {
    type Output = Self;

    fn rem(self, rhs: Self) -> Self::Output {
        Self((self.0 as i128 % rhs.0 as i128) as i64)
    }
}

impl Rem<i32> for I32F32 {
    type Output = Self;

    fn rem(self, rhs: i32) -> Self::Output {
        Self((self.0 as i128 % Self::from(rhs).0 as i128) as i64)
    }
}

impl Rem<f32> for I32F32 {
    type Output = Self;

    fn rem(self, rhs: f32) -> Self::Output {
        Self((self.0 as i128 % Self::from(rhs).0 as i128) as i64)
    }
}

impl PartialEq<i32> for I32F32 {
    fn eq(&self, other: &i32) -> bool {
        *self == Self::from(*other)
    }
}

impl PartialEq<f32> for I32F32 {
    fn eq(&self, other: &f32) -> bool {
        *self == Self::from(*other)
    }
}

impl PartialOrd<i32> for I32F32 {
    fn partial_cmp(&self, other: &i32) -> Option<std::cmp::Ordering> {
        self.partial_cmp(&Self::from(*other))
    }
}

impl PartialOrd<f32> for I32F32 {
    fn partial_cmp(&self, other: &f32) -> Option<std::cmp::Ordering> {
        self.partial_cmp(&Self::from(*other))
    }
}

impl AddAssign for I32F32 {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
    }
}

impl AddAssign<i32> for I32F32 {
    fn add_assign(&mut self, rhs: i32) {
        self.0 += Self::from(rhs).0;
    }
}

impl AddAssign<f32> for I32F32 {
    fn add_assign(&mut self, rhs: f32) {
        self.0 += Self::from(rhs).0;
    }
}

impl SubAssign for I32F32 {
    fn sub_assign(&mut self, rhs: Self) {
        self.0 -= rhs.0;
    }
}

impl SubAssign<i32> for I32F32 {
    fn sub_assign(&mut self, rhs: i32) {
        self.0 -= Self::from(rhs).0;
    }
}

impl SubAssign<f32> for I32F32 {
    fn sub_assign(&mut self, rhs: f32) {
        self.0 -= Self::from(rhs).0;
    }
}

impl MulAssign for I32F32 {
    fn mul_assign(&mut self, rhs: Self) {
        self.0 = ((self.0 as i128 * rhs.0 as i128) >> 32) as i64;
    }
}

impl MulAssign<i32> for I32F32 {
    fn mul_assign(&mut self, rhs: i32) {
        self.0 = ((self.0 as i128 * Self::from(rhs).0 as i128) >> 32) as i64;
    }
}

impl MulAssign<f32> for I32F32 {
    fn mul_assign(&mut self, rhs: f32) {
        self.0 = ((self.0 as i128 * Self::from(rhs).0 as i128) >> 32) as i64;
    }
}

impl DivAssign for I32F32 {
    fn div_assign(&mut self, rhs: Self) {
        self.0 = (((self.0 as i128) << 32) / rhs.0 as i128) as i64;
    }
}

impl DivAssign<i32> for I32F32 {
    fn div_assign(&mut self, rhs: i32) {
        self.0 = (((self.0 as i128) << 32) / Self::from(rhs).0 as i128) as i64;
    }
}

impl DivAssign<f32> for I32F32 {
    fn div_assign(&mut self, rhs: f32) {
        self.0 = (((self.0 as i128) << 32) / Self::from(rhs).0 as i128) as i64;
    }
}

impl RemAssign for I32F32 {
    fn rem_assign(&mut self, rhs: Self) {
        self.0 = (self.0 as i128 % rhs.0 as i128) as i64;
    }
}

impl RemAssign<i32> for I32F32 {
    fn rem_assign(&mut self, rhs: i32) {
        self.0 = (self.0 as i128 % Self::from(rhs).0 as i128) as i64;
    }
}

impl RemAssign<f32> for I32F32 {
    fn rem_assign(&mut self, rhs: f32) {
        self.0 = (self.0 as i128 % Self::from(rhs).0 as i128) as i64;
    }
}
