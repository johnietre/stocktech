// TODO: Swap beta for correlation
use crate::cbuf::CBuf;
use std::collections::BTreeMap;
use std::sync::{atomic::{AtomicI8, AtomicU64, Ordering}, Arc, Mutex};
use utils::atomic_value::NEAtomicArcValue as NEAAV;

#[derive(Clone)]
pub struct SimpleMA {
    value: f64,
    prev_prices: CBuf<f64>,
}

impl SimpleMA {
    pub fn new(prices: &[f64], period: usize) -> Self {
        let l = prices.len();
        assert_ne!(period, 0, "period must be positive");
        assert!(
            l >= period,
            "length of prices must be at least given period"
        );
        let prev_prices = CBuf::from_iter(prices.iter().skip(l - period).copied());
        let value = prev_prices.iter().sum::<f64>() / l as f64;
        Self { value, prev_prices }
    }

    pub fn add_price(&mut self, price: f64) -> f64 {
        let old = self.prev_prices.push(price);
        self.value += (price - old) / self.period() as f64;
        self.value
    }

    #[inline(always)]
    pub fn value(&self) -> f64 {
        self.value
    }

    #[inline(always)]
    pub fn period(&self) -> usize {
        self.prev_prices.len()
    }
}

#[repr(i8)]
#[derive(PartialEq, Eq, Clone, Copy)]
pub enum Signal {
    Sell = 0,
    Hold = 1,
    Buy = 2,
}

impl Signal {
    pub const fn from_i8(i: i8) -> Self {
        match i {
            -1 => Signal::Sell,
            0 => Signal::Hold,
            1 => Signal::Buy,
            //_ => unreachable!("invalid number"),
            _ => unreachable!(),
        }
    }

    pub const fn as_f64(self) -> f64 {
        self as i8 as f64
    }
}

type SimpleMAC = SimpleMACrossover;

#[derive(Clone)]
pub struct SimpleMACrossover {
    value: Signal,
    short_value: f64,
    long_value: f64,
    prev_prices: CBuf<f64>,
    short_period: usize,
}

impl SimpleMACrossover {
    pub fn new(prices: &[f64], short_period: usize, long_period: usize) -> Self {
        let l = prices.len();
        assert!(
            short_period != 0 && long_period != 0,
            "periods must be positive"
        );
        assert!(
            long_period > short_period,
            "long period must be greater than short"
        );
        assert!(
            l >= long_period,
            "length of prices must be at least given periods"
        );
        let (sp, lp) = (short_period as f64, long_period as f64);
        let (mut short_sum, mut long_sum) = (0.0, 0.0);
        for i in 0..short_period {
            short_sum += prices[i];
            long_sum += prices[i];
        }
        let mut short_value = short_sum / sp;
        for i in short_period..long_period {
            let price = prices[i];
            short_value += (price - prices[i - short_period]) / sp;
            long_sum += price;
        }
        let mut long_value = long_sum / lp;
        for i in long_period..l {
            let price = prices[i];
            short_value += (price - prices[i - short_period]) / sp;
            long_value += (price - prices[i - long_period]) / lp;
        }
        let value = if short_value > long_value {
            Signal::Buy
        } else if short_value < long_value {
            Signal::Sell
        } else {
            Signal::Hold
        };
        let prev_prices = CBuf::from_iter(prices.iter().skip(l - long_period).copied());
        Self {
            value,
            short_value,
            long_value,
            prev_prices,
            short_period,
        }
    }

    pub fn add_price(&mut self, price: f64) -> Signal {
        let oldl = self.prev_prices.push(price);
        let olds = self.prev_prices[self.short_period];
        self.short_value += (price - olds) / self.short_period() as f64;
        self.long_value += (price - oldl) / self.long_period() as f64;
        self.value = if self.short_value > self.long_value {
            Signal::Buy
        } else if self.short_value < self.long_value {
            Signal::Sell
        } else {
            Signal::Hold
        };
        self.value
    }

