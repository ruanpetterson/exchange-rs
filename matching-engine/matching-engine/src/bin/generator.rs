use std::io;
use std::io::BufWriter;
use std::io::Result;
use std::io::Write;
use std::mem;
use std::thread;

use arrayvec::ArrayVec;
use clap::Parser;
use compact_str::CompactString;
use crossbeam::channel::Sender;
use exchange_types::OrderRequest;
use exchange_types::OrderSide;
use rand::Rng;
use uuid::Uuid;

type Message = ArrayVec<u8, 256>;

#[derive(Parser)]
struct Args {
    #[clap(short = 'n', default_value_t = 10_000_000)]
    total: usize,
    #[clap(short = 'j', long = "jobs", default_value_t = num_cpus::get())]
    workers: usize,
}

fn main() -> Result<()> {
    let Args { total, workers } = Args::parse();

    let (tx, rx) = crossbeam::channel::bounded::<Message>(1024 * 4);

    let order_generator_workers = 1.max(workers - 1);
    for jobs in fair_division(total, order_generator_workers) {
        let tx = tx.clone();
        thread::spawn(move || {
            let mut rng = rand::thread_rng();
            let mut buf = BufWriter::new(ArrayVec::new_const());
            for _ in 0..jobs {
                worker(&mut buf, &tx, &mut rng);
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

#[inline(always)]
fn worker(
    buf: &mut BufWriter<Message>,
    tx: &Sender<Message>,
    rng: &mut rand::rngs::ThreadRng,
) {
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

    let Ok(_) = serde_json::to_writer(&mut *buf, &order) else {
        return;
    };

    buf.write_all(b"\n").unwrap();
    buf.flush().unwrap();

    tx.send(mem::replace(buf.get_mut(), Message::new_const()))
        .unwrap();
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
