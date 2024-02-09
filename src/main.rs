use execution::{OrderBook, Side};

fn main() {
    let mut book = OrderBook::new();
    book.add(Side::Bid, 100, 10);
    book.add(Side::Ask, 101, 10);
    book.add(Side::Ask, 102, 10);
    book.add(Side::Bid, 99, 10);
    book.add(Side::Bid, 98, 10);
    book.add(Side::Ask, 103, 10);
    book.add(Side::Ask, 104, 10);
    let id = book.add(Side::Bid, 105, 10);
    println!("{:?}", book.cancel(id));
    let (bid, ask) = book.update_best_bid_ask();
    println!("Best bid: {:?}, best ask: {:?}", bid, ask);
    println!("Total bid quantity at {}: {:?}", bid, book.get_total_qty(Side::Bid, bid));
    println!("Total ask quantity at {}: {:?}", ask, book.get_total_qty(Side::Ask, ask));
}
