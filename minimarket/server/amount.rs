#![allow(dead_code)]

use std::cmp::Ordering;
use std::fmt;
use std::iter::{Product, Sum};
use std::ops::*;
use std::str::FromStr;

#[derive(Clone, Debug, Default)]
pub struct FormatOpts {
    pub width: Option<usize>,
    pub precision: Option<usize>,
    pub sign: bool,
    pub accounting: bool,
}

#[derive(Debug, PartialEq, Eq)]
pub enum ParseError {
    NonDigit,
    DoubleDecimal,
    MaxIntExceeded,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ParseError::NonDigit => write!(f, "non digit encountered"),
            ParseError::DoubleDecimal => write!(f, "double decimal encountered"),
            ParseError::MaxIntExceeded => write!(f, "maximum integral part exceeded"),
        }
    }
}

impl std::error::Error for ParseError {}

const U64_MAX_NUM_DIGITS: u64 = 20;
const AMOUNT64_DECS: u64 = 9;
const AMOUNT64_INTS: u64 = U64_MAX_NUM_DIGITS - AMOUNT64_DECS;

#[derive(Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct Amount64(u64);

impl Amount64 {
    pub const INTEGERS: u64 = AMOUNT64_INTS;
    pub const DECIMALS: u64 = AMOUNT64_DECS;
    pub const EPSILON: Self = Self(1);

    pub const MAX: Self = Self(u64::MAX);
    pub const MIN: Self = Self(0);

    pub const SCALE: u64 = 10u64.pow(AMOUNT64_DECS as _);
    pub const MAX_DEC_U64: u64 = 10u64.pow(AMOUNT64_DECS as _) - 1;

    pub const ONE: Self = Self(1 * 10u64.pow(AMOUNT64_DECS as _));
    pub const ZERO: Self = Self(0);

    pub const fn from_u64(u: u64) -> Option<Self> {
        const N: u64 = 10u64.pow(AMOUNT64_DECS as _);
        if u > Self::MAX.0 / N {
            return None;
        }
        Some(Self(u * N))
    }

    /// Expects an unscaled integer and fully scaled decimal
    pub const fn from_parts(int: u64, dec: u64) -> Option<Self> {
        const MAX_INT: u64 = Amount64::MAX.to_parts().0;
        const MAX_DEC: u64 = Amount64::MAX.to_parts().1;
        // TODO: truncate dec if necessary
        if int > MAX_INT {
            return None;
        } else if int == MAX_INT {
            if dec > MAX_DEC {
                return None;
            }
        }
        Some(Self(int * 10u64.pow(AMOUNT64_DECS as _) + dec))
    }

    /// Expects both a scaled integer and decimal
    pub const fn from_split(int: u64, dec: u64) -> Option<Self> {
        // TODO: checks
        if dec >= 10u64.pow(AMOUNT64_DECS as _) {
            return None;
        }
        Some(Self(int + dec))
    }

    /// Expects both an unscaled integer and decimal. This means that
    /// `Amount64::from_unscaled(1, 5)` yeilds 1.5, `Amount64::from_unscaled(1, 50)` yeilds the
    /// same, `Amount64::from_unscaled(1, 505)` yeilds 1.505, etc. Decimal values exceeding the
    /// max decimal range are treated as invalid and None is returned.
    pub const fn from_unscaled_parts(int: u64, dec: u64) -> Option<Self> {
        const MAX_INT: u64 = Amount64::MAX.to_parts().0;
        const MAX_DEC: u64 = Amount64::MAX.to_parts().1;

        if dec >= Amount64::SCALE {
            return None;
        }
        let dec = if dec != 0 {
            dec * 10u64.pow(AMOUNT64_DECS as u32 - dec.ilog10() - 1)
        } else {
            0
        };

        if int > MAX_INT {
            return None;
        } else if int == MAX_INT {
            if dec > MAX_DEC {
                return None;
            }
        }
        Some(Self(int * Amount64::SCALE + dec))
    }

    pub const fn from_int_dec(int: u64, dec: u64) -> Option<Self> {
        Self::from_unscaled_parts(int, dec)
    }

