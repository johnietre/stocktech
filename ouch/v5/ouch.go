package ouch

import (
  "github.com/johnietre/stocktech/common/nasdaq"
  utils "github.com/johnietre/utils/go"
)

type UserRefNum uint32

func NewUserRefNum(num uint32) UserRefNum {
  return UserRefNum(num)
}

func (urn UserRefNum) Incr() UserRefNum {
  return urn + 1
}

func (urn UserRefNum) Add(n uint32) UserRefNum {
  return urn + UserRefNum(n)
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

type Firm struct {
}

type TagValue struct {
  OptionValue OptionValue
}

func (tv TagValue) Length() byte {
  return byte(tv.OptionValue.size())
}

const MaxOptionalAppendageLen int = 1<<16 - 1

type OptionalAppendage struct {
  inner *utils.Slice[TagValue]
}

func NewOptionalAppendage(tvs []TagValue) (OptionalAppendage, bool) {
  if len(tvs) > MaxOptionalAppendageLen {
    return OptionalAppendage{}, false
  }
  return OptionalAppendage{inner: utils.NewSlice(tvs)}, true
}

// Push returns true if the value was successfully added.
func (oa OptionalAppendage) Push(tv TagValue) bool {
  if oa.Len() >= MaxOptionalAppendageLen {
    return false
  }
  oa.inner.PushBack(tv)
  return true
}

// Insert returns true if the value was successfully added.
func (oa OptionalAppendage) Insert(index int, tv TagValue) bool {
  if oa.Len() >= MaxOptionalAppendageLen {
    return false
  }
  oa.inner.Insert(index, tv)
  return true
}

// Pop removes the last value and returns it, if it exists.
func (oa OptionalAppendage) Pop() (TagValue, bool) {
  return oa.inner.PopBack()
}

// Remove removes the last value and returns it, if it exists.
func (oa OptionalAppendage) Remove(index int) (TagValue, bool) {
  return oa.inner.Remove(index)
}

// Len returns the length of the optional appendage.
func (oa OptionalAppendage) Len() int {
  return oa.inner.Len()
}

// Inner returns the inner slice of the optional appendage.
func (oa OptionalAppendage) Inner() []TagValue {
  return oa.inner.Data()
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
  Symbol nasdaq.Symbol
  Price nasdaq.Price
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
  Quantity uint32
  Price nasdaq.Price
  TimeInForce TimeInForce
  Display Display
  InterMarketSweepEligibility IMSE
  ClOrdId ClOrdId
  Appendage
}

func (ror ReplaceOrderRequest) Type() byte {
  return 'U'
}
