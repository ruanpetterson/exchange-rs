use std::path::PathBuf;
use std::time::Instant;

use anyhow::Result;
use clap::Parser;
use compact_str::CompactString;
use exchange_core::ExchangeExt;
use exchange_rt::Engine;
use owo_colors::OwoColorize;
use tap::Pipe;
use tokio::io::{AsyncBufReadExt as _, BufReader};
use tokio::sync::mpsc;
use tokio::task::JoinSet;

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

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let mut set = JoinSet::new();

    let (raw_tx, raw_rx) = flume::bounded(65_536);
    let (deserialized_tx, mut deserialized_rx) = mpsc::channel(65_536);

    set.spawn(async move {
        let reader = match &args.input.unwrap_or_default() {
            Input::File(path) => {
                let file = tokio::fs::File::open(path).await?;
                tokio_util::either::Either::Left(file)
            }
            Input::Stdin => {
                let stdin = tokio::io::stdin();
                tokio_util::either::Either::Right(stdin)
            }
        }
        .pipe(BufReader::new);

        let mut lines = reader.lines();
        while let Some(line) = lines.next_line().await? {
            raw_tx.send_async(line).await?;
        }

        Ok::<_, anyhow::Error>(())
    });

    set.spawn(async move {
        while let Ok(raw_order) = raw_rx.recv_async().await {
            let Ok(deserialized_order) = serde_json::from_str(&raw_order)
            else {
                continue;
            };

            deserialized_tx.send(deserialized_order).await?;
        }

        Ok::<_, anyhow::Error>(())
    });

    let mut engine = Engine::new(&args.pair);

    let mut i = 0.0f64;
    let begin = Instant::now();
    while let Some(order) = deserialized_rx.recv().await {
        if let Err(err) = engine.process(order) {
            eprintln!("something went wrong: {}", err);
        };
        i += 1.0;
    }
    let elapsed = begin.elapsed();
    let (ask_length, bid_length) = engine.orderbook().len();

    eprintln!(
        "{:>12} {} order(s) in {:.2}s",
        "Total".bold().green(),
        i.round() as i64,
        elapsed.as_secs_f64(),
    );
    eprintln!(
        "{:>12} {:.2} orders/s",
        "Average".bold().green(),
        i / elapsed.as_secs_f64(),
    );
    eprintln!();
    eprintln!("{}", " Orderbook info ".bold().white().on_black());
    if let Some((ask_price, bid_price)) = engine.orderbook().spread() {
        eprintln!("{}", "    Spread".bold());
        eprintln!("{:>8} {}", "Ask".bold().green(), ask_price);
        eprintln!("{:>8} {}", "Bid".bold().green(), bid_price);
    }
    eprintln!("{}", "    Length".bold());
    eprintln!("{:>8} {}", "Ask".bold().green(), ask_length);
    eprintln!("{:>8} {}", "Bid".bold().green(), bid_length);

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
