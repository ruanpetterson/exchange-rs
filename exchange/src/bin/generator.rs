use std::io;
use std::io::{Result, Write};

use clap::Parser;
use compact_str::CompactString;
use crossbeam::channel::Sender;
use exchange_rt::Request;
use exchange_types::Order;
use rand::Rng;
use rayon::prelude::*;

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
        0 => Request::Delete {
            order_id: rng.gen(),
        },
        _ => Request::Create {
            pair: CompactString::new_inline("BTC/USDC"),
            order: rng.gen::<Order>(),
        },
    };

    let Ok(serialized_order) = serde_json::to_string(&order) else {
        return;
    };

    let _ = tx.send(serialized_order);
}