    pub fn from_f64(f: f64) -> Option<Self> {
        if f < 0.0 || f > u64::MAX as f64 {
            return None;
        }
        const N: u64 = 10u64.pow(AMOUNT64_DECS as _);
        Some(Self((f.trunc() as u64 * N) + (f.fract() * N as f64).round() as u64))
    }

    pub fn to_f64(self) -> f64 {
        let (int, dec) = self.to_parts();
        int as f64 + (dec as f64 / 10.0f64.powi(AMOUNT64_DECS as _))
    }

    pub const fn to_parts(self) -> (u64, u64) {
        const N: u64 = 10u64.pow(AMOUNT64_DECS as u32);
        (self.0 / N, self.0 % N)
    }

    pub const fn split(self) -> (u64, u64) {
        const N: u64 = 10u64.pow(AMOUNT64_DECS as u32);
        ((self.0 / N) * N, self.0 % N)
    }

    pub const fn to_be_bytes(self) -> [u8; 8] {
        self.0.to_be_bytes()
    }

    pub const fn to_le_bytes(self) -> [u8; 8] {
        self.0.to_le_bytes()
    }

    pub const fn to_ne_bytes(self) -> [u8; 8] {
        self.0.to_ne_bytes()
    }

    pub const fn from_be_bytes(arr: [u8; 8]) -> Self {
        Self(u64::from_be_bytes(arr))
    }

    pub const fn from_le_bytes(arr: [u8; 8]) -> Self {
        Self(u64::from_le_bytes(arr))
    }

    pub const fn from_ne_bytes(arr: [u8; 8]) -> Self {
        Self(u64::from_ne_bytes(arr))
    }

    pub fn be_bytes_into(self, s: &mut [u8]) {
        assert!(s.len() >= 8, "need length of at least 8, got {}", s.len());
        unsafe {
            *s.as_mut_ptr().cast::<[u8; 8]>() = self.to_be_bytes();
        }
    }

    pub fn le_bytes_into(self, s: &mut [u8]) {
        assert!(s.len() >= 8, "need length of at least 8, got {}", s.len());
        unsafe {
            *s.as_mut_ptr().cast::<[u8; 8]>() = self.to_le_bytes();
        }
    }

    pub fn ne_bytes_into(self, s: &mut [u8]) {
        assert!(s.len() >= 8, "need length of at least 8, got {}", s.len());
        unsafe {
            *s.as_mut_ptr().cast::<[u8; 8]>() = self.to_ne_bytes();
        }
    }

    pub const fn be_bytes_from(s: &[u8]) -> Self {
        assert!(s.len() >= 8, "need length of at least 8");
        unsafe { Self::from_be_bytes(*s.as_ptr().cast::<[u8; 8]>()) }
    }

    pub const fn le_bytes_from(s: &[u8]) -> Self {
        assert!(s.len() >= 8, "need length of at least 8");
        unsafe { Self::from_le_bytes(*s.as_ptr().cast::<[u8; 8]>()) }
    }

    pub const fn ne_bytes_from(s: &[u8]) -> Self {
        assert!(s.len() >= 8, "need length of at least 8");
        unsafe { Self::from_ne_bytes(*s.as_ptr().cast::<[u8; 8]>()) }
    }

    /// Round `places` to the right of the decimal. Negative means round to the left (integer
    /// part). 0 rounds to ones place, 1 rounds to tens place, etc. Rounding to more integer places
    /// than the number has makes it zero.
    pub fn round(self, places: i32) -> Self {
        if places < 0 {
            if places < -(AMOUNT64_INTS as i32) {
                return Self(0);
            }
            return self.round_int((-places) as u32);
        } else if places > 0 {
            if places >= AMOUNT64_DECS as _ {
                return self;
            }
            return self.round_dec(places as u32);
        } else {
            let (mut int, dec) = self.to_parts();
            if dec > 5 * 10u64.pow(AMOUNT64_DECS as u32 - 1) {
                int += 1;
            }
            Self::from_parts(int, 0).unwrap_or(Self(0))
        }
    }