    #[inline(always)]
    pub fn value(&self) -> Signal {
        self.value
    }

    #[inline(always)]
    pub fn short_period(&self) -> usize {
        self.short_period
    }

    #[inline(always)]
    pub fn long_period(&self) -> usize {
        self.prev_prices.len()
    }
}

type ExponentialMAC = ExponentialMACrossover;

#[derive(Clone)]
pub struct ExponentialMACrossover {
    value: Signal,
    short_value: f64,
    long_value: f64,
    short_period: usize,
    long_period: usize,
}

impl ExponentialMACrossover {
    pub fn new(prices: &[f64], short_period: usize, long_period: usize) -> Self {
        let l = prices.len();
        assert!(
            short_period != 0 && long_period != 0,
            "periods must be positive"
        );
        assert!(
            long_period > short_period,
            "long period must be greater than short"
        );
        assert!(
            l >= long_period,
            "length of prices must be at least given periods"
        );
        let (sp, lp) = (short_period as f64, long_period as f64);
        let (sfactor, lfactor) = (2.0 / (sp + 1.0), 2.0 / (lp + 1.0));
        let (olfs, olfl) = (1.0 - sfactor, 1.0 - lfactor);
        let (mut short_sum, mut long_sum) = (0.0, 0.0);
        for i in 0..short_period {
            short_sum += prices[i];
            long_sum += prices[i];
        }
        let mut short_value = short_sum / sp;
        for i in short_period..long_period {
            let price = prices[i];
            short_value = (price * sfactor) + (short_value * olfs);
            long_sum += price;
        }
        let mut long_value = long_sum / lp;
        for i in long_period..l {
            let price = prices[i];
            short_value = (price * sfactor) + (short_value * olfs);
            long_value = (price * lfactor) + (long_value * olfl);
        }
        let value = if short_value > long_value {
            Signal::Buy
        } else if short_value < long_value {
            Signal::Sell
        } else {
            Signal::Hold
        };
        Self {
            value,
            short_value,
            long_value,
            short_period,
            long_period,
        }
    }

    pub fn add_price(&mut self, price: f64) -> Signal {
        let sfactor = 2.0 / (self.short_period() as f64 + 1.0);
        let lfactor = 2.0 / (self.long_period() as f64 + 1.0);
        let (olfs, olfl) = (1.0 - sfactor, 1.0 - lfactor);
        self.short_value = (price * sfactor) + (self.short_value * olfs);
        self.long_value = (price * lfactor) + (self.long_value * olfl);
        self.value = if self.short_value > self.long_value {
            Signal::Buy
        } else if self.short_value < self.long_value {
            Signal::Sell
        } else {
            Signal::Hold
        };
        self.value
    }

    #[inline(always)]
    pub fn value(&self) -> Signal {
        self.value
    }

    #[inline(always)]
    pub fn short_period(&self) -> usize {
        self.short_period
    }

    #[inline(always)]
    pub fn long_period(&self) -> usize {
        self.long_period
    }
}

#[derive(Clone, Copy)]
pub struct BollingerBandValues {
    ma_value: f64,
    band_down: f64,
    band_up: f64,
}

pub type SimpleMABB = SimpleMABollingerBand;

#[derive(Clone)]
pub struct SimpleMABollingerBand {
    value: BollingerBandValues,
    prev_prices: CBuf<f64>,
    old_mean: f64,
    variance: f64,
}

impl SimpleMABollingerBand {
    pub fn new(prices: &[f64], period: usize) -> Self {
        let n = period as f64;
        let prev_prices = &prices[prices.len() - period..];
        let ma_value = prev_prices.iter().sum::<f64>() / n;
        let mean = prev_prices.iter().sum::<f64>() / n;
        let variance = prev_prices.iter().map(|&x| (x - mean).powi(2)).sum::<f64>() / n;
        let sd = variance.sqrt();
        Self {
            value: BollingerBandValues {
                ma_value: ma_value,
                band_down: ma_value - (2.0 * sd),
                band_up: ma_value - (2.0 * sd),
            },
            prev_prices: CBuf::from(prev_prices.to_vec()),
            old_mean: mean,
            variance,
        }
    }

