use std::fmt;
use std::fs;
use std::io;
use std::io::BufRead;
use std::path::PathBuf;
use std::time::Instant;

use anyhow::Result;
use clap::Parser;
use compact_str::CompactString;
use exchange_core::ExchangeExt;
use matching_engine_rt::Engine;
use owo_colors::OwoColorize;

#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

#[derive(Debug, Parser)]
#[clap(author, version, about)]
struct Args {
    #[clap(short, long, default_value = "BTC/USDC")]
    symbol: CompactString,
    #[clap(
        short,
        long,
        default_value_t = Input::default(),
        help = "Orders source"
    )]
    input: Input,
    #[clap(
        short,
        long,
        default_value_t = Output::default(),
        help = "Orderbook events destination"
    )]
    output: Output,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let (tx, rx) = crossbeam::channel::bounded(128 * 1024);

    std::thread::spawn(move || -> Result<()> {
        let mut reader = io::BufReader::new(args.input);
        let mut buf = String::with_capacity(1024);
        while reader.read_line(&mut buf).is_ok() {
            let order = serde_json::from_str(&buf);
            match order {
                Ok(order) => tx.send(order)?,
                Err(error) if error.is_eof() => break,
                Err(error) => {
                    eprintln!("{error}");
                }
            }
            buf.clear();
        }

        Ok(())
    });

    let mut engine = Engine::new(&args.symbol);

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

    // TODO: use this as `io::Write` instead relying on `(e)println`s.
    match &args.output {
        Output::Stdout => {}
        Output::File(_path) => {}
    };

    Ok(())
}

#[derive(Clone, Debug, Default)]
enum Input {
    #[default]
    Stdin,
    File(PathBuf),
}

impl fmt::Display for Input {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Input::Stdin => "/dev/stdin".fmt(f),
            Input::File(path) => path.display().fmt(f),
        }
    }
}

impl From<&str> for Input {
    #[inline]
    fn from(s: &str) -> Self {
        match s {
            "stdin" | "/dev/stdin" => Input::Stdin,
            s => Input::File(s.to_owned().into()),
        }
    }
}

impl io::Read for Input {
    #[inline]
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        match self {
            Input::Stdin => io::stdin().lock().read(buf),
            Input::File(path) => fs::File::open(path)?.read(buf),
        }
    }
}

#[derive(Clone, Debug, Default)]
enum Output {
    #[default]
    Stdout,
    File(PathBuf),
}

impl fmt::Display for Output {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Output::Stdout => "/dev/stdout".fmt(f),
            Output::File(path) => path.display().fmt(f),
        }
    }
}

impl From<&str> for Output {
    #[inline]
    fn from(s: &str) -> Self {
        match s {
            "stdout" | "/dev/stdout" => Output::Stdout,
            s => Output::File(s.to_owned().into()),
        }
    }
}