    fn round_dec(self, places: u32) -> Self {
        let (mut int, mut dec) = self.to_parts();
        let n = 10u64.pow(AMOUNT64_DECS as u32 - places - 1);
        dec /= n;
        let d = dec % 10;
        dec -= d;
        if d % 10 >= 5 {
            dec += 10;
            if dec >= 100 {
                int += 1;
                dec = 0;
            }
        }
        dec *= n;
        Self::from_parts(int, dec).unwrap_or(Self(0))
    }

    fn round_int(self, places: u32) -> Self {
        let (mut int, _) = self.to_parts();
        let n = 10u64.pow(places - 1);
        int /= n;
        let d = int % 10;
        int -= d;
        if d % 10 >= 5 {
            int += 10;
        }
        int *= n;
        Self::from_parts(int, 0).unwrap_or(Self(0))
    }

    pub const fn trunc(self) -> Self {
        const N: u64 = 10u64.pow(AMOUNT64_DECS as u32);
        Self((self.0 / N) * N)
    }

    pub const fn fract(self) -> Self {
        const N: u64 = 10u64.pow(AMOUNT64_DECS as u32);
        Self(self.0 % N)
    }

    pub fn format(self, opts: FormatOpts) -> String {
        let (mut num, prec, width) = (self, opts.precision.unwrap_or(0), opts.width.unwrap_or(0));
        if let Some(_) = opts.precision {
            num = num.round(prec as _);
        }
        let (int, mut dec) = num.to_parts();
        while dec != 0 && dec % 10 == 0 {
            dec /= 10;
        }
        let mut res = int.to_string();
        if let Some(diff) = width.checked_sub(res.len()) {
            res = "0".repeat(diff) + &res;
        }
        if opts.precision.is_none() || prec >= 1 {
            let mut dec_str = format!(".{dec}");
            if let Some(diff) = prec.checked_sub(dec_str.len() - 1) {
                dec_str.push_str(&"0".repeat(diff));
            }
            res.push_str(&dec_str);
        }
        res
    }
}

impl FromStr for Amount64 {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        const MAX_INT: u64 = Amount64::MAX.to_parts().0;

        let mut in_dec = false;
        let (mut int, mut dec) = (0, 0);
        let mut dec_place = AMOUNT64_DECS as u32 - 1;
        for b in s.bytes() {
            match b {
                b'.' => {
                    if in_dec {
                        return Err(ParseError::DoubleDecimal);
                    }
                    in_dec = true;
                }
                b'0'..=b'9' => {
                    if !in_dec {
                        int = int * 10 + (b - b'0') as u64;
                        if int > MAX_INT {
                            return Err(ParseError::MaxIntExceeded);
                        }
                    } else {
                        if dec_place == 0 {
                            break;
                        }
                        dec += (b - b'0') as u64 * 10u64.pow(dec_place);
                        dec_place -= 1;
                    }
                }
                _ => return Err(ParseError::NonDigit),
            }
        }
        Ok(Self::from_parts(int, dec).unwrap())
    }
}

impl Add for Amount64 {
    type Output = Self;

    fn add(self, other: Self) -> Self::Output {
        Self(self.0 + other.0)
    }
}

impl AddAssign for Amount64 {
    fn add_assign(&mut self, other: Self) {
        self.0 += other.0
    }
}

impl Sub for Amount64 {
    type Output = Self;

    fn sub(self, other: Self) -> Self::Output {
        Self(self.0 - other.0)
    }
}

impl SubAssign for Amount64 {
    fn sub_assign(&mut self, other: Self) {
        self.0 -= other.0
    }
}

impl Mul for Amount64 {
    type Output = Self;

    fn mul(self, other: Self) -> Self::Output {
        let (int1, dec1) = self.to_parts();
        let (int2, dec2) = other.to_parts();
        Self(self.0 * int2 + int1 * dec2 + (dec1 * dec2 / Self::SCALE))
        //Self(self.0 * other.0)
    }
}

impl MulAssign for Amount64 {
    fn mul_assign(&mut self, other: Self) {
         *self = *self * other;
        //self.0 *= other.0
    }
}

impl Div for Amount64 {
    type Output = Self;

