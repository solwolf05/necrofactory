use std::{
    fmt::Display,
    ops::{Add, Div, Mul, Sub},
    u32,
};

#[repr(transparent)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Fixed(pub(super) u32);

impl Fixed {
    pub const ZERO: Self = Self(0);

    pub const MAX: Self = Self(u32::MAX);

    pub const MIN: Self = Self(1);

    pub const FACTOR: u64 = u32::MAX as u64 + 1;

    pub fn inverse(self) -> Self {
        Self(Self::MAX.0 - self.0 + 1)
    }

    pub fn checked_inverse(self) -> Option<Self> {
        if self == Self::ZERO {
            None
        } else {
            Some(Self(Self::MAX.0 - self.0 + 1))
        }
    }

    pub fn checked_add(self, rhs: Self) -> Option<Self> {
        self.0.checked_add(rhs.0).map(Self)
    }

    pub fn strict_add(self, rhs: Self) -> Self {
        Self(self.0.strict_add(rhs.0))
    }

    pub fn saturating_add(self, rhs: Self) -> Self {
        Self(self.0.saturating_add(rhs.0))
    }

    pub fn wrapping_add(self, rhs: Self) -> Self {
        Self(self.0.wrapping_add(rhs.0))
    }

    pub fn overflowing_add(self, rhs: Self) -> (Self, bool) {
        let (result, overflow) = self.0.overflowing_add(rhs.0);
        (Self(result), overflow)
    }

    pub fn carrying_add(self, rhs: Self, carry: bool) -> (Self, bool) {
        let (sum, carry) = self.0.carrying_add(rhs.0, carry);
        (Self(sum), carry)
    }

    pub fn checked_sub(self, rhs: Self) -> Option<Self> {
        self.0.checked_sub(rhs.0).map(Self)
    }

    pub fn strict_sub(self, rhs: Self) -> Self {
        Self(self.0.strict_sub(rhs.0))
    }

    pub fn saturating_sub(self, rhs: Self) -> Self {
        Self(self.0.saturating_sub(rhs.0))
    }

    pub fn wrapping_sub(self, rhs: Self) -> Self {
        Self(self.0.wrapping_sub(rhs.0))
    }

    pub fn overflowing_sub(self, rhs: Self) -> (Self, bool) {
        let (result, overflow) = self.0.overflowing_sub(rhs.0);
        (Self(result), overflow)
    }

    pub fn checked_mul(self, rhs: Self) -> Option<Self> {
        let prod = (self.0 as u64) * (rhs.0 as u64);
        let shifted = prod >> 32;
        if shifted > u32::MAX as u64 {
            None
        } else {
            Some(Self(shifted as u32))
        }
    }

    pub fn strict_mul(self, rhs: Self) -> Self {
        match self.checked_mul(rhs) {
            Some(v) => v,
            None => panic!("Fixed32 strict_mul overflow"),
        }
    }

    pub fn saturating_mul(self, rhs: Self) -> Self {
        let prod = (self.0 as u64) * (rhs.0 as u64);
        let shifted = prod >> 32;
        if shifted > u32::MAX as u64 {
            Self::MAX
        } else {
            Self(shifted as u32)
        }
    }

    pub fn wrapping_mul(self, rhs: Self) -> Self {
        let prod = (self.0 as u64).wrapping_mul(rhs.0 as u64);
        Self((prod >> 32) as u32)
    }

    pub fn overflowing_mul(self, rhs: Self) -> (Self, bool) {
        let prod = (self.0 as u64) * (rhs.0 as u64);
        let shifted = prod >> 32;
        let overflow = shifted > u32::MAX as u64;
        (Self(shifted as u32), overflow)
    }

    pub fn checked_div(self, rhs: Self) -> Option<Self> {
        if rhs.0 == 0 {
            return None;
        }
        let num = (self.0 as u64) << 32;
        let result = num / rhs.0 as u64;
        if result > u32::MAX as u64 {
            None
        } else {
            Some(Self(result as u32))
        }
    }

    pub fn strict_div(self, rhs: Self) -> Self {
        match self.checked_div(rhs) {
            Some(v) => v,
            None => panic!("Unit32 strict_div overflow or divide by zero"),
        }
    }

    pub fn saturating_div(self, rhs: Self) -> Self {
        if rhs.0 == 0 {
            panic!("attempt to divide by zero");
        }
        let num = (self.0 as u64) << 32;
        let result = num / rhs.0 as u64;
        if result > u32::MAX as u64 {
            Self::MAX
        } else {
            Self(result as u32)
        }
    }

