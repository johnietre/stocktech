package cob

import (
	"fmt"
	"time"
)

type Orders []*Order

type OrderSide byte

const (
	OrderSideInvalid OrderSide = 0
	OrderSideBuy     OrderSide = 1
	OrderSideSell    OrderSide = 2
)

type Order struct {
	Id             uint64
	Symbol         string
	Price          float64
	Qty            uint64
	avgFilledPrice float64
	qtyFilled      uint64
	Side           OrderSide
	Time           int64
	FilledTime     int64
}

func (o *Order) IsMarket() bool {
	return o.Price == 0.0
}

func (o *Order) IsFilled() bool {
	return o.qtyFilled == o.Qty
}

func (o *Order) qtyLeft() uint64 {
	return o.Qty - o.qtyFilled
}

func (o *Order) fillWith(price float64, qty uint64) uint64 {
	if left := o.qtyLeft(); left <= qty {
		o.avgFilledPrice
		o.qtyFilled = o.Qty
		return qty - left
	}
	return 0
}

type OrderBook struct {
	symbol string
	asks   *asks
	bids   *bids
}

// AddOrder adds an order to the order book. It returns the order back if it
// was filled, otherwise, returns nil. Any filled orders are also returned. The
// order should not be used after being passed to this function.
func (ob *OrderBook) AddOrder(order *Order) (*Order, Orders) {
	switch order.Side {
	case OrderSideBuy:
	case OrderSideSell:
		filledOrders := ob.bids.tryFill(order)
		if order.IsFilled() {
		}
	default:
		// TODO
		panic(fmt.Sprint("invalid order side: ", order.Side))
	}
	return nil, nil
}

func (ob *OrderBook) Symbol() string {
	return ob.symbol
}

type bids struct {
	entries []entry
}

func (bs *bids) tryFill(o *Order) Orders {
	if o.IsMarket() {
		newStart := 0
		for i, ent := range bs.entries {
			//
		}
		return nil
	}
	var filled Orders
	return filled
}

func (bs *bids) addOrder(o *Order) {
	//
}

type asks struct {
	entries []entry
}

type entry struct {
	price  float64
	orders []*Order
}