    fn div(self, other: Self) -> Self::Output {
        let int = self.0 / other.0;
        let fract = (self.0 % other.0) * Self::SCALE / other.0;
        Self::from_int_dec(int, fract).expect("bad division")
        //Self(self.0 / other.0)
    }
}

impl DivAssign for Amount64 {
    fn div_assign(&mut self, other: Self) {
        *self = *self / other;
        //self.0 /= other.0
    }
}

impl Rem for Amount64 {
    type Output = Self;

    fn rem(self, other: Self) -> Self::Output {
        Self(self.0 - other.0)
    }
}

impl RemAssign for Amount64 {
    fn rem_assign(&mut self, other: Self) {
        self.0 -= other.0
    }
}

impl fmt::Display for Amount64 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(&self.format(FormatOpts {
            precision: f.precision(),
            width: f.width(),
            sign: f.sign_plus(),
            ..Default::default()
        }))
    }
}

impl Sum for Amount64 {
    fn sum<I>(iter: I) -> Self
    where I: Iterator<Item = Self> {
        iter.fold(Self(0), |sum, n| sum + n)
    }
}

impl Product for Amount64 {
    fn product<I>(iter: I) -> Self
    where I: Iterator<Item = Self> {
        iter.fold(Self(0), |prod, n| prod * n)
    }
}

impl<'a> Sum<&'a Amount64> for Amount64 {
    fn sum<I>(iter: I) -> Self
    where I: Iterator<Item = &'a Self> {
        iter.fold(Self(0), |sum, &n| sum + n)
    }
}

impl<'a> Product<&'a Amount64> for Amount64 {
    fn product<I>(iter: I) -> Self
    where I: Iterator<Item = &'a Self> {
        iter.fold(Self(0), |prod, &n| prod * n)
    }
}

const U128_MAX_NUM_DIGITS: u128 = 39;
const AMOUNT128_DECS: u128 = 18;
const AMOUNT128_INTS: u128 = U128_MAX_NUM_DIGITS - AMOUNT128_DECS;

#[derive(Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct Amount128(u128);

impl Amount128 {
    pub const INTEGERS: u128 = AMOUNT128_INTS;
    pub const DECIMALS: u128 = AMOUNT128_DECS;

    pub const MAX: Self = Self(u128::MAX);
    pub const MIN: Self = Self(0);

    pub const MAX_DEC_U128: u128 = 10u128.pow(AMOUNT128_DECS as _) - 1;

    pub const ONE: Self = Self(1 * 10u128.pow(AMOUNT128_DECS as _));
    pub const ZERO: Self = Self(0);

    pub const fn from_u128(u: u128) -> Option<Self> {
        const N: u128 = 10u128.pow(AMOUNT128_DECS as _);
        if u > Self::MAX.0 / N {
            return None;
        }
        Some(Self(u * N))
    }

    pub const fn from_parts(int: u128, dec: u128) -> Option<Self> {
        const MAX_INT: u128 = Amount128::MAX.to_parts().0;
        const MAX_DEC: u128 = Amount128::MAX.to_parts().1;
        // TODO: truncate dec if necessary
        if int > MAX_INT {
            return None;
        } else if int == MAX_INT {
            if dec > MAX_DEC {
                return None;
            }
        }
        Some(Self(int * 10u128.pow(AMOUNT128_DECS as _) + dec))
    }

    pub const fn from_split(int: u128, dec: u128) -> Option<Self> {
        // TODO: checks
        if dec >= 10u128.pow(AMOUNT128_DECS as _) {
            return None;
        }
        Some(Self(int + dec))
    }

    pub fn from_f64(f: f64) -> Option<Self> {
        if f < 0.0 || f > u128::MAX as f64 {
            return None;
        }
        // TODO: fix
        const N: u128 = 10u128.pow(AMOUNT128_DECS as _);
        let s = Self((f.trunc() as u128 * N) + (f.fract() * N as f64).round() as u128);
        Some(s)
    }

    pub fn to_f64(self) -> f64 {
        let (int, dec) = self.to_parts();
        int as f64 + (dec as f64 / 10.0f64.powi(AMOUNT128_DECS as _))
    }