    pub fn add_price(&mut self, price: f64) -> BollingerBandValues {
        let n = self.period() as f64;
        let old = self.prev_prices.push(price);
        // Calculate SMA
        let ma_value = self.value.ma_value + ((price - old) / self.period() as f64);
        let mean = self.old_mean + ((price - old) / n);
        self.variance += (price - old) * (price - mean + old - self.old_mean) / (n - 1.0);
        self.old_mean = mean;
        let sd = self.variance.sqrt();
        self.value = BollingerBandValues {
            ma_value,
            band_down: ma_value - (2.0 * sd),
            band_up: ma_value - (2.0 * sd),
        };
        self.value
    }

    #[inline(always)]
    pub fn value(&self) -> BollingerBandValues {
        self.value
    }

    #[inline(always)]
    pub fn period(&self) -> usize {
        self.prev_prices.len()
    }
}

#[derive(Clone)]
pub struct RSI {
    value: f64,
    period: usize,
    prev_price: f64,
    prev_gain: f64,
    prev_loss: f64,
}

impl RSI {
    pub fn new(prices: &[f64], period: usize) -> Self {
        let (l, perf) = (prices.len(), period as f64);
        assert_ne!(period, 0, "period must be positive");
        assert!(
            l > period,
            "length of prices must be at least one more than preiod"
        );
        let (p, g, lo) = prices.iter().take(period + 1).skip(1).fold(
            (prices[0], 0.0, 0.0),
            |(prev, g, l), &p| {
                if p > prev {
                    (p, g + (p - prev), l)
                } else {
                    (p, g, l + (prev - p))
                }
            },
        );
        let (ag, al) = (g / perf, lo / perf);
        let mut value = if al != 0.0 {
            100.0 - (100.0 / (1.0 + (ag / al)))
        } else {
            100.0
        };
        let plo = perf - 1.0;
        let (pp, pg, pl) =
            prices
                .iter()
                .skip(period + 1)
                .fold((p, ag, al), |(prev, pg, pl), &p| {
                    let (ag, al) = if p >= prev {
                        (((pg * plo) + (p - prev)) / perf, pl * plo / perf)
                    } else {
                        (pg * plo / perf, (pl * plo + (prev - p)) / perf)
                    };
                    value = if al != 0.0 {
                        100.0 - (100.0 / (1.0 + (ag / al)))
                    } else {
                        100.0
                    };
                    (p, ag, al)
                });
        Self {
            value,
            period,
            prev_price: pp,
            prev_gain: pg,
            prev_loss: pl,
        }
    }

    pub fn add_price(&mut self, price: f64) -> f64 {
        let perf = self.period() as f64;
        let plo = perf - 1.0;
        let (ag, al) = if price >= self.prev_price {
            (
                (self.prev_gain * plo + (price - self.prev_price)) / perf,
                self.prev_loss * plo / perf,
            )
        } else {
            (
                self.prev_gain * plo / perf,
                (self.prev_loss * plo + (self.prev_price - price)) / perf,
            )
        };
        self.value = if al != 0.0 {
            100.0 - (100.0 / (1.0 + (ag / al)))
        } else {
            100.0
        };
        self.value
    }

    #[inline(always)]
    pub fn value(&self) -> f64 {
        self.value
    }

    #[inline(always)]
    pub fn period(&self) -> usize {
        self.period
    }
}

#[derive(Clone, Copy)]
pub struct Corr {
    value: f64,
    x_mean: f64,
    x_var: f64,
    y_mean: f64,
    y_var: f64,
    cov: f64,
    n: usize,
}

