use std::io::Read;
use std::io::Result;
use std::path::PathBuf;
use std::time::Instant;

use clap::Parser;
use compact_str::CompactString;

use orderbook::engine::Engine;
use orderbook::engine::OrderRequest;

#[derive(Parser)]
#[clap(author, version, about)]
struct Args {
    #[clap(short, long, default_value = "BTC/USDC")]
    pair: CompactString,
    #[clap(short, long, parse(from_str), help = "Orders source")]
    input: Option<Input>,
    #[clap(
        short,
        long,
        parse(from_str),
        help = "Orderbook events destination"
    )]
    output: Option<Output>,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let content = match &args.input.unwrap_or_default() {
        Input::File(path) => std::fs::read_to_string(&path)?,
        Input::Stdin => {
            let mut buffer = String::new();
            std::io::stdin().read_to_string(&mut buffer)?;
            buffer
        }
    };
    let orders: Vec<OrderRequest> = serde_json::from_str(&content)?;

    let mut engine = Engine::new(&args.pair);
    let mut events = Vec::with_capacity(1024);

    let mut i = 0.0f64;
    let begin = Instant::now();
    for order in orders {
        events.append(&mut engine.process(order));
        i += 1.0;
    }
    let end = Instant::now();

    let elapsed = end - begin;

    eprintln!("Elapsed time: {:.2}s", elapsed.as_secs_f64());
    eprintln!("Total:        {}", i.round() as i64);
    eprintln!("Average:      {:.2} orders/s", i / elapsed.as_secs_f64());

    match &args.output.unwrap_or_default() {
        Output::Stdout => {
            // TODO: impl serde feature
        }
        Output::File(..) => unimplemented!(),
    };

    Ok(())
}

#[derive(Debug, Default)]
enum Input {
    #[default]
    Stdin,
    File(PathBuf),
}

impl From<&str> for Input {
    fn from(s: &str) -> Self {
        Input::File(s.to_owned().into())
    }
}

#[derive(Default)]
enum Output {
    #[default]
    Stdout,
    File(PathBuf),
}

impl From<&str> for Output {
    fn from(s: &str) -> Self {
        Output::File(s.to_owned().into())
    }
}
