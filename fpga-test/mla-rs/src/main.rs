use std::collections::HashMap;
use std::sync::Arc;

fn main() {
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Decision {
    Sell = -1,
    Hold = 0,
    Buy = 1,
}

impl Decision {
    fn as_f64(self) -> f64 {
        self as i8 as _
    }
}

const N_INPUTS: usize = 5;
type Decisions = [Decision; N_INPUTS];
type Weights = [f64; N_INPUTS];

type Sym = Arc<str>;

struct SymWeights {
    weights: HashMap<Sym, Weights>,
}

impl SymWeights {
    fn decide(&self, sym: impl AsRef<str>, decisions: Decisions) -> Option<Decision> {
        const WEIGHT_CUTOFF: f64 = 0.33;
        let sym = sym.as_ref();
        let res = self.weights
            .get(sym)?
            .iter()
            .copied()
            .zip(decisions)
            .fold(0.0, |sum, (w, d)| sum + w * d.as_f64())
            / N_INPUTS as f64;
        Some(if res > WEIGHT_CUTOFF {
            Decision::Buy
        } else if res < -WEIGHT_CUTOFF {
            Decision::Sell
        } else {
            Decision::Hold
        })
    }

    fn update(&self, sym: impl AsRef<str>, correct: [bool; N_INPUTS]) -> Output<Weights> {
        let sym = sym.as_ref();
        let mut weights = self.weights.get(sym)?;
        for (i, b) in correct.into_iter().enumerate() {
            if b {
                weights[i] *= 1.1;
            } else {
                weights[i] *= 0.95;
            }
        }
    }
}