    pub fn wrapping_div(self, rhs: Self) -> Self {
        if rhs.0 == 0 {
            panic!("attempt to divide by zero");
        }
        let num = (self.0 as u64) << 32;
        Self((num / rhs.0 as u64) as u32)
    }

    pub fn overflowing_div(self, rhs: Self) -> (Self, bool) {
        if rhs.0 == 0 {
            panic!("attempt to divide by zero");
        }
        let num = (self.0 as u64) << 32;
        let result = num / rhs.0 as u64;
        let overflow = result > u32::MAX as u64;
        (Self(result as u32), overflow)
    }

    pub fn checked_mul_u32(self, rhs: u32) -> Option<Self> {
        self.0.checked_mul(rhs).map(Self)
    }

    pub fn strict_mul_u32(self, rhs: u32) -> Self {
        Self(self.0.strict_mul(rhs))
    }

    pub fn saturating_mul_u32(self, rhs: u32) -> Self {
        Self(self.0.saturating_mul(rhs))
    }

    pub fn wrapping_mul_u32(self, rhs: u32) -> Self {
        Self(self.0.wrapping_mul(rhs))
    }

    pub fn overflowing_mul_u32(self, rhs: u32) -> (Self, bool) {
        let (v, of) = self.0.overflowing_mul(rhs);
        (Self(v), of)
    }

    pub fn checked_div_u32(self, rhs: u32) -> Option<Self> {
        self.0.checked_div(rhs).map(Self)
    }

    pub fn strict_div_u32(self, rhs: u32) -> Self {
        Self(self.0.strict_div(rhs))
    }

    pub fn saturating_div_u32(self, rhs: u32) -> Self {
        Self(self.0.saturating_div(rhs))
    }

    pub fn wrapping_div_u32(self, rhs: u32) -> Self {
        Self(self.0.wrapping_div(rhs))
    }

    pub fn overflowing_div_u32(self, rhs: u32) -> (Self, bool) {
        let (v, of) = self.0.overflowing_div(rhs);
        (Self(v), of)
    }
}

impl From<f32> for Fixed {
    fn from(value: f32) -> Self {
        if value < 0.0 || value >= 1.0 as f32 {
            panic!("value out of bounds");
        }
        Self((value * Self::FACTOR as f32) as u32)
    }
}

impl From<Fixed> for f32 {
    fn from(value: Fixed) -> Self {
        value.0 as f32 / Fixed::FACTOR as f32
    }
}

impl Display for Fixed {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let explicit_precision = f.precision();
        let precision = explicit_precision.unwrap_or(6);

        // Use u128 to avoid overflow
        let scale = 10u128.pow(precision as u32);
        let value = self.0 as u128;

        // Round: (value * scale) / 2^32
        let scaled = (value * scale + (1u128 << 31)) >> 32;

        let int_part = scaled / scale;
        let frac_part = scaled % scale;

        // If user explicitly requested precision, print exactly that
        if let Some(p) = explicit_precision {
            if p == 0 {
                return write!(f, "{int_part}");
            }
            return write!(f, "{int_part}.{frac_part:0width$}", width = p);
        }

        // Default behavior (like floats):
        // Trim trailing zeros
        if frac_part == 0 {
            return write!(f, "{int_part}");
        }

        let mut frac_str = format!("{:0width$}", frac_part, width = precision);

        while frac_str.ends_with('0') {
            frac_str.pop();
        }

        write!(f, "{int_part}.{frac_str}")
    }
}

impl Add for Fixed {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl Sub for Fixed {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0 - rhs.0)
    }
}

impl Mul for Fixed {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Self(((self.0 as u64 * rhs.0 as u64) >> 32) as u32)
    }
}

impl Mul<u32> for Fixed {
    type Output = Self;

    fn mul(self, rhs: u32) -> Self::Output {
        Self(self.0 * rhs)
    }
}

impl Div for Fixed {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        if rhs.0 == 0 {
            panic!("attempt to divide by zero");
        }
        let num = (self.0 as u64) << 32;
        let result = num / rhs.0 as u64;
        if result > u32::MAX as u64 {
            panic!("attempt to divide with overflow")
        } else {
            Self(result as u32)
        }
    }
}

impl Div<u32> for Fixed {
    type Output = Self;

    fn div(self, rhs: u32) -> Self::Output {
        Self(self.0 / rhs)
    }
}
