package nasdaq

import (
  "fmt"
  "math"
)

const (
  MaxPriceFloat64 float64 = 199_999.9900
  maxPriceUint32 uint32 = 199_999_9900
  marketUint32 uint32 = 200_000_0000
  marketCrossUint32 uint32 = 214_748_3327

  maxPriceInt32 int32 = 199_999_9900
  marketInt32 int32 = 200_000_0000
  marketCrossInt32 int32 = 214_748_3327
)

type Price struct {
  inner uint32
}

func PriceFromFloat64(f float64) (Price, bool) {
  if f > MaxPriceFloat64 || f < 0.0 {
    return Price{}, false
  }
  ip, fp := math.Modf(f)
  return Price{
    inner: uint32(ip) * 10_000 + uint32(fp * 10_000.0),
  }, true
}

func (p Price) ToFloat64() float64 {
  // TODO
  return float64(p.inner) / 10_000.0
}

func (p Price) ToFloat64Safe() (f float64, ok bool) {
  if p.inner <= maxPriceUint32 {
    f = float64(p.inner) / 10_000.0
  }
  return
}

func (p Price) ToParts() (uint32, uint32) {
  return p.inner / 10_000, p.inner % 10_000
}

func (p Price) IsMarket() bool {
  // TODO: check for market cross too?
  return p.inner == marketUint32 || p.inner == marketCrossUint32 || p.inner == math.MaxUint32
}

func (p Price) Eq(other Price) bool {
  return p.inner <= maxPriceUint32 && p.inner == other.inner
}

func (p Price) Less(other Price) bool {
  return p.inner <= maxPriceUint32 && p.inner < other.inner
}

/*
func (p Price) Cmp(other Price) int {
  //
}
*/

func (p Price) String() string {
  if p.IsMarket() {
    // FIXME: what to do
    return "MARKET ORDER"
  }
  dollars, cents := p.ToParts()
  return fmt.Sprintf("%d.%.04f", dollars, float32(cents))
}

type SignedPrice struct {
  inner int32
}

func SignedPriceFromFloat64(f float64) (SignedPrice, bool) {
  if f > MaxPriceFloat64 || f < 0.0 {
    return SignedPrice{}, false
  }
  ip, fp := math.Modf(f)
  return SignedPrice{
    inner: int32(ip) * 10_000 + int32(fp * 10_000.0),
  }, true
}

func (p SignedPrice) ToFloat64() float64 {
  // TODO
  return float64(p.inner) / 10_000.0
}

func (p SignedPrice) ToFloat64Safe() (f float64, ok bool) {
  if p.inner <= maxPriceInt32 {
    f = float64(p.inner) / 10_000.0
  }
  return
}

func (p SignedPrice) ToParts() (int32, int32) {
  return p.inner / 10_000, p.inner % 10_000
}

func (p SignedPrice) IsMarket() bool {
  // TODO: check for market cross too?
  return p.inner == marketInt32 || p.inner == marketCrossInt32 || p.inner == math.MaxInt32
}

func (p SignedPrice) Eq(other SignedPrice) bool {
  return p.inner <= maxPriceInt32 && p.inner == other.inner
}

func (p SignedPrice) Less(other SignedPrice) bool {
  return p.inner <= maxPriceInt32 && p.inner < other.inner
}

/*
func (p SignedPrice) Cmp(other SignedPrice) int {
  //
}
*/

func (p SignedPrice) String() string {
  if p.IsMarket() {
    // FIXME: what to do
    return "MARKET ORDER"
  }
  dollars, cents := p.ToParts()
  return fmt.Sprintf("%d.%.04f", dollars, float32(cents))
}

type Symbol struct {
  inner [8]byte
}
