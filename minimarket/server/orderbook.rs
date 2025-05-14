use crate::order::*;
use either::Either;
use std::collections::LinkedList;

pub struct OrderOutput {
    order: Either<u64, Order>,
    affected: Vec<Either<u64, Order>>,
}

#[derive(Clone, PartialEq, Debug)]
pub struct ConsolidatedOrderBook {
    bids: LinkedList<ConsolidatedOrder>,
    asks: LinkedList<ConsolidatedOrder>,
}

impl ConsolidatedOrderBook {
    pub const fn new() -> Self {
        Self {
            bids: LinkedList::new(),
            asks: LinkedList::new(),
        }
    }

    pub const fn bids(&self) -> &LinkedList<ConsolidatedOrder> {
        &self.bids
    }

    pub const fn asks(&self) -> &LinkedList<ConsolidatedOrder> {
        &self.asks
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct OrderBook {
    bids: LinkedList<Order>,
    asks: LinkedList<Order>,
}

impl OrderBook {
    pub const fn new() -> Self {
        Self {
            bids: LinkedList::new(),
            asks: LinkedList::new(),
        }
    }

    pub const fn bids(&self) -> &LinkedList<Order> {
        &self.bids
    }

    pub const fn asks(&self) -> &LinkedList<Order> {
        &self.asks
    }

    pub fn get_order(&self, id: u64) -> Option<&Order> {
        self.bids
            .iter()
            .chain(self.asks.iter())
            .find(|o| o.id == id)
    }

    pub fn consolidated(&self) -> ConsolidatedOrderBook {
        let mut book = ConsolidatedOrderBook::new();

        let mut prev_price = None;
        for bid in self.bids.iter() {
            if Some(bid.limit) == prev_price {
                let back = book.bids.back_mut().unwrap();
                back.qty += bid.qty;
            } else {
                if bid.is_filled() {
                    continue;
                }
                book.bids.push_back(bid.consolidated());
            }
            prev_price = Some(bid.limit);
        }

        prev_price = None;
        for ask in self.asks.iter() {
            if Some(ask.limit) == prev_price {
                let back = book.asks.back_mut().unwrap();
                back.qty += ask.qty;
            } else {
                if ask.is_filled() {
                    continue;
                }
                book.asks.push_back(ask.consolidated());
            }
            prev_price = Some(ask.limit);
        }
        book
    }

    pub fn add_order(&mut self, mut order: Order) -> OrderOutput {
        #[inline(always)]
        fn gt(x: crate::amount::Price, y: crate::amount::Price) -> bool {
            x > y
        }

        #[inline(always)]
        fn lt(x: crate::amount::Price, y: crate::amount::Price) -> bool {
            x < y
        }

        let mut out = OrderOutput {
            order: Either::Left(order.id),
            affected: Vec::new(),
        };

        let check = match order.side {
            OrderSide::Buy => gt,
            OrderSide::Sell => lt,
        };
        let mut cur = match order.side {
            OrderSide::Buy => self.asks.cursor_front_mut(),
            OrderSide::Sell => self.bids.cursor_front_mut(),
        };
        while let Some(o) = cur.current() {
            match (o.limit.is_market(), order.limit.is_market()) {
                (true, true) => {
                    cur.move_next();
                    continue;
                },
                (false, false) if check(o.limit, order.limit) => break,
                _ => (),
            }
            if order.qty_left() >= o.qty_left() {
                if order.limit.is_market() {
                    order.fill_with(o.limit, o.qty_left());
                    o.fill_with(o.limit, o.qty_left());
                } else {
                    order.fill_with(order.limit, o.qty_left());
                    o.fill_with(order.limit, o.qty_left());
                }
            } else {
                if order.limit.is_market() {
                    order.fill_with(o.limit, order.qty_left());
                    o.fill_with(o.limit, order.qty_left());
                } else {
                    order.fill_with(order.limit, order.qty_left());
                    o.fill_with(order.limit, order.qty_left());
                }
            }
            if o.is_filled() {
                out.affected.extend(cur.remove_current().map(Either::Right));
            }
            if order.is_filled() {
                break;
            }
            cur.move_next();
        }
        if order.is_filled() {
            out.order = Either::Right(order);
            return out;
        }

        let check = match order.side {
            OrderSide::Buy => lt,
            OrderSide::Sell => gt,
        };
        let mut cur = match order.side {
            OrderSide::Buy => self.bids.cursor_front_mut(),
            OrderSide::Sell => self.asks.cursor_front_mut(),
        };
        while let Some(bid) = cur.current() {
            if !bid.limit.is_market() {
                if order.limit.is_market() || check(bid.limit, order.limit) {
                    cur.insert_before(order);
                    return out;
                }
            }
            cur.move_next();
        }
        cur.insert_before(order);
        out
    }
}

/*
struct List<T>(Option<Box<ListItem<T>>>);

impl List {
    fn take(&mut self) -> Option<Box<ListItem<T>>> {
        self.0.take()
    }

    fn add_after(&mut self, val: T, f: impl Fn(&T, &T) -> bool) {
        let Some(head) = self.0.as_mut() else {
            self.0 = Some(ListItem::new_boxed(val));
            return;
        };
        head.add_after(val, f);
    }

    /*
    fn push_front_if(&mut self, value: T, f: impl Fn(&T, &T) -> bool) {
    }
    */
}

struct ListItem<T> {
    value: T,
    next: Option<Box<ListItem<T>>>,
}

impl<T> ListItem<T> {
    fn new_boxed(value: T) -> Box<Self> {
        Self::with_next_boxed(value, None)
    }

