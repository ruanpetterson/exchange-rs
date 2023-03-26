use std::io::Result;
use std::path::Path;

use compact_str::{format_compact, CompactString};
use orderbook::engine::OrderRequest;
use orderbook_core::OrderSide;
use rand::Rng;

const N: usize = 7_500_000;

fn main() -> Result<()> {
    let mut rng = rand::thread_rng();
    let mut orders = Vec::with_capacity(N);

    for i in 1..=N {
        let order = match rng.gen_range(0..1000) {
            0 => OrderRequest::Delete {
                order_id: format_compact!("{}", rng.gen_range(1..=i as u64)),
            },
            _ => OrderRequest::Create {
                account_id: format_compact!("{}", rng.gen_range(1..10)),
                amount: rng.gen_range(1000..2000).into(),
                order_id: format_compact!("{}", i as u64),
                pair: CompactString::new_inline("BTC/USDC"),
                limit_price: rng.gen_range(1000..2000).into(),
                side: match rng.gen_range(0..2) {
                    0 => OrderSide::Ask,
                    _ => OrderSide::Bid,
                },
            },
        };

        orders.push(order);
    }

    let content = serde_json::to_string(&orders)?;
    let path = Path::new("./orders.json");

    std::fs::write(&path, content)?;

    Ok(())
}
