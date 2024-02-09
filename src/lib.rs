use rand::Rng;
use std::collections::{BTreeMap, HashMap, VecDeque};

pub type Price = u64;

pub type OrderQty = u64;

#[derive(Debug, Clone, Copy, PartialOrd, Ord, PartialEq, Eq, Hash)]
pub struct OrderId(u64);

#[derive(Debug)]
pub enum Side {
    /// Buy side
    Bid,

    /// Sell side
    Ask,
}

#[derive(Debug)]
struct Order {
    /// Unique identifier for the order
    id: OrderId,

    /// Quantity of the order
    qty: OrderQty,
}

#[derive(Debug)]
struct HalfBook {
    /// Map of price to index in price_levels
    price_map: BTreeMap<Price, usize>,

    /// Vector of price levels, each level is a queue of orders
    price_levels: Vec<VecDeque<Order>>,
}

impl HalfBook {
    fn new() -> HalfBook {
        HalfBook {
            price_map: BTreeMap::new(),
            price_levels: Vec::with_capacity(50_000),
        }
    }

    /// Get the total quantity at a given price level
    ///
    /// # Arguments
    ///
    /// * `price` - The price level to get the total quantity for
    ///
    /// # Returns
    ///
    /// The total quantity at the given price level
    fn get_total_qty(&self, price: Price) -> OrderQty {
        self.price_levels[self.price_map[&price]]
            .iter()
            .map(|o| o.qty)
            .sum()
    }
}

#[derive(Debug, PartialEq)]
pub enum CancelResult {
    /// Order was not found
    NotFound,

    /// Order was successfully canceled
    Canceled,
}

#[derive(Debug)]
pub struct OrderBook {
    /// Bid side of the order book
    bids: HalfBook,

    /// Ask side of the order book
    asks: HalfBook,

    /// Best bid price
    best_bid: Price,

    /// Best ask price
    best_ask: Price,

    /// Map of order id to side and price level
    order_loc: HashMap<OrderId, (Side, usize)>,
}

impl OrderBook {
    pub fn new() -> OrderBook {
        OrderBook {
            best_bid: 0,
            best_ask: 0,
            bids: HalfBook::new(),
            asks: HalfBook::new(),
            order_loc: HashMap::new(),
        }
    }

    /// Get the total quantity at a given price level
    ///
    /// # Arguments
    ///
    /// * `side` - The side of the order book
    /// * `price` - The price level to get the total quantity for
    ///
    /// # Returns
    ///
    /// The total quantity at the given price level
    pub fn get_total_qty(&self, side: Side, price: Price) -> OrderQty {
        match side {
            Side::Bid => self.bids.get_total_qty(price),
            Side::Ask => self.asks.get_total_qty(price),
        }
    }

    /// Add an order to the order book
    ///
    /// # Arguments
    ///
    /// * `side` - The side of the order
    /// * `price` - The price of the order
    /// * `qty` - The quantity of the order
    ///
    /// # Returns
    ///
    /// The unique identifier for the order
    pub fn add(&mut self, side: Side, price: Price, qty: OrderQty) -> OrderId {
        let id = OrderId(rand::thread_rng().gen());
        let book = match side {
            Side::Ask => &mut self.asks,
            Side::Bid => &mut self.bids,
        };
        match book.price_map.get(&price) {
            Some(idx) => {
                self.order_loc.insert(id, (side, *idx));
                book.price_levels[*idx].push_back(Order { id, qty });
            }
            None => {
                self.order_loc.insert(id, (side, book.price_levels.len()));
                book.price_map.insert(price, book.price_levels.len());
                book.price_levels
                    .push(VecDeque::from(vec![Order { id, qty }]));
            }
        };
        id
    }

    /// Cancel an order
    ///
    /// # Arguments
    ///
    /// * `id` - The unique identifier of the order to cancel
    ///
    /// # Returns
    ///
    /// The result of the cancel operation
    pub fn cancel(&mut self, id: OrderId) -> CancelResult {
        match self.order_loc.remove(&id) {
            None => CancelResult::NotFound,
            Some((side, price)) => {
                match side {
                    Side::Bid => &mut self.bids,
                    Side::Ask => &mut self.asks,
                }
                .price_levels[price]
                    .retain(|o| o.id != id);
                CancelResult::Canceled
            }
        }
    }

    /// Update the best bid and ask prices
    ///
    /// This method should be called after any operation that modifies the order book
    /// to ensure that the best bid and ask prices are up to date
    ///
    /// # Returns
    ///
    /// A tuple containing the best bid and ask prices, respectively
    pub fn update_best_bid_ask(&mut self) -> (Price, Price) {
        for (price, idx) in self.asks.price_map.iter() {
            match self.asks.price_levels[*idx].is_empty() {
                false => {
                    self.best_ask = *price;
                    break;
                }
                true => continue,
            }
        }
        for (price, idx) in self.bids.price_map.iter().rev() {
            match self.bids.price_levels[*idx].is_empty() {
                false => {
                    self.best_bid = *price;
                    break;
                }
                true => continue,
            }
        }
        (self.best_bid, self.best_ask)
    }
}

// TODO: Implement the fill method

#[derive(Debug)]
enum OrderStatus {
    Unititialized,
    Created,
    Filled,
    PartiallyFilled,
}

#[derive(Debug)]
struct FillResult {
    remaining: u64,
    status: OrderStatus,
    orders: Vec<(u64, u64)>,
}

impl FillResult {
    fn new() -> Self {
        FillResult {
            orders: Vec::new(),
            remaining: u64::MAX,
            status: OrderStatus::Unititialized,
        }
    }

    fn avg_price(&self) -> f64 {
        let (total, quantity) = self.orders.iter().fold((0, 0), |(total, quantity), (price, qty)| {
            (total + price * qty, quantity + qty)
        });
        total as f64 / quantity as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_order_book() {
        let mut book = OrderBook::new();
        book.add(Side::Bid, 100, 10);
        book.add(Side::Ask, 101, 10);
        book.add(Side::Ask, 101, 10);
        book.add(Side::Ask, 102, 10);
        book.add(Side::Bid, 99, 10);
        book.add(Side::Bid, 98, 10);
        book.add(Side::Ask, 103, 10);
        book.add(Side::Ask, 104, 10);
        let id = book.add(Side::Bid, 105, 10);
        assert_eq!(book.cancel(id), CancelResult::Canceled);
        let (bid, ask) = book.update_best_bid_ask();
        assert_eq!(bid, 100);
        assert_eq!(ask, 101);
        assert_eq!(book.get_total_qty(Side::Bid, bid), 10);
        assert_eq!(book.get_total_qty(Side::Ask, ask), 20);
    }
}