    fn with_next_boxed(value: T, next: Self) -> Box<Self> {
        Box::new(Self { value, next })
    }

    fn add_after(&mut self, value: T, f: impl Fn(&T, &T) -> bool) {
        let Some(next) = self.next.as_mut() else {
            self.next = Some(ListItem::new_boxed(value));
            return;
        };
        if f(&self.value, &value) {
            self.next = Self::with_next_boxed(value, self.next);
            return;
        }
        next.add_after(val, f);
    }
}
*/

#[cfg(test)]
mod test {
    use crate::amount::*;
    use super::*;

    fn new_order(side: OrderSide) -> Order {
        use std::sync::atomic::{AtomicU64, Ordering};
        static ID: AtomicU64 = AtomicU64::new(1);

        Order {
            id: ID.fetch_add(1, Ordering::Relaxed),
            side,
            symbol: Sym::default(),
            limit: Price::MARKET,
            qty: Amount64::from_f64(5.0).expect("bad amount"),
            created_at: 0,
            updated_at: 0,
            filled_at: 0,
            canceled_at: 0,
            filled_qty: Amount64::ZERO,
            avg_price: Price::ZERO,
        }
    }

    #[test]
    fn add_buy() {
        let mut book = OrderBook::new();

        let ids_limits = [
            (1, Price::MARKET),
            (2, "1.0".parse::<Price>().expect("bad price")),
            (3, "2.0".parse::<Price>().expect("bad price")),
            (4, "3.0".parse::<Price>().expect("bad price")),
            (5, "3.0".parse::<Price>().expect("bad price")),
            (6, "2.0".parse::<Price>().expect("bad price")),
            (7, "1.0".parse::<Price>().expect("bad price")),
            (8, Price::MARKET),
        ];
        let mut orders = jtutils::make_map!(btree);
        for (id, limit) in ids_limits {
            let order = Order {
                id,
                limit,
                ..new_order(OrderSide::Buy)
            };
            let out = book.add_order(order.clone());
            assert_eq!(out.order, Either::Left(id));
            assert_eq!(out.affected, Vec::new());
            orders.insert(id, order);
        }

        assert_eq!(
            book.bids,
            jtutils::make_linked_list![
                orders[&1].clone(),
                orders[&8].clone(),
                orders[&4].clone(),
                orders[&5].clone(),
                orders[&3].clone(),
                orders[&6].clone(),
                orders[&2].clone(),
                orders[&7].clone(),
            ],
        );
    }

    #[test]
    fn add_sell() {
        let mut book = OrderBook::new();

        let ids_limits = [
            (1, Price::MARKET),
            (2, "1.0".parse::<Price>().expect("bad price")),
            (3, "2.0".parse::<Price>().expect("bad price")),
            (4, "3.0".parse::<Price>().expect("bad price")),
            (5, "3.0".parse::<Price>().expect("bad price")),
            (6, "2.0".parse::<Price>().expect("bad price")),
            (7, "1.0".parse::<Price>().expect("bad price")),
            (8, Price::MARKET),
        ];
        let mut orders = jtutils::make_map!(btree);
        for (id, limit) in ids_limits {
            let order = Order {
                id,
                limit,
                ..new_order(OrderSide::Sell)
            };
            let out = book.add_order(order.clone());
            assert_eq!(out.order, Either::Left(id));
            assert_eq!(out.affected, Vec::new());
            orders.insert(id, order);
        }

        assert_eq!(
            book.asks,
            jtutils::make_linked_list![
                orders[&1].clone(),
                orders[&8].clone(),
                orders[&2].clone(),
                orders[&7].clone(),
                orders[&3].clone(),
                orders[&6].clone(),
                orders[&4].clone(),
                orders[&5].clone(),
            ],
        );
    }

