use std::io;
use std::io::Result;
use std::io::Write;

use clap::Parser;
use compact_str::CompactString;
use crossbeam::channel::Sender;
use exchange_types::OrderRequest;
use exchange_types::OrderSide;
use rand::Rng;
use rayon::prelude::*;
use uuid::Uuid;

#[derive(Parser)]
struct Args {
    #[clap(short = 'n', default_value_t = 10_000_000)]
    total: usize,
}

fn main() -> Result<()> {
    let Args { total } = Args::parse();

    let (tx, rx) = crossbeam::channel::bounded(1024);

    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(2)
        .build()
        .unwrap();

    pool.spawn(move || {
        (0..total).into_par_iter().for_each(move |_| {
            let tx = tx.clone();
            worker(tx);
        })
    });

    let mut stdout = io::stdout();
    while let Ok(order) = rx.recv() {
        writeln!(stdout, "{}", order)?;
    }

    Ok(())
}

fn worker(tx: Sender<String>) {
    let mut rng = rand::thread_rng();

    let order = match rng.gen_range(0..1_000) {
        0 => OrderRequest::Delete {
            order_id: Uuid::from_bytes(rng.gen::<[u8; 16]>()),
        },
        _ => OrderRequest::Create {
            account_id: Uuid::from_bytes(rng.gen::<[u8; 16]>()),
            amount: rng.gen_range(100..10_000).into(),
            order_id: Uuid::from_bytes(rng.gen::<[u8; 16]>()),
            pair: CompactString::new_inline("BTC/USDC"),
            limit_price: rng.gen_range(100..10_000).into(),
            side: match rng.gen_range(0..2) {
                0 => OrderSide::Ask,
                _ => OrderSide::Bid,
            },
        },
    };

    let Ok(serialized_order) = serde_json::to_string(&order) else {
        return;
    };

    let _ = tx.send(serialized_order);
}
