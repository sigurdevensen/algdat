use rand::Rng;
use std::time::Instant;

fn main() {
    let mut rng = rand::rng();
    let n = 1000;

    let random_price_change: Vec<i32> = (0..n)
        .map(|_| rng.random_range(-100..100))
        .collect();

    let mut actual_price = vec![];
    actual_price.push(random_price_change[0] + 200);

    for i in 1..random_price_change.len() {
        actual_price.push(random_price_change[i] + actual_price[i - 1]);
    }

    let start = Instant::now();

    let mut best_price = actual_price[0];
    let mut best_profit = 0;
    let mut buy_day = 0;
    let mut sell_day = 0;
    let mut temp_buy_day = 0;

    for (i, &price) in actual_price.iter().enumerate() {
        if price - best_price > best_profit {
            best_profit = price - best_price;
            sell_day = i;
            buy_day = temp_buy_day;
        }
        if price < best_price {
            best_price = price;
            temp_buy_day = i;
        }
    }

    let elapsed = start.elapsed();

    println!("Best time to buy:  day {} (price {})", buy_day, actual_price[buy_day]);
    println!("Best time to sell: day {} (price {})", sell_day, actual_price[sell_day]);
    println!("Best profit: {}", best_profit);
    println!("Tid brukt: {:?}", elapsed);
}
