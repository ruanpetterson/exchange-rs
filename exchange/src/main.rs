use std::io::BufRead;
use std::io::BufReader;
use std::path::PathBuf;
use std::sync::mpsc;
use std::time::Instant;

use anyhow::Result;
use clap::Parser;
use compact_str::CompactString;
use exchange_core::ExchangeExt;
use exchange_rt::Engine;
use owo_colors::OwoColorize;

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
    #[clap(long, help = "Orderbook persistent storage")]
    path: Option<PathBuf>,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let (tx, rx) = mpsc::sync_channel(65_536);

    std::thread::spawn(move || -> Result<()> {
        let mut buf_read: Box<dyn BufRead> =
            match &args.input.unwrap_or_default() {
                Input::File(path) => {
                    let file = std::fs::File::open(path)?;
                    Box::new(BufReader::new(file))
                }
                Input::Stdin => {
                    let stdin = std::io::stdin();
                    Box::new(BufReader::new(stdin))
                }
            };

        let mut buf = String::with_capacity(4096);
        while buf_read.read_line(&mut buf).is_ok() {
            let order = serde_json::from_str(&buf);
            buf.clear();
            match order {
                Err(error) => {
                    if error.is_eof() {
                        break;
                    }

                    eprintln!("{error}");
                }
                Ok(order) => tx.send(order)?,
            }
        }

        Ok(())
    });

    let mut engine = if let Some(path) = &args.path {
        Engine::new(&args.pair).path(path)?
    } else {
        Engine::new(&args.pair)
    };

    let mut i = 0.0f64;
    let begin = Instant::now();
    while let Ok(order) = rx.recv() {
        if let Err(err) = engine.process(order) {
            eprintln!("something went wrong: {}", err);
        };
        i += 1.0;
    }
    let end = Instant::now();

    let elapsed = end - begin;
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
