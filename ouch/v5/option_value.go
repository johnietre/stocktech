package ouch

import (
  "github.com/johnietre/stocktech/common/nasdaq"
)

type OptionValue interface {
  num() int
  size() int
}

type OptionValueSecondaryOrdRefNum uint64
func (OptionValueSecondaryOrdRefNum) num() int { return 1 }
func (OptionValueSecondaryOrdRefNum) size() int { return 8 }

type OptionValueFirm Firm
func (OptionValueFirm) num() int { return 2 }
func (OptionValueFirm) size() int { return 4 }

type OptionValueMinQty uint32
func (OptionValueMinQty) num() int { return 3 }
func (OptionValueMinQty) size() int { return 4 }

type OptionValueCustomerType CustomerType
func (OptionValueCustomerType) num() int { return 4 }
func (OptionValueCustomerType) size() int { return 1 }

type OptionValueMaxFloor uint32
func (OptionValueMaxFloor) num() int { return 5 }
func (OptionValueMaxFloor) size() int { return 4 }

type OptionValuePriceType PriceType
func (OptionValuePriceType) num() int { return 6 }
func (OptionValuePriceType) size() int { return 1 }

type OptionValuePegOffset nasdaq.SignedPrice
func (OptionValuePegOffset) num() int { return 7 }
func (OptionValuePegOffset) size() int { return 4 }

type OptionValueDiscretionPrice nasdaq.Price
func (OptionValueDiscretionPrice) num() int { return 9 }
func (OptionValueDiscretionPrice) size() int { return 8 }

type OptionValueDiscretionPriceType PriceType
func (OptionValueDiscretionPriceType) num() int { return 10 }
func (OptionValueDiscretionPriceType) size() int { return 1 }

type OptionValueDiscretionPegOffset nasdaq.SignedPrice
func (OptionValueDiscretionPegOffset) num() int { return 11 }
func (OptionValueDiscretionPegOffset) size() int { return 4 }

type OptionValuePostOnly PostOnly
func (OptionValuePostOnly) num() int { return 12 }
func (OptionValuePostOnly) size() int { return 1 }

type OptionValueRandomReserved uint32
func (OptionValueRandomReserved) num() int { return 13 }
func (OptionValueRandomReserved) size() int { return 4 }

// DEPRECATED
type OptionValueRoute uint32
func (OptionValueRoute) num() int { return 14 }
func (OptionValueRoute) size() int { return 4 }

type OptionValueExpireTime uint32
func (OptionValueExpireTime) num() int { return 15 }
func (OptionValueExpireTime) size() int { return 4 }

type OptionValueTradeNow TradeNow
func (OptionValueTradeNow) num() int { return 16 }
func (OptionValueTradeNow) size() int { return 1 }

type OptionValueHandleInst HandleInst
func (OptionValueHandleInst) num() int { return 17 }
func (OptionValueHandleInst) size() int { return 1 }

type OptionValueBboWeightIndicator BboWeightIndicator
func (OptionValueBboWeightIndicator) num() int { return 18 }
func (OptionValueBboWeightIndicator) size() int { return 1 }

type OptionValueDisplayQuantity uint32
func (OptionValueDisplayQuantity) num() int { return 22 }
func (OptionValueDisplayQuantity) size() int { return 4 }

type OptionValueDisplayPrice nasdaq.Price
func (OptionValueDisplayPrice) num() int { return 23 }
func (OptionValueDisplayPrice) size() int { return 8 }

type OptionValueGroupId uint16
func (OptionValueGroupId) num() int { return 24 }
func (OptionValueGroupId) size() int { return 2 }

type OptionValueSharesLocated SharesLocated
func (OptionValueSharesLocated) num() int { return 25 }
func (OptionValueSharesLocated) size() int { return 1 }

type OptionValueLocateBroker Broker
func (OptionValueLocateBroker) num() int { return 26 }
func (OptionValueLocateBroker) size() int { return 4 }

type OptionValueSide Side
func (OptionValueSide) num() int { return 27 }
func (OptionValueSide) size() int { return 1 }

type OptionValueUserRefIdx uint8
func (OptionValueUserRefIdx) num() int { return 28 }
func (OptionValueUserRefIdx) size() int { return 1 }
