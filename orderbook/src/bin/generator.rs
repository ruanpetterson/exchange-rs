use std::path::Path;
use std::{fs::File, io::Result};

use compact_str::{format_compact, CompactString};
use orderbook::engine::OrderRequest;
use orderbook_core::OrderSide;
use rand::Rng;

const N: usize = 7_500_000;

fn main() -> Result<()> {
    let path = Path::new("./orders.json");
    let file = File::options()
        .read(true)
        .write(true)
        .create(true)
        .truncate(true)
        .open(path)?;

    let mut rng = rand::thread_rng();
    let orders = (1..=N)
        .map(|i| match rng.gen_range(0..1000) {
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
        })
        .collect::<Vec<_>>();

    serde_json::to_writer(file, &orders)?;

    Ok(())
}
