use compact_str::{format_compact, CompactString};
use criterion::{
    black_box, criterion_group, criterion_main, BatchSize, Criterion,
};
use exchange_rt::Engine;
use exchange_types::{Request, Side};
use rand::Rng;

const PAIR: &'static str = "BENCH";

pub fn in_memory(c: &mut Criterion) {
    let mut rng = rand::thread_rng();

    let mut orders = (1..).map(|i| match rng.gen_range(0..1_000) {
        0 => Request::Delete {
            order_id: format_compact!("{}", rng.gen_range(1..=i as u64)),
        },
        _ => Request::Create {
            account_id: format_compact!("{}", rng.gen_range(1..100)),
            amount: rng.gen_range(100..10_000).into(),
            order_id: format_compact!("{}", i as u64),
            pair: CompactString::new_inline(PAIR),
            limit_price: rng.gen_range(100..10_000).into(),
            side: match rng.gen_range(0..2) {
                0 => Side::Ask,
                _ => Side::Bid,
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
