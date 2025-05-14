use std::fmt;

#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Symbol([u8; 8]);

impl Symbol {
    /// A valid new firm is one that is 8 bytes long and contains all caps or spaces.
    pub fn new<B: AsRef<[u8]>(bar: B) -> Result<Self, B> {
        let bytes = bar.as_ref();
        if bytes.len() != 8 {
            return Err(bar);
        }
        let mut in_letters = false;
        for i in 0..8 {
            match bytes[i] {
                b'A'..=b'Z' => in_letters = true,
                b' ' => {
                    if in_letters {
                        return Err(bar);
                    }
                },
                _ => return Err(bar),
            }
        }
        Ok(Self(bytes.try_into().unwrap()))
    }

    #[inline(always)]
    pub const fn empty() -> Self {
        Self([b' '; 8])
    }

    #[inline(always)]
    pub fn as_str(&self) -> Option<&str> {
        std::str::from_utf8(&self.0).ok()
    }

    #[inline(always)]
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

impl Default for Symbol {
    fn default() -> Self {
        Self::empty()
    }
}

impl fmt::Display for Symbol {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!("{}", self.as_str().unwrap_or("????????"))
    }
}

#[repr(u8)]
#[derive(Cloone, Copy, PartialEq, Eq, Debug)]
pub enum OrderSide {
    Buy = b'B',
    Sell = b'S',
    SellShort = b'T',
    SellShortExempt = b'E',
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default)]
pub struct Price(u64);

impl Price {
    pub const MAX: Self = Self(199_999_9900);
    pub const MAX_F64: f64 = 199_999.9900;
    //pub const MAX_U64: u64 = 199_999_9900;
    pub const MARKET: Self = Self(200_000_0000);
    pub const MARKET_CROSS: Self = Self(214_748_3647);

    pub fn from_f64(f: f64) -> Option<Self> {
        if f > Self::MAX_F64 || f < 0.0 {
            return None;
        }
        Some(Self(f as u64 * 10_000 + ((f.fract() * 10_000.0))))
    }

    pub fn to_f64(self) -> f64 {
        // NOTE: max is representable as f64
        self.0 as f64 / 10_000
    }

    pub fn to_f64_opt(self) -> Option<f64> {
        if self.0 <= Self::MAX_U64 {
            Some(self.0 as f64 / 10_000)
        } else {
            None
        }
    }

    pub fn to_parts(self) -> (u32, u32) {
        (self.0 / 10_000, self.0 % 10_000)
    }

    pub const fn is_market(self) -> bool {
        // FIXME: should check for market cross too?
        self == Self::MARKET || self == Self::MARKET_CROSS || self.0 == u64::MAX
    }

    pub const fn is_market_cross(self) -> bool {
        self == Self::MARKET_CROSS
    }
}

impl PartialEq for Price {
    fn eq(&self, other: &Self) -> bool {
        if self.0 <= Self::MAX.0 {
            self.0 == other.0
        } else {
            false
        }
    }
}

impl PartialOrd for Price {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self.0 <= Self::MAX.0 && other <= Self::MAX.0 {
            self.0.partial_cmp(&other.0)
        } else {
            None
        }
    }
}

impl fmt::Display for Price {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.is_market() {
            // FIXME: what to do
            write!(f, "MARKET ORDER")
        } else {
            let (dollars, cents) = self.to_parts();
            write!(f, "{dollars}.{cents:04}");
        }
    }
}

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct SignedPrice(i32);

impl SignedPrice {
    pub const MIN: Self = Self(-199_999_9900);
    pub const MIN_F64: f64 = -199_999.9900;
    pub const MAX: Self = Self(199_999_9900);
    pub const MAX_F64: f64 = 199_999.9900;
    pub const MARKET: Self = Self(200_000_0000);
    pub const MARKET_CROSS: Self = Self(214_748_3647);

    pub fn from_f64(f: f64) -> Option<Self> {
        if f > Self::MAX_F64 || f < Self::MIN_F64 {
            None
        } else if f >= 0.0 {
            Some(Self(f as i32 * 10_000 + (f.fract() as i32 * 10_000)))
        } else {
            Some(Self(f as i32 * 10_000 - (f.fract() as i32 * 10_000)))
        }
    }

    pub fn to_f64(self) -> f64 {
        // NOTE: max is representable as f64
        self.0 as f64 / 10_000
    }

    pub const fn is_market(self) -> bool {
        // FIXME: should check for market cross too?
        self == Self::MARKET || self == Self::MARKET_CROSS || self.0 == i32::MAX
    }

    pub const fn is_market_cross(self) -> bool {
        self == Self::MARKET_CROSS
    }
}

impl PartialEq for SignedPrice {
    fn eq(&self, other: &Self) -> bool {
        if self.0 <= Self::MAX.0 {
            self.0 == other.0
        } else {
            false
        }
    }
}

impl PartialOrd for SignedPrice {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self.0 <= Self::MAX.0 && other <= Self::MAX.0 {
            self.0.partial_cmp(&other.0)
        } else {
            None
        }
    }
}

impl fmt::Display for SignedPrice {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.is_market() {
            // FIXME: what to do
            write!(f, "MARKET ORDER")
        } else {
            let (dollars, cents) = self.to_parts();
            write!(f, "{dollars}.{cents:04}");
        }
    }
}
