use std::io;
use std::io::{Result, Write};

use compact_str::CompactString;
use exchange_types::{OrderRequest, OrderSide};
use rand::Rng;
use uuid::Uuid;

const N: usize = 10_000_000;

fn main() -> Result<()> {
    let mut rng = rand::thread_rng();
    let orders = (1..=N)
        .map(|_| match rng.gen_range(0..1_000) {
            0 => OrderRequest::Delete {
                order_id: Uuid::new_v4(),
            },
            _ => OrderRequest::Create {
                account_id: Uuid::new_v4(),
                amount: rng.gen_range(100..10_000).into(),
                order_id: Uuid::new_v4(),
                pair: CompactString::new_inline("BTC/USDC"),
                limit_price: rng.gen_range(100..10_000).into(),
                side: match rng.gen_range(0..2) {
                    0 => OrderSide::Ask,
                    _ => OrderSide::Bid,
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