    #[test]
    fn buy_market_sell() {
        let mut book = OrderBook::new();

        book.add_order(Order {
            id: 1,
            limit: Price::MARKET,
            qty: Amount64::from_int_dec(5, 0).expect("bad amount"),
            ..new_order(OrderSide::Buy)
        });
        book.add_order(Order {
            id: 2,
            limit: Price::from_int_dec(1, 0).expect("bad price"),
            qty: Amount64::from_int_dec(5, 0).expect("bad amount"),
            ..new_order(OrderSide::Buy)
        });
        book.add_order(Order {
            id: 3,
            limit: Price::from_int_dec(2, 0).expect("bad price"),
            qty: Amount64::from_int_dec(5, 0).expect("bad amount"),
            ..new_order(OrderSide::Buy)
        });
        book.add_order(Order {
            id: 4,
            limit: Price::from_int_dec(3, 0).expect("bad price"),
            qty: Amount64::from_int_dec(5, 0).expect("bad amount"),
            ..new_order(OrderSide::Buy)
        });

        let mut sell_order = Order {
            id: 5,
            limit: Price::MARKET,
            qty: Amount64::from_int_dec(20, 0).expect("bad amount"),
            ..new_order(OrderSide::Sell)
        };
        let out = book.add_order(sell_order.clone());
        assert_eq!(out.order, Either::Left(5));
        assert_eq!(out.affected.len(), 3);
        for e in out.affected.iter() {
            match e {
                Either::Left(id) => panic!("expected order, got ID {id}"),
                Either::Right(ref o) => {
                    if !o.is_filled() {
                        panic!("expected filled order: {o:?}");
                    }
                    if o.limit != o.avg_price {
                        panic!("limit != avg_price: {o:?}");
                    }
                }
            }
        }

        sell_order.filled_qty = Amount64::from_u64(15).expect("bad price");
        sell_order.avg_price = Price::from_int_dec(2, 0).expect("bad price");
        let got_order = book.get_order(5).expect("expected order with ID 5");
        assert_eq!(got_order, &sell_order);

        assert_eq!(
            book,
            OrderBook {
                asks: jtutils::make_linked_list![
                    sell_order.clone(),
                ],
                bids: jtutils::make_linked_list![
                    Order {
                        id: 1,
                        limit: Price::MARKET,
                        qty: Amount64::from_int_dec(5, 0).expect("bad amount"),
                        ..new_order(OrderSide::Buy)
                    },
                ],
            },
        );
    }

    #[test]
    fn sell_market_buy() {
        let mut book = OrderBook::new();

        book.add_order(Order {
            id: 1,
            limit: Price::MARKET,
            qty: Amount64::from_int_dec(5, 0).expect("bad amount"),
            ..new_order(OrderSide::Sell)
        });
        book.add_order(Order {
            id: 2,
            limit: Price::from_int_dec(1, 0).expect("bad price"),
            qty: Amount64::from_int_dec(5, 0).expect("bad amount"),
            ..new_order(OrderSide::Sell)
        });
        book.add_order(Order {
            id: 3,
            limit: Price::from_int_dec(2, 0).expect("bad price"),
            qty: Amount64::from_int_dec(5, 0).expect("bad amount"),
            ..new_order(OrderSide::Sell)
        });
        book.add_order(Order {
            id: 4,
            limit: Price::from_int_dec(3, 0).expect("bad price"),
            qty: Amount64::from_int_dec(5, 0).expect("bad amount"),
            ..new_order(OrderSide::Sell)
        });

        let mut buy_order = Order {
            id: 5,
            limit: Price::MARKET,
            qty: Amount64::from_int_dec(20, 0).expect("bad amount"),
            ..new_order(OrderSide::Buy)
        };
        let out = book.add_order(buy_order.clone());
        assert_eq!(out.order, Either::Left(5));
        assert_eq!(out.affected.len(), 3);
        for e in out.affected.iter() {
            match e {
                Either::Left(id) => panic!("expected order, got ID {id}"),
                Either::Right(ref o) => {
                    if !o.is_filled() {
                        panic!("expected filled order: {o:?}");
                    }
                    if o.limit != o.avg_price {
                        panic!("limit != avg_price: {o:?}");
                    }
                }
            }
        }

        buy_order.filled_qty = Amount64::from_u64(15).expect("bad price");
        buy_order.avg_price = Price::from_int_dec(2, 0).expect("bad price");
        let got_order = book.get_order(5).expect("expected order with ID 5");
        assert_eq!(got_order, &buy_order);

        assert_eq!(
            book,
            OrderBook {
                asks: jtutils::make_linked_list![
                    Order {
                        id: 1,
                        limit: Price::MARKET,
                        qty: Amount64::from_int_dec(5, 0).expect("bad amount"),
                        ..new_order(OrderSide::Sell)
                    },
                ],
                bids: jtutils::make_linked_list![
                    buy_order.clone(),
                ],
            },
        );
    }

