use std::{
    fmt::Display,
    ops::{Add, Div, Mul, Neg, Sub},
    u32,
};

use crate::math::fixed::Fixed;

/// A hybrid floating-point number with integer and fractional parts.
/// Does not have floating point error.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Hybrid {
    pub int: i32,
    pub frac: Fixed,
}

impl Hybrid {
    pub const ZERO: Self = Self {
        int: 0,
        frac: Fixed::ZERO,
    };

    pub const MAX: Self = Self {
        int: i32::MAX,
        frac: Fixed::MAX,
    };

    pub const MIN: Self = Self {
        int: i32::MIN,
        frac: Fixed::ZERO,
    };

    pub const MIN_POSITIVE: Self = Self {
        int: 0,
        frac: Fixed::MIN,
    };

    pub const MIN_NEGATIVE: Self = Self {
        int: -1,
        frac: Fixed::MAX,
    };

    pub fn new(int: i32, frac: Fixed) -> Self {
        Self { int, frac: frac }
    }

    pub fn is_positive(&self) -> bool {
        self.int.is_positive()
    }

    pub fn is_negative(&self) -> bool {
        self.int.is_negative()
    }

    pub fn abs(self) -> Self {
        if self.is_positive() { self } else { -self }
    }

    fn mul_fixed(lhs: i128, rhs: i128) -> Hybrid {
        // Multiply → 128.64
        let prod = lhs * rhs;

        // Shift back to 64.32
        let shifted = prod >> 32;

        if shifted >> 64 != 0 {
            panic!("attempt to multiply with overflow")
        }

        // Extract parts
        let int = (shifted >> 32) as i32;
        let frac = Fixed((shifted as u128 & 0xFFFF_FFFF) as u32);

        Self::new(int, frac)
    }

    fn div_fixed(lhs: i128, rhs: i128) -> Hybrid {
        if rhs == 0 {
            panic!("attempt to divide by zero");
        }

        // Upscale numerator
        let num = lhs << 32;

        let quot = num / rhs;

        if quot >> 64 != 0 {
            panic!("attempt to divide with overflow")
        }

        let int = (quot >> 32) as i32;
        let frac = Fixed((quot as u128 & 0xFFFF_FFFF) as u32);

        Self::new(int, frac)
    }
}

impl From<i32> for Hybrid {
    fn from(value: i32) -> Self {
        Self {
            int: value,
            frac: Fixed::ZERO,
        }
    }
}

impl From<Hybrid> for i32 {
    fn from(value: Hybrid) -> Self {
        value.int
    }
}

impl From<f32> for Hybrid {
    fn from(value: f32) -> Self {
        let int = value.trunc() as i32;
        let frac = Fixed::from(value.fract());
        Self::new(int, frac)
    }
}

impl From<Hybrid> for f32 {
    fn from(value: Hybrid) -> Self {
        value.int as f32 + f32::from(value.frac)
    }
}

impl Display for Hybrid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let explicit_precision = f.precision();
        let precision = f.precision().unwrap_or(6);

        // Combine into signed 64.32 fixed stored in i128
        let raw = ((self.int as i128) << 32) | (self.frac.0 as i128);

        let negative = raw.is_negative();
        let abs = raw.abs() as u128;

        let scale = 10u128.pow(precision as u32);

        // Round: (abs * scale) / 2^32
        let scaled = (abs * scale + (1u128 << 31)) >> 32;

        let int_part = scaled / scale;
        let frac_part = scaled % scale;

        // Explicit precision → print exactly
        if let Some(p) = explicit_precision {
            if negative {
                write!(f, "-")?;
            }

            if p == 0 {
                return write!(f, "{int_part}");
            }

            return write!(f, "{int_part}.{frac_part:0width$}", width = p);
        }

        // Default float-like behavior (trim zeros)
        if negative {
            write!(f, "-")?;
        }

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

impl Add for Hybrid {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        let (frac, overflow) = self.frac.overflowing_add(rhs.frac);
        Self::new(self.int + rhs.int + overflow as i32, frac)
    }
}

impl Add<i32> for Hybrid {
    type Output = Self;

    fn add(self, rhs: i32) -> Self::Output {
        Self::new(self.int + rhs, self.frac)
    }
}

impl Sub for Hybrid {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        let (frac, overflow) = self.frac.overflowing_sub(rhs.frac);
        Self::new(self.int - rhs.int - overflow as i32, frac)
    }
}

impl Sub<i32> for Hybrid {
    type Output = Self;

    fn sub(self, rhs: i32) -> Self::Output {
        Self::new(self.int - rhs, self.frac)
    }
}

impl Mul for Hybrid {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        // Convert to signed 64.32
        let lhs = ((self.int as i128) << 32) + self.frac.0 as i128;
        let rhs = ((rhs.int as i128) << 32) + rhs.frac.0 as i128;

        Self::mul_fixed(lhs, rhs)
    }
}

impl Mul<i32> for Hybrid {
    type Output = Self;

    fn mul(self, rhs: i32) -> Self::Output {
        // Convert to signed 64.32
        let lhs = ((self.int as i128) << 32) + self.frac.0 as i128;
        let rhs = (rhs as i128) << 32;

        Self::mul_fixed(lhs, rhs)
    }
}

impl Div for Hybrid {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        let lhs = ((self.int as i128) << 32) + self.frac.0 as i128;
        let rhs = ((rhs.int as i128) << 32) + rhs.frac.0 as i128;

        Self::div_fixed(lhs, rhs)
    }
}

impl Div<i32> for Hybrid {
    type Output = Self;

    fn div(self, rhs: i32) -> Self::Output {
        let lhs = ((self.int as i128) << 32) + self.frac.0 as i128;
        let rhs = (rhs as i128) << 32;

        Self::div_fixed(lhs, rhs)
    }
}

impl Neg for Hybrid {
    type Output = Self;

    fn neg(self) -> Self::Output {
        let (frac, overflow) = Fixed::ZERO.overflowing_sub(self.frac);
        Self::new(-(overflow as i32) - self.int, frac)
    }
}