    pub const fn to_parts(self) -> (u128, u128) {
        const N: u128 = 10u128.pow(AMOUNT128_DECS as u32);
        (self.0 / N, self.0 % N)
    }

    pub const fn split(self) -> (u128, u128) {
        const N: u128 = 10u128.pow(AMOUNT128_DECS as u32);
        ((self.0 / N) * N, self.0 % N)
    }

    pub const fn to_be_bytes(self) -> [u8; 16] {
        self.0.to_be_bytes()
    }

    pub const fn to_le_bytes(self) -> [u8; 16] {
        self.0.to_le_bytes()
    }

    pub const fn to_ne_bytes(self) -> [u8; 16] {
        self.0.to_ne_bytes()
    }

    pub const fn from_be_bytes(arr: [u8; 16]) -> Self {
        Self(u128::from_be_bytes(arr))
    }

    pub const fn from_le_bytes(arr: [u8; 16]) -> Self {
        Self(u128::from_le_bytes(arr))
    }

    pub const fn from_ne_bytes(arr: [u8; 16]) -> Self {
        Self(u128::from_ne_bytes(arr))
    }

    pub fn be_bytes_into(self, s: &mut [u8]) {
        assert!(s.len() >= 16, "need length of at least 16, got {}", s.len());
        unsafe {
            *s.as_mut_ptr().cast::<[u8; 16]>() = self.to_be_bytes();
        }
    }

    pub fn le_bytes_into(self, s: &mut [u8]) {
        assert!(s.len() >= 16, "need length of at least 16, got {}", s.len());
        unsafe {
            *s.as_mut_ptr().cast::<[u8; 16]>() = self.to_le_bytes();
        }
    }

    pub fn ne_bytes_into(self, s: &mut [u8]) {
        assert!(s.len() >= 16, "need length of at least 16, got {}", s.len());
        unsafe {
            *s.as_mut_ptr().cast::<[u8; 16]>() = self.to_ne_bytes();
        }
    }

    pub const fn be_bytes_from(s: &[u8]) -> Self {
        assert!(s.len() >= 16, "need length of at least 16");
        unsafe { Self::from_be_bytes(*s.as_ptr().cast::<[u8; 16]>()) }
    }

    pub const fn le_bytes_from(s: &[u8]) -> Self {
        assert!(s.len() >= 16, "need length of at least 16");
        unsafe { Self::from_le_bytes(*s.as_ptr().cast::<[u8; 16]>()) }
    }

    pub const fn ne_bytes_from(s: &[u8]) -> Self {
        assert!(s.len() >= 16, "need length of at least 16");
        unsafe { Self::from_ne_bytes(*s.as_ptr().cast::<[u8; 16]>()) }
    }

    /// Round `places` to the right of the decimal. Negative means round to the left (integer
    /// part). 0 rounds to ones place, 1 rounds to tens place, etc. Rounding to more integer places
    /// than the number has makes it zero.
    pub fn round(self, places: i32) -> Self {
        if places < 0 {
            if places < -(AMOUNT128_INTS as i32) {
                return Self(0);
            }
            return self.round_int((-places) as u32);
        } else if places > 0 {
            if places >= AMOUNT128_DECS as _ {
                return self;
            }
            return self.round_dec(places as u32);
        } else {
            let (mut int, dec) = self.to_parts();
            if dec > 5 * 10u128.pow(AMOUNT128_DECS as u32 - 1) {
                int += 1;
            }
            Self::from_parts(int, 0).unwrap_or(Self(0))
        }
    }

    fn round_dec(self, places: u32) -> Self {
        let (mut int, mut dec) = self.to_parts();
        let n = 10u128.pow(AMOUNT128_DECS as u32 - places - 1);
        dec /= n;
        let d = dec % 10;
        dec -= d;
        if d % 10 >= 5 {
            dec += 10;
            if dec >= 100 {
                int += 1;
                dec = 0;
            }
        }
        dec *= n;
        Self::from_parts(int, dec).unwrap_or(Self(0))
    }