    #[test]
    fn consilidated_bids() {
        let mut book = OrderBook::new();
        book.add_order(Order {
            limit: Price::MARKET,
            qty: Amount64::from_int_dec(5, 0).expect("bad amount"),
            ..new_order(OrderSide::Buy)
        });
        book.add_order(Order {
            limit: Price::MARKET,
            qty: Amount64::from_int_dec(5, 0).expect("bad amount"),
            ..new_order(OrderSide::Buy)
        });
        book.add_order(Order {
            limit: Price::MARKET,
            qty: Amount64::from_int_dec(5, 0).expect("bad amount"),
            ..new_order(OrderSide::Buy)
        });
        book.add_order(Order {
            limit: Price::from_int_dec(5, 0).expect("bad price"),
            qty: Amount64::from_int_dec(5, 0).expect("bad amount"),
            ..new_order(OrderSide::Buy)
        });
        book.add_order(Order {
            limit: Price::from_int_dec(5, 0).expect("bad price"),
            qty: Amount64::from_int_dec(5, 0).expect("bad amount"),
            ..new_order(OrderSide::Buy)
        });
        book.add_order(Order {
            limit: Price::from_int_dec(10, 0).expect("bad price"),
            qty: Amount64::from_int_dec(5, 0).expect("bad amount"),
            ..new_order(OrderSide::Buy)
        });
        assert_eq!(
            book.consolidated(),
            ConsolidatedOrderBook {
                bids: jtutils::make_linked_list![
                    ConsolidatedOrder {
                        symbol: new_order(OrderSide::Buy).symbol,
                        side: OrderSide::Buy,
                        limit: Price::MARKET,
                        qty: Amount64::from_int_dec(15, 0).expect("bad amount"),
                    },
                    ConsolidatedOrder {
                        symbol: new_order(OrderSide::Buy).symbol,
                        side: OrderSide::Buy,
                        limit: Price::from_int_dec(10, 0).expect("bad price"),
                        qty: Amount64::from_int_dec(5, 0).expect("bad amount"),
                    },
                    ConsolidatedOrder {
                        symbol: new_order(OrderSide::Buy).symbol,
                        side: OrderSide::Buy,
                        limit: Price::from_int_dec(5, 0).expect("bad price"),
                        qty: Amount64::from_int_dec(10, 0).expect("bad amount"),
                    },
                ],
                asks: LinkedList::new(),
            },
        );
    }