impl Corr {
    pub fn new(xs: &[f64], ys: &[f64]) -> Self {
        assert!(xs.len() >= 2 && ys.len() >= 2, "must have at least 2 values for each");
        assert_eq!(xs.len(), ys.len(), "lengths must be the same");
        let n = ys.len() as f64;
        let (x_sum, y_sum) = xs
            .iter()
            .zip(ys.iter())
            .fold((0.0, 0.0), |(x_sum, y_sum), (x, y)| (x_sum + x, y_sum + y));
        let (x_mean, y_mean) = (x_sum / n, y_sum / n);
        let (mut cov_sum, mut x_var_sum, mut y_var_sum) = (0.0, 0.0, 0.0);
        for (x, y) in xs.into_iter().zip(ys) {
            cov_sum += (x - x_mean) * (y - y_mean);
            x_var_sum += (x - x_mean).powi(2);
            y_var_sum += (y - y_mean).powi(2);
        }
        let value = if x_var_sum != 0.0 && y_var_sum != 0.0 {
            cov_sum / (x_var_sum * y_var_sum).sqrt()
        } else {
            0.0
        };
        let df = n - 1.0;
        Self {
            value,
            x_mean,
            x_var: x_var_sum / df,
            y_mean,
            y_var: y_var_sum / df,
            cov: cov_sum / df,
            n: ys.len(),
        }

    }

    pub fn add_price(&mut self, x: f64, y: f64) -> f64 {
        let n = self.n as f64;
        let new_n = n + 1.0;
        let new_x_mean = (self.x_mean * n + x) / new_n;
        let new_x_var = ((n - 1.0) * self.x_var + (x - new_x_mean) * (x - self.x_mean)) / n;
        let new_y_mean = (self.y_mean * n + y) / new_n;
        let new_y_var = ((n - 1.0) * self.y_var + (y - new_y_mean) * (y - self.y_mean)) / n;
        let new_cov =
            (self.cov * (n - 1.0) + (new_n / n) * (x - new_x_mean) * (y - new_y_mean)) / n;
        self.value = if new_x_var != 0.0 && new_y_var != 0.0 {
            new_cov / (new_x_var * new_y_var).sqrt()
        } else {
            0.0
        };
        self.x_mean = new_x_mean;
        self.x_var = new_x_var;
        self.y_mean = new_y_mean;
        self.y_var = new_y_var;
        self.cov = new_cov;
        self.n += 1;
        self.value
    }

    #[inline(always)]
    pub fn value(&self) -> f64 {
        self.value
    }
}

#[repr(transparent)]
pub struct AtomicF64(AtomicU64);

impl AtomicF64 {
    pub fn new(v: f64) -> Self {
        Self(AtomicU64::new(v.to_bits()))
    }

    pub fn load(&self, order: Ordering) -> f64 {
        f64::from_bits(self.0.load(order))
    }

    pub fn store(&self, val: f64, order: Ordering) {
        self.0.store(val.to_bits(), order);
    }
}

pub struct AtomicSignal(AtomicI8);

impl AtomicSignal {
    pub const fn new(sig: Signal) -> Self {
        Self(AtomicI8::new(sig as _))
    }

    pub fn load(&self, order: Ordering) -> Signal {
        match self.0.load(order) {
            -1 => Signal::Sell,
            0 => Signal::Hold,
            1 => Signal::Buy,
            _ => unreachable!(),
        }
    }

    pub fn store(&self, sig: Signal, order: Ordering) {
        self.0.store(sig as _, order);
    }
}

pub type Sym = Arc<str>;

pub type SymPair = (Sym, Sym);

pub type SymNum = u16;

pub type SymNumPair = (SymNum, SymNum);

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct SymNumPair(u32);

impl SymNumPair {
    #[inline(always)]
    pub const fn new(s1: u16, s2: u16) -> SymNumPair {
        let (s1, s2) = (s1 as u32, s2 as u32);
        if s1 > s2 {
            Self(s1 << 16 | s2)
        } else {
            Self(s2 << 16 | s1)
        }
    }

    #[inline(always)]
    pub const fn sym_num1(self) -> u16 {
        self.0 as u16
    }