    fn round_int(self, places: u32) -> Self {
        let (mut int, _) = self.to_parts();
        let n = 10u128.pow(places - 1);
        int /= n;
        let d = int % 10;
        int -= d;
        if d % 10 >= 5 {
            int += 10;
        }
        int *= n;
        Self::from_parts(int, 0).unwrap_or(Self(0))
    }

    pub const fn trunc(self) -> Self {
        const N: u128 = 10u128.pow(AMOUNT128_DECS as u32);
        Self((self.0 / N) * N)
    }

    pub const fn fract(self) -> Self {
        const N: u128 = 10u128.pow(AMOUNT128_DECS as u32);
        Self(self.0 % N)
    }

    pub fn format(self, opts: FormatOpts) -> String {
        let (mut num, prec, width) = (self, opts.precision.unwrap_or(0), opts.width.unwrap_or(0));
        if let Some(_) = opts.precision {
            num = num.round(prec as _);
        }
        let (int, mut dec) = num.to_parts();
        while dec != 0 && dec % 10 == 0 {
            dec /= 10;
        }
        let mut res = int.to_string();
        if let Some(diff) = width.checked_sub(res.len()) {
            res = "0".repeat(diff) + &res;
        }
        if opts.precision.is_none() || prec >= 1 {
            let mut dec_str = format!(".{dec}");
            if let Some(diff) = prec.checked_sub(dec_str.len() - 1) {
                dec_str.push_str(&"0".repeat(diff));
            }
            res.push_str(&dec_str);
        }
        res
    }
}

impl FromStr for Amount128 {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        const MAX_INT: u128 = Amount128::MAX.to_parts().0;

        let mut in_dec = false;
        let (mut int, mut dec) = (0, 0);
        let mut dec_place = AMOUNT128_DECS as u32 - 1;
        for b in s.bytes() {
            match b {
                b'.' => {
                    if in_dec {
                        return Err(ParseError::DoubleDecimal);
                    }
                    in_dec = true;
                }
                b'0'..=b'9' => {
                    if !in_dec {
                        int = int * 10 + (b - b'0') as u128;
                        if int > MAX_INT {
                            return Err(ParseError::MaxIntExceeded);
                        }
                    } else {
                        if dec_place == 0 {
                            break;
                        }
                        dec += (b - b'0') as u128 * 10u128.pow(dec_place);
                        dec_place -= 1;
                    }
                }
                _ => return Err(ParseError::NonDigit),
            }
        }
        Ok(Self::from_parts(int, dec).unwrap())
    }
}

impl Add for Amount128 {
    type Output = Self;

    fn add(self, other: Self) -> Self::Output {
        Self(self.0 + other.0)
    }
}

impl AddAssign for Amount128 {
    fn add_assign(&mut self, other: Self) {
        self.0 += other.0
    }
}

impl Sub for Amount128 {
    type Output = Self;

    fn sub(self, other: Self) -> Self::Output {
        Self(self.0 - other.0)
    }
}

impl SubAssign for Amount128 {
    fn sub_assign(&mut self, other: Self) {
        self.0 -= other.0
    }
}

impl Mul for Amount128 {
    type Output = Self;

    fn mul(self, other: Self) -> Self::Output {
        Self(self.0 * other.0)
    }
}

impl MulAssign for Amount128 {
    fn mul_assign(&mut self, other: Self) {
        self.0 *= other.0
    }
}

impl Div for Amount128 {
    type Output = Self;

    fn div(self, other: Self) -> Self::Output {
        Self(self.0 / other.0)
    }
}

impl DivAssign for Amount128 {
    fn div_assign(&mut self, other: Self) {
        self.0 /= other.0
    }
}

impl Rem for Amount128 {
    type Output = Self;

    fn rem(self, other: Self) -> Self::Output {
        Self(self.0 - other.0)
    }
}

impl RemAssign for Amount128 {
    fn rem_assign(&mut self, other: Self) {
        self.0 -= other.0
    }
}