    #[test]
    fn consilidated_asks() {
        let mut book = OrderBook::new();
        book.add_order(Order {
            limit: Price::MARKET,
            qty: Amount64::from_int_dec(5, 0).expect("bad amount"),
            ..new_order(OrderSide::Sell)
        });
        book.add_order(Order {
            limit: Price::MARKET,
            qty: Amount64::from_int_dec(5, 0).expect("bad amount"),
            ..new_order(OrderSide::Sell)
        });
        book.add_order(Order {
            limit: Price::MARKET,
            qty: Amount64::from_int_dec(5, 0).expect("bad amount"),
            ..new_order(OrderSide::Sell)
        });
        book.add_order(Order {
            limit: Price::from_int_dec(5, 0).expect("bad price"),
            qty: Amount64::from_int_dec(5, 0).expect("bad amount"),
            ..new_order(OrderSide::Sell)
        });
        book.add_order(Order {
            limit: Price::from_int_dec(5, 0).expect("bad price"),
            qty: Amount64::from_int_dec(5, 0).expect("bad amount"),
            ..new_order(OrderSide::Sell)
        });
        book.add_order(Order {
            limit: Price::from_int_dec(10, 0).expect("bad price"),
            qty: Amount64::from_int_dec(5, 0).expect("bad amount"),
            ..new_order(OrderSide::Sell)
        });
        assert_eq!(
            book.consolidated(),
            ConsolidatedOrderBook {
                asks: jtutils::make_linked_list![
                    ConsolidatedOrder {
                        symbol: new_order(OrderSide::Sell).symbol,
                        side: OrderSide::Sell,
                        limit: Price::MARKET,
                        qty: Amount64::from_int_dec(15, 0).expect("bad amount"),
                    },
                    ConsolidatedOrder {
                        symbol: new_order(OrderSide::Sell).symbol,
                        side: OrderSide::Sell,
                        limit: Price::from_int_dec(5, 0).expect("bad price"),
                        qty: Amount64::from_int_dec(10, 0).expect("bad amount"),
                    },
                    ConsolidatedOrder {
                        symbol: new_order(OrderSide::Sell).symbol,
                        side: OrderSide::Sell,
                        limit: Price::from_int_dec(10, 0).expect("bad price"),
                        qty: Amount64::from_int_dec(5, 0).expect("bad amount"),
                    },
                ],
                bids: LinkedList::new(),
            },
        );
    }

    #[test]
    fn consilidated_bids_asks() {
        let mut book = OrderBook::new();

        book.add_order(Order {
            limit: Price::from_int_dec(5, 0).expect("bad price"),
            qty: Amount64::from_int_dec(5, 0).expect("bad amount"),
            ..new_order(OrderSide::Buy)
        });
        book.add_order(Order {
            limit: Price::from_int_dec(5, 0).expect("bad price"),
            qty: Amount64::from_int_dec(5, 0).expect("bad amount"),
            ..new_order(OrderSide::Buy)
        });
        book.add_order(Order {
            limit: Price::from_int_dec(10, 0).expect("bad price"),
            qty: Amount64::from_int_dec(5, 0).expect("bad amount"),
            ..new_order(OrderSide::Buy)
        });

        book.add_order(Order {
            limit: Price::from_int_dec(105, 0).expect("bad price"),
            qty: Amount64::from_int_dec(5, 0).expect("bad amount"),
            ..new_order(OrderSide::Sell)
        });
        book.add_order(Order {
            limit: Price::from_int_dec(105, 0).expect("bad price"),
            qty: Amount64::from_int_dec(5, 0).expect("bad amount"),
            ..new_order(OrderSide::Sell)
        });
        book.add_order(Order {
            limit: Price::from_int_dec(110, 0).expect("bad price"),
            qty: Amount64::from_int_dec(5, 0).expect("bad amount"),
            ..new_order(OrderSide::Sell)
        });

        assert_eq!(
            book.consolidated(),
            ConsolidatedOrderBook {
                bids: jtutils::make_linked_list![
                    ConsolidatedOrder {
                        symbol: new_order(OrderSide::Buy).symbol,
                        side: OrderSide::Buy,
                        limit: Price::from_int_dec(10, 0).expect("bad price"),
                        qty: Amount64::from_int_dec(5, 0).expect("bad amount"),
                    },
                    ConsolidatedOrder {
                        symbol: new_order(OrderSide::Buy).symbol,
                        side: OrderSide::Buy,
                        limit: Price::from_int_dec(5, 0).expect("bad price"),
                        qty: Amount64::from_int_dec(10, 0).expect("bad amount"),
                    },
                ],
                asks: jtutils::make_linked_list![
                    ConsolidatedOrder {
                        symbol: new_order(OrderSide::Sell).symbol,
                        side: OrderSide::Sell,
                        limit: Price::from_int_dec(105, 0).expect("bad price"),
                        qty: Amount64::from_int_dec(10, 0).expect("bad amount"),
                    },
                    ConsolidatedOrder {
                        symbol: new_order(OrderSide::Sell).symbol,
                        side: OrderSide::Sell,
                        limit: Price::from_int_dec(110, 0).expect("bad price"),
                        qty: Amount64::from_int_dec(5, 0).expect("bad amount"),
                    },
                ],
            },
        );
    }
}
