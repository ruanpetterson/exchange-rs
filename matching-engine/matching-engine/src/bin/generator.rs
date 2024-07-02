use std::io;
use std::io::BufWriter;
use std::io::Result;
use std::io::Write;
use std::sync::OnceLock;
use std::thread;

use arrayvec::ArrayVec;
use clap::Parser;
use compact_str::CompactString;
use crossbeam_channel::Sender;
use exchange_types::OrderRequest;
use exchange_types::OrderSide;
use rand::distributions::Bernoulli;
use rand::distributions::Distribution;
use rand::Rng;
use rust_decimal::Decimal;
use uuid::Uuid;

type Message = ArrayVec<u8, 512>;

#[derive(Parser)]
struct Args {
    #[clap(short = 'n', default_value_t = 10_000_000)]
    total: usize,
    #[clap(short = 'j', long = "jobs", default_value_t = num_cpus::get())]
    workers: usize,
}

fn main() -> Result<()> {
    let Args {
        total: jobs,
        workers,
    } = Args::parse();

    let (tx, rx) = crossbeam_channel::bounded::<Message>(1024 * 4);

    let workers = 1.max(workers - 1);
    for jobs_per_worker in fair_division(jobs, workers) {
        let tx = tx.clone();
        thread::spawn(move || {
            let mut rng = rand::thread_rng();
            for _ in 0..jobs_per_worker {
                worker(&tx, &mut rng);
            }
        });
    }

    drop(tx);

    let mut out = {
        let stdout = io::stdout().lock();
        BufWriter::new(stdout)
    };

    while let Ok(order) = rx.recv() {
        out.write_all(order.as_slice())?;
    }

    out.flush()?;

    Ok(())
}

thread_local! {
    static SIDE_DISTRIBUTION: OnceLock<Bernoulli> = const { OnceLock::new() };
}

#[inline(always)]
fn worker(tx: &Sender<Message>, rng: &mut rand::rngs::ThreadRng) {
    let mut buf = Message::new_const();

    let side_distribution = SIDE_DISTRIBUTION.with(|side_dist| {
        *side_dist.get_or_init(move || unsafe {
            Bernoulli::from_ratio(1, 2).unwrap_unchecked()
        })
    });

    let order = OrderRequest::Create {
        account_id: Uuid::from_bytes(rng.gen::<[u8; 16]>()),
        amount: Decimal::from(rng.gen_range(100..10_000)).into(),
        order_id: Uuid::from_bytes(rng.gen::<[u8; 16]>()),
        symbol: CompactString::new_inline("BTC/USDC"),
        limit_price: Decimal::from(rng.gen_range(100..10_000)).into(),
        side: match side_distribution.sample(rng) {
            true => OrderSide::Ask,
            false => OrderSide::Bid,
        },
    };

    let Ok(_) = serde_json::to_writer(&mut buf, &order) else {
        return;
    };

    buf.write_all(b"\n").unwrap();
    buf.flush().unwrap();

    tx.send(buf).unwrap();
}

#[inline(always)]
fn fair_division(jobs: usize, workers: usize) -> impl Iterator<Item = usize> {
    let jobs_per_worker = jobs / workers;
    let mut remaining = jobs % workers;

    (0..workers)
        .map(move |_| jobs_per_worker)
        .map(move |jobs_per_worker| {
            let i = (remaining > 0) as usize;
            remaining -= i;
            jobs_per_worker + i
        })
}