impl fmt::Display for Amount128 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // TODO: alternate
        let (prec, num) = if let Some(p) = f.precision() {
            (p, self.round(p as _))
        } else {
            (0, *self)
        };
        let (int, mut dec) = num.to_parts();
        while dec != 0 && dec % 10 == 0 {
            dec /= 10;
        }
        f.pad_integral(true, "", &int.to_string())?;
        if !(dec == 0 && prec == 0) {
            let mut dec_str = dec.to_string();
            if dec_str.len() < prec {
                dec_str.push_str(&"0".repeat(prec - dec_str.len()));
            }
            write!(f, ".{dec_str}")?;
        }
        Ok(())
    }
}

impl Sum for Amount128 {
    fn sum<I>(iter: I) -> Self
    where I: Iterator<Item = Self> {
        iter.fold(Self(0), |sum, n| sum + n)
    }
}

impl Product for Amount128 {
    fn product<I>(iter: I) -> Self
    where I: Iterator<Item = Self> {
        iter.fold(Self(0), |prod, n| prod * n)
    }
}

impl<'a> Sum<&'a Amount128> for Amount128 {
    fn sum<I>(iter: I) -> Self
    where I: Iterator<Item = &'a Self> {
        iter.fold(Self(0), |sum, &n| sum + n)
    }
}

impl<'a> Product<&'a Amount128> for Amount128 {
    fn product<I>(iter: I) -> Self
    where I: Iterator<Item = &'a Self> {
        iter.fold(Self(0), |prod, &n| prod * n)
    }
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default)]
pub struct Price(Amount64);

impl Price {
    pub const MIN: Self = Self(Amount64::MIN);
    pub const MAX: Self = Self(Amount64(Amount64::MAX.0 - 1));
    /*
    pub const MIN_F64: f64 = -999_999_999.999_999_999;
    pub const MAX_F64: f64 = 999_999_999.999_999_999;
    */

    //pub const MARKET: Self = Self(0);
    pub const MARKET: Self = Self(Amount64::MAX);
    pub const ZERO: Self = Self(Amount64::ZERO);

    pub const fn from_parts(int: u64, dec: u64) -> Option<Self> {
        match Amount64::from_parts(int, dec) {
            Some(amt) => Some(Self(amt)),
            None => None,
        }
    }

    pub const fn from_int_dec(int: u64, dec: u64) -> Option<Self> {
        match Amount64::from_int_dec(int, dec) {
            Some(amt) => Some(Self(amt)),
            None => None,
        }
    }

    pub fn from_amount(amt: Amount64) -> Option<Self> {
        let price = Self(amt);
        if price.is_market() || !price.is_valid() {
            return None;
        }
        Some(price)
    }

    pub fn from_f64(f: f64) -> Option<Self> {
        let inner = Amount64::from_f64(f)?;
        if inner > Self::MAX.0 {
            return None;
        }
        Some(Self(inner))
    }

    pub const fn from_bytes(b: [u8; 8]) -> Self {
        Self(Amount64::from_le_bytes(b))
    }

    pub const fn to_bytes(self) -> [u8; 8] {
        self.0.to_le_bytes()
    }

    pub const fn is_market(self) -> bool {
        self.0.0 == Price::MARKET.0.0
    }

    pub const fn is_valid(self) -> bool {
        self.is_market() || (self.0.0 <= Price::MAX.0.0 && self.0.0 >= Price::MIN.0.0)
    }

    pub const fn to_amount64(&self) -> Amount64 {
        self.0
    }
}

impl PartialEq for Price {
    fn eq(&self, other: &Self) -> bool {
        self.is_valid() && self.0 == other.0
    }
}

impl PartialOrd for Price {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if !self.is_valid() {
            return None;
        }
        self.0.partial_cmp(&other.0)
    }
}

impl PartialEq<Amount64> for Price {
    fn eq(&self, other: &Amount64) -> bool {
        self.is_valid() && self.0 == *other
    }
}

impl PartialOrd<Amount64> for Price {
    fn partial_cmp(&self, other: &Amount64) -> Option<Ordering> {
        if !self.is_valid() {
            return None;
        }
        self.0.partial_cmp(other)
    }
}

impl FromStr for Price {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Amount64::from_str(s).map(Price)
    }
}