    #[inline(always)]
    pub const fn sym_num2(self) -> u16 {
        (self.0 >> 16) as u16
    }
}

pub struct Info {
    sym: Arc<str>,
    combo: Mutex<Combo>,
    price: AtomicF64,
    decision: AtomicSignal,
}

pub struct Combo {
    smac: SimpleMAC,
    emac: ExponentialMAC,
    bb: SimpleMABB,
    rsi: RSI,
}

#[derive(Clone, Copy)]
struct Decisions {
    smac: Signal,
    emac: Signal,
    bb: Signal,
    rsi: Signal,
    corrs: Signal,
}

// Price, Price, Corr
pub type PPC = (Arc<AtomicF64>, Arc<AtomicF64>, Mutex<Corr>);

pub type Corrs = NEAAV<BTreeMap<SymNumPair, PPC>>;

// Indexed by SymNum
pub type Infos = NEAAV<Vec<Info>>;

struct SymsAlgos {
    syms: RwLock<Vec<Option<Sym>>>,
    infos: Infos,
    corrs: Corrs,
}

impl SymsAlgos {
    pub fn get_sym(&self, sym_num: SymNum) -> Option<Sym> {
        self.syms
            .read()
            .unwrap()
            .get(sym_num as usize)?
            .map(Arc::clone)
    }

    pub fn add_sym(&self, sym: Sym) -> SymNum {
        let mut syms = self.syms.write().lock();
        for (i, osym) in syms.iter_mut().enumerate() {
            if osym.is_none() {
                *osym = Some(sym);
                return i as _;
            }
        }
        syms.push(Some(sym));
        syms.len() as u16 - 1
    }

    pub fn add_price(&self, sym: &Sym, price: f64) -> Option<Decisions> {
        // TODO: Adjust
        const BB_MARGIN: f64 = 0.03;
        const MIN_CORR: f64 = 0.5;
        const CORR_BOUND: f64 = 0.5;

        let infos = self.infos.load(Ordering::Relaxed);
        let info = infos.get(sym)?;
        info.price.store(price, Ordering::SeqCst);
        let mut combo = info.combo.lock().unwrap();
        let smac = combo.smac.add_price(price);
        let emac = combo.emac.add_price(price);
        let bb = match combo.bb.add_price(price) {
            bb if bb.ma_value <= bb.band_down * (1.0 + BB_MARGIN) => Signal::Buy,
            bb if bb.ma_value >= bb.band_up * (1.0 - BB_MARGIN) => Signal::Sell,
            _ => Signal::Hold,
        };
        let rsi = match combo.rsi.add_price(price) {
            v if v < 0.3 => Signal::Buy,
            v if v > 0.7 => Signal::Sell,
            _ => Signal::Hold,
        };
        drop(combo);

        let (mut corrs, mut n) = (0.0, 0);
        let corrs_map = self.corrs.load(Ordering::Relaxed);
        for info in infos.values() {
            let (s1, s2) = if &info.sym > sym {
                (Arc::clone(&sym), Arc::clone(&info.sym))
            } else if &info.sym < sym {
                (Arc::clone(&info.sym), Arc::clone(&sym))
            } else {
                continue;
            };
            let corr = if let Some((p1, p2, c)) = corrs_map.get(&(s1, s2)) {
                c.lock()
                    .unwrap()
                    .add_price(p1.load(Ordering::SeqCst), p2.load(Ordering::SeqCst))
            } else {
                continue;
            };
            if corr >= MIN_CORR || corr <= -MIN_CORR {
                corrs += info.decision.load(Ordering::SeqCst).as_f64() * corr;
                n += 1;
            }
        }
        let max = n as f64;
        let (lower, upper) = (-max * CORR_BOUND, max * CORR_BOUND);
        let corrs = if corrs > upper {
            Signal::Buy
        } else if corrs < lower {
            Signal::Sell
        } else {
            Signal::Hold
        };

        Some(Decisions {
            smac,
            emac,
            bb,
            rsi,
            corrs,
        })
    }
}
