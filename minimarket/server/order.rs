use crate::amount::{Amount64, Price};
use std::error::Error as StdError;
use std::fmt;
use std::io::{self, prelude::*};
use std::mem;
use std::ops::Deref;

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum OrderSide {
    Buy = b'B',
    Sell = b'S',
}

impl OrderSide {
    fn from_u8(u: u8) -> Result<Self, u8> {
        match u {
            b'B' => Ok(OrderSide::Buy),
            b'S' => Ok(OrderSide::Sell),
            _ => Err(u),
        }
    }
}

pub type Sym = [u8; 8];

#[derive(Clone, PartialEq, Debug)]
pub struct ConsolidatedOrder {
    pub symbol: Sym,
    pub side: OrderSide,
    pub limit: Price,
    pub qty: Amount64,
}

#[derive(Clone, PartialEq, Debug)]
pub struct Order {
    pub id: u64,

    pub symbol: Sym,
    pub side: OrderSide,
    pub limit: Price,
    pub qty: Amount64,

    // Nanosecond-precision timestamps
    pub created_at: i64,
    pub updated_at: i64,
    pub filled_at: i64,
    pub canceled_at: i64,

    pub filled_qty: Amount64,
    pub avg_price: Price,
}

impl Order {
    const SIZE: usize =
        8 +
        mem::size_of::<Sym>() + mem::size_of::<OrderSide>() + 8 + 8 +
        8 + 8 + 8 + 8 +
        8 + 8;

    const fn empty() -> Self {
        Self {
            id: 0,

            symbol: [0u8; 8],
            side: OrderSide::Buy,
            limit: Price::MARKET,
            qty: Amount64::ZERO,

            created_at: 0,
            updated_at: 0,
            filled_at: 0,
            canceled_at: 0,

            filled_qty: Amount64::ZERO,
            avg_price: Price::MARKET,
        }
    }

    pub const fn consolidated(&self) -> ConsolidatedOrder {
        ConsolidatedOrder {
            symbol: self.symbol,
            side: self.side,
            limit: self.limit,
            qty: self.qty,
        }
    }

    pub fn fill_with(&mut self, price: Price, qty: Amount64) -> bool {
        let sum = self.avg_price.to_amount64() * self.filled_qty + price.to_amount64() * qty;
        let filled_qty = self.filled_qty + qty;
        let avg_price = sum / filled_qty;
        let Some(ap) = Price::from_amount(avg_price) else { 
            return false;
        };
        //println!("{}, {price}: {} {old_qty} {avg_price} {}", self.id, self.avg_price, new_qty);
        self.filled_qty = filled_qty;
        self.avg_price = ap;
        true
    }

    pub fn is_filled(&self) -> bool {
        self.qty == self.filled_qty
    }

    pub fn qty_left(&self) -> Amount64 {
        self.qty - self.filled_qty
    }
}

impl Ser for Order {
    fn ser_into<W: Write>(&self, w: &mut W) -> io::Result<()> {
        let mut buf = Vec::with_capacity(Self::SIZE);
        buf.extend_from_slice(&self.id.to_le_bytes());

        buf.extend_from_slice(&self.symbol);
        buf.push(self.side as u8);
        buf.extend_from_slice(&self.limit.to_bytes());
        buf.extend_from_slice(&self.qty.to_le_bytes());

        buf.extend_from_slice(&self.created_at.to_le_bytes());
        buf.extend_from_slice(&self.updated_at.to_le_bytes());
        buf.extend_from_slice(&self.filled_at.to_le_bytes());
        buf.extend_from_slice(&self.canceled_at.to_le_bytes());

        buf.extend_from_slice(&self.filled_qty.to_le_bytes());
        buf.extend_from_slice(&self.avg_price.to_bytes());
        debug_assert_eq!(buf.len(), Self::SIZE);

        w.write_all(&buf)
    }
}

impl Deser for Order {
    fn deser_from<R: Read>(r: &mut R) -> Result<Self, DeserError> {
        let mut x = Self::empty();
        x.deser_from_in_place(r)?;
        Ok(x)
    }

