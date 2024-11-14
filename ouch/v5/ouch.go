package ouch

import (
  "math"
)

const (
  MaxPriceFloat64 float64 = 199_999.9900
  maxPriceUint64 uint64 = 199_999_9900
)

type Symbol struct {
  inner [8]byte
}

// TODO

type Price struct {
  inner uint64
}

func PriceFromFloat64(f float64) (Price, bool) {
  if f > MaxPriceFloat64 || f < 0.0 {
    return Price{}, false
  }
  ip, fp := math.Modf(f)
  return Price{
    inner: uint64(ip) * 10_000 + uint64(fp * 10_000.0),
  }, true
}

func (p Price) ToFloat64() float64 {
  // TODO
  return float64(p.inner) / 10_000.0
}

func (p Price) ToFloat64Safe() (f float64, ok bool) {
  if self.inner <= MaxPriceFloat64 {
    f = float64(self.inner) / 10_000.0
  }
  return
}

func (p Price) ToParts() (uint64, uint64) {
  return p.inner / 10_000, p.inner % 10_000
}

func (p Price) IsMarket() bool {
  // TODO: check for market cross too?
  return p.inner == marketUint64 || p.inner == marketCrossUint64 || p.inner == math.MaxUint64
}

func (p Price) Eq(other Price) bool {
  return p.inner <= maxPriceUint64 && p.inner == other.inner
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
  return fmt.Sprintf("%d.%04f", dollars, cents)
}

type ClOrdId struct {
  inner [14]byte
}

func ClOrdIdFromBytes(b []byte) (coi ClOrdId, ok bool) {
  if len(b) != 14 {
    return
  }
  for i, b := range b {
    // FIXME: are there specific places spaces are/aren't allowed e.g., are
    // spaces only padding characers, can an ID be all spaces?
    if (b >= 'A' && b <= 'Z') || (b >= 'a' || b <= 'z') || b == ' ' {
      return
    }
    coi.inner[i] = b
  }
  return coi, true
}

func (coi ClOrdId) String() string {
  return string(coi.inner[:])
}

type Appendage struct {
    //AppendageLength uint16
    OptionalAppendage []TagValue
}

type OrderSide byte
const (
  OrderSideBuy OrderSide = 'B'
  OrderSideSell OrderSide = 'S'
  OrderSideSellShort OrderSide = 'T'
  OrderSideSellShortExempt OrderSide = 'E'
)

type TimeInForce byte
const (
  TimeInForceDay TimeInForce = '0'
  TimeInForceIOC TimeInForce = '3'
  TimeInForceGTX TimeInForce = '5'
  TimeInForceGTT TimeInForce = '6'
  TimeInForceAfterHours TimeInForce = 'E'
)

type Display byte
const (
  DisplayVisible Display = 'Y'
  DisplayHidden Display = 'N'
  DisplayAttributable Display = 'A'
)

type Capacity byte
const (
  CapacityAgency Capacity = 'A'
  CapacityPrincipal Capacity = 'P'
  CapacityRiskless Capacity = 'R'
  CapacityOther Capacity = 'O'
)

type InterMarketSweepEligibility byte
type IMSE = InterMarketSweepEligibility
const (
  IMSEEligible IMSE = 'Y'
  IMSENotEligible IMSE = 'N'
)

type CrossType byte
const (
  CrossTypeContinuousMarket CrossType = 'N'
  CrossTypeOpeningCross CrossType = 'O'
  CrossTypeClosingCross CrossType = 'C'
  CrossTypeHaltIpo CrossType = 'H'
  CrossTypeSupplemental CrossType = 'S'
  CrossTypeRetail CrossType = 'R'
  CrossTypeExtendedLife CrossType = 'E'
  CrossTypeAfterHoursClose CrossType = 'A'
)

type CustomerType byte
const (
  CustomerTypeRetailDesignatedOrder CustomerType = 'R'
  CustomerTypeNotRetailDesignated CustomerType = 'N'
)

type PriceType byte
const (
  PriceTypeLimit PriceType = 'L'
  PriceTypeMarketPeg PriceType = 'P'
  PriceTypeMidpointPeg PriceType = 'M'
  PriceTypePrimaryPeg PriceType = 'R'
  PriceTypeMarketMakerPeg PriceType = 'Q'
  PriceTypeMidpoint PriceType = 'm'
)

type PostOnly byte
const (
  PostOnlyPostOnly PostOnly = 'P'
  PostOnlyNo PostOnly = 'N'
)

type EnterOrder struct {
  UserRefNum UserRefNum
  Side OrderSide
  Quantity uint32
  Symbol Symbol
  Price Price
  TimeInForce TimeInForce
  Display Display
  Capacity Capacity
  InterMarketSweepEligibility IMSE
  CrossType CrossType
  ClOrdId ClOrdId
  Appendage
}

func (eo EnterOrder) Type() byte {
  return 'O'
}

type ReplaceOrderRequest struct {
  OrigUserRefNum UserRefNum
  UserRefNum UserRefNum
  Quantity u32
  Price Price
  TimeInForce TimeInForce
  Display Display
  InterMarketSweepEligibility IMSE
  ClOrdId ClOrdId
  Appendage
}

func (ror ReplaceOrderRequest) Type() byte {
  return 'U'
}