impl fmt::Display for Price {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if *self == Self::MARKET {
            return write!(f, "MARKET");
        }
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    mod amount64 {
        use super::*;

        #[test]
        fn from_f64() {
            let amt = Amount64::from_f64(12.1).expect("should be Some");
            assert_eq!(amt.0, 12__100_000_000);
        }

        #[test]
        fn mul() {
            let num1 = Amount64::from_int_dec(5, 0).expect("should be Some");
            let num2 = Amount64::from_int_dec(2, 0).expect("should be Some");
            assert_eq!(num1 * num2, Amount64::from_int_dec(10, 0).expect("should be Some"));

            let num1 = Amount64::from_int_dec(5, 0).expect("should be Some");
            let num2 = Amount64::from_int_dec(1, 0).expect("should be Some");
            assert_eq!(num1 * num2, Amount64::from_int_dec(5, 0).expect("should be Some"));

            let num1 = Amount64::from_int_dec(5, 0).expect("should be Some");
            let num2 = Amount64::from_int_dec(10, 0).expect("should be Some");
            assert_eq!(num1 * num2, Amount64::from_int_dec(50, 0).expect("should be Some"));

            let num1 = Amount64::from_int_dec(5, 0).expect("should be Some");
            let num2 = Amount64::from_int_dec(0, 5).expect("should be Some");
            assert_eq!(num1 * num2, Amount64::from_int_dec(2, 5).expect("should be Some"));

            let num1 = Amount64::from_int_dec(0, 5).expect("should be Some");
            let num2 = Amount64::from_int_dec(0, 5).expect("should be Some");
            assert_eq!(num1 * num2, Amount64::from_int_dec(0, 25).expect("should be Some"));
        }

        #[test]
        fn div() {
            let num = Amount64::from_int_dec(5, 0).expect("should be Some");
            let den = Amount64::from_int_dec(2, 0).expect("should be Some");
            assert_eq!(num / den, Amount64::from_int_dec(2, 5).expect("should be Some"));

            let num = Amount64::from_int_dec(5, 0).expect("should be Some");
            let den = Amount64::from_int_dec(1, 0).expect("should be Some");
            assert_eq!(num / den, Amount64::from_int_dec(5, 0).expect("should be Some"));

            let num = Amount64::from_int_dec(5, 0).expect("should be Some");
            let den = Amount64::from_int_dec(10, 0).expect("should be Some");
            assert_eq!(num / den, Amount64::from_int_dec(0, 5).expect("should be Some"));

            let num = Amount64::from_int_dec(5, 0).expect("should be Some");
            let den = Amount64::from_int_dec(0, 5).expect("should be Some");
            assert_eq!(num / den, Amount64::from_int_dec(10, 0).expect("should be Some"));
        }
    }

    mod amount128 {
        use super::*;
        type Amount = Amount128;

        /*
        #[test]
        fn from_f64() {
            let amt = Amount::from_f64(12.1).expect("should be Some");
            assert_eq!(amt.0, 12__100_000_000_000_000_000);
        }
        */

        #[test]
        fn format() {
            let amt = Amount::from_str("12.12").expect("should be Ok");
            assert_eq!(
                amt.format(FormatOpts {
                    ..Default::default()
                }),
                "12.12",
            );
            assert_eq!(
                amt.format(FormatOpts {
                    width: Some(1),
                    precision: Some(3),
                    ..Default::default()
                }),
                "12.120",
            );
            assert_eq!(
                amt.format(FormatOpts {
                    width: Some(3),
                    precision: Some(1),
                    ..Default::default()
                }),
                "012.1",
            );

            let amt = Amount::from_str("12.55").expect("should be Ok");
            assert_eq!(
                amt.format(FormatOpts {
                    ..Default::default()
                }),
                "12.55",
            );
            assert_eq!(
                amt.format(FormatOpts {
                    width: Some(1),
                    precision: Some(3),
                    ..Default::default()
                }),
                "12.550",
            );
            assert_eq!(
                amt.format(FormatOpts {
                    width: Some(3),
                    precision: Some(1),
                    ..Default::default()
                }),
                "012.6",
            );
            assert_eq!(
                amt.format(FormatOpts {
                    precision: Some(0),
                    ..Default::default()
                }),
                "13",
            );
        }
    }
}
