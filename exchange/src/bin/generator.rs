use std::io;
use std::io::{Result, Write};

use compact_str::{format_compact, CompactString};
use exchange_types::{Request, Side};
use rand::Rng;

const N: usize = 10_000_000;

fn main() -> Result<()> {
    let mut rng = rand::thread_rng();
    let orders = (1..=N)
        .map(|i| match rng.gen_range(0..1_000) {
            0 => Request::Delete {
                order_id: format_compact!("{}", rng.gen_range(1..=i as u64)),
            },
            _ => Request::Create {
                account_id: format_compact!("{}", rng.gen_range(1..100)),
                amount: rng.gen_range(100..10_000).into(),
                order_id: format_compact!("{}", i as u64),
                pair: CompactString::new_inline("BTC/USDC"),
                limit_price: rng.gen_range(100..10_000).into(),
                side: match rng.gen_range(0..2) {
                    0 => Side::Ask,
                    _ => Side::Bid,
                },
            },
        })
        .filter_map(|order| serde_json::to_string(&order).ok());

    let mut stdout = io::stdout();
    for order in orders {
        writeln!(stdout, "{}", order)?;
    }

    Ok(())
}