    fn deser_from_in_place<R: Read>(&mut self, r: &mut R) -> Result<(), DeserError> {
        let mut buf = vec![0u8; Self::SIZE];
        r.read_exact(&mut buf)?;
        let mut sr = SliceReader::new(&buf);
        self.id = sr
            .read_exact(mem::size_of_val(&self.id))
            .unwrap()
            .try_into()
            .map(u64::from_le_bytes)
            .map_err(DeserError::other)?;

        self.symbol = sr
            .read_exact(mem::size_of_val(&self.symbol))
            .unwrap()
            .try_into()
            .map_err(DeserError::other)?;
        self.side = OrderSide::from_u8(*sr.read_one().unwrap())
            .map_err(|u| DeserError::other(format!("received bad side byte: {u}")))?;
        self.limit = sr
            .read_exact(mem::size_of_val(&self.limit))
            .unwrap()
            .try_into()
            .map(Price::from_bytes)
            .map_err(DeserError::other)?;
        self.qty = sr
            .read_exact(mem::size_of_val(&self.qty))
            .unwrap()
            .try_into()
            .map(Amount64::from_le_bytes)
            .map_err(DeserError::other)?;

        self.created_at = sr
            .read_exact(mem::size_of_val(&self.created_at))
            .unwrap()
            .try_into()
            .map(i64::from_le_bytes)
            .map_err(DeserError::other)?;
        self.updated_at = sr
            .read_exact(mem::size_of_val(&self.updated_at))
            .unwrap()
            .try_into()
            .map(i64::from_le_bytes)
            .map_err(DeserError::other)?;
        self.filled_at = sr
            .read_exact(mem::size_of_val(&self.filled_at))
            .unwrap()
            .try_into()
            .map(i64::from_le_bytes)
            .map_err(DeserError::other)?;
        self.canceled_at = sr
            .read_exact(mem::size_of_val(&self.canceled_at))
            .unwrap()
            .try_into()
            .map(i64::from_le_bytes)
            .map_err(DeserError::other)?;

        self.filled_qty = sr
            .read_exact(mem::size_of_val(&self.filled_qty))
            .unwrap()
            .try_into()
            .map(Amount64::from_le_bytes)
            .map_err(DeserError::other)?;
        self.avg_price = sr
            .read_exact(mem::size_of_val(&self.avg_price))
            .unwrap()
            .try_into()
            .map(Price::from_bytes)
            .map_err(DeserError::other)?;

        Ok(())
    }
}

#[derive(Clone)]
struct SliceReader<'a, T>(&'a [T]);

impl<'a, T> SliceReader<'a, T> {
    const fn new(s: &'a [T]) -> Self {
        Self(s)
    }

    fn read_one(&mut self) -> Option<&'a T> {
        if self.len() == 0 {
            return None;
        }
        let t = &self.0[0];
        self.0 = &self.0[1..];
        Some(t)
    }

    fn read(&mut self, n: usize) -> &'a [T] {
        let n = self.len().min(n);
        let s = &self.0[..n];
        self.0 = &self.0[n..];
        s
    }

    fn read_exact(&mut self, n: usize) -> Option<&'a [T]> {
        if n > self.len() {
            return None;
        }
        let n = self.len().min(n);
        let s = &self.0[..n];
        self.0 = &self.0[n..];
        Some(s)
    }
}

impl<'a, T> Deref for SliceReader<'a, T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub trait Ser {
    fn ser_into<W: Write>(&self, w: &mut W) -> io::Result<()>;

    fn ser(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        self.ser_into(&mut buf).expect("bad serialization");
        buf
    }
}

pub trait Deser: Sized {
    fn deser_from<R: Read>(r: &mut R) -> Result<Self, DeserError>;

    fn deser(buf: &[u8]) -> Result<Self, DeserError> {
        Self::deser_from(&mut std::io::Cursor::new(buf))
    }

    fn deser_from_in_place<R: Read>(&mut self, r: &mut R) -> Result<(), DeserError> {
        *self = Self::deser_from(r)?;
        Ok(())
    }
}

#[derive(Debug)]
pub enum DeserError {
    Io(io::Error),
    Other(Box<dyn StdError>),
}

impl DeserError {
    pub fn other(e: impl Into<Box<dyn StdError>>) -> Self {
        DeserError::Other(e.into())
    }
}

impl From<io::Error> for DeserError {
    fn from(e: io::Error) -> Self {
        DeserError::Io(e)
    }
}

impl fmt::Display for DeserError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            DeserError::Io(ref e) => write!(f, "{e}"),
            DeserError::Other(ref e) => write!(f, "{e}"),
        }
    }
}

impl StdError for DeserError {}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_empty_order() {
        let order = Order::empty();
        let ser = order.ser();
        let dorder = Order::deser(&ser).expect("bad deser");
        assert_eq!(order, dorder);
    }
}
