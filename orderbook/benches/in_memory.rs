use compact_str::{format_compact, CompactString};
use criterion::{
    black_box, criterion_group, criterion_main, BatchSize, Criterion,
};
use orderbook_core::OrderSide;
use orderbook_rt::Engine;
use orderbook_types::OrderRequest;
use rand::Rng;

const PAIR: &'static str = "BENCH";

pub fn in_memory(c: &mut Criterion) {
    let mut rng = rand::thread_rng();

    let mut orders = (1..).map(|i| match rng.gen_range(0..1_000) {
        0 => OrderRequest::Delete {
            order_id: format_compact!("{}", rng.gen_range(1..=i as u64)),
        },
        _ => OrderRequest::Create {
            account_id: format_compact!("{}", rng.gen_range(1..100)),
            amount: rng.gen_range(100..10_000).into(),
            order_id: format_compact!("{}", i as u64),
            pair: CompactString::new_inline(PAIR),
            limit_price: rng.gen_range(100..10_000).into(),
            side: match rng.gen_range(0..2) {
                0 => OrderSide::Ask,
                _ => OrderSide::Bid,
            },
        },
    });

    c.bench_function("process", |b| {
        b.iter_batched(
            || Engine::new(PAIR),
            |mut engine| {
                let incoming_order = black_box(orders.next().unwrap());
                black_box(engine.process(incoming_order))
            },
            BatchSize::SmallInput,
        );
    });
}

criterion_group!(benches, in_memory);
criterion_main!(benches);
