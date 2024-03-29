use std::error::Error;
use std::fs;
use std::fs::File;
use std::future::Future;
use std::io;
use std::io::{stdin, BufRead, BufReader};
use std::path::PathBuf;
use std::pin::Pin;
use std::thread;
use std::time::Instant;

use anyhow::Result;
use clap::Parser;
use compact_str::CompactString;
use either::Either;
use exchange_algo::Orderbook;
use exchange_core::{Asset, ExchangeExt};
use exchange_rt::{ConnectorError, Request, Runtime};
use futures::TryFutureExt;
use owo_colors::OwoColorize;
use serde::de::DeserializeOwned;
use tap::{Pipe, TapFallible as _};

#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

#[derive(Parser)]
#[clap(author, version, about)]
struct Args {
    #[clap(short, long, default_value = "BTC/USDC")]
    pair: CompactString,
    #[clap(short, long, parse(from_str), help = "Orders source")]
    input: Option<InputType>,
    #[clap(
        short,
        long,
        parse(from_str),
        help = "Orderbook events destination"
    )]
    output: Option<Output>,
}

impl<O> exchange_rt::Read for Input<O>
where
    O: Asset + Send + 'static,
    <O as Asset>::OrderId: std::marker::Send,
{
    type Request = Request<O>;

    #[inline]
    fn recv(
        &mut self,
    ) -> Pin<
        Box<
            dyn Future<
                    Output = Result<
                        Self::Request,
                        Box<dyn Error + Send + Sync + 'static>,
                    >,
                > + Send
                + '_,
        >,
    > {
        let fut = self
            .rx
            .recv_async()
            .map_err(Box::<dyn Error + Send + Sync + 'static>::from)
            .map_err(ConnectorError::Other)
            .map_err(Into::into);

        Box::pin(fut)
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let mut args = Args::parse();

    let orderbook = Orderbook::new();
    let engine = Runtime::new(&args.pair, orderbook).with_connector(
        "stdin_or_file",
        Input::from(args.input.take().unwrap_or_default()),
    )?;

    let begin = Instant::now();
    let engine = engine.run().await?;
    let elapsed = begin.elapsed();

    let (ask_length, bid_length) = engine.orderbook().len();

    eprintln!(
        "{:>12} {} request(s) in {:.2}s",
        "Total".bold().green(),
        engine.processed(),
        elapsed.as_secs_f64(),
    );
    eprintln!(
        "{:>12} {:.2} request/s",
        "Average".bold().green(),
        engine.processed() as f64 / elapsed.as_secs_f64(),
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
        Output::File(_path) => unimplemented!(),
    };

    Ok(())
}

struct Input<O: Asset> {
    rx: flume::Receiver<Request<O>>,
}

impl<O> From<InputType> for Input<O>
where
    O: Asset + DeserializeOwned + Send + Sync + 'static,
    <O as Asset>::OrderId: DeserializeOwned + Send + Sync + 'static,
{
    #[inline]
    fn from(type_: InputType) -> Self {
        let (tx, rx) = flume::bounded(128 * 1024);

        thread::spawn(move || -> Result<()> {
            let buf_read = match type_ {
                InputType::File(path) => Either::Left(File::open(path)?),
                InputType::Stdin => Either::Right(stdin()),
            }
            .pipe(BufReader::new);

            for order in
                buf_read.lines().map_while(Result::ok).filter_map(|line| {
                    serde_json::from_str(&line)
                        .tap_err(|error| {
                            eprintln!("ERROR: {error}");
                        })
                        .ok()
                })
            {
                tx.send(order)?;
            }

            Ok(())
        });

        Self { rx }
    }
}

#[derive(Debug, Default)]
enum InputType {
    #[default]
    Stdin,
    File(PathBuf),
}

impl From<&str> for InputType {
    #[inline]
    fn from(s: &str) -> Self {
        Self::File(s.to_owned().into())
    }
}

impl io::Read for InputType {
    #[inline]
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        match self {
            Self::Stdin => io::stdin().lock().read(buf),
            Self::File(path) => fs::File::open(path)?.read(buf),
        }
    }
}

#[derive(Default)]
enum Output {
    #[default]
    Stdout,
    File(PathBuf),
}

impl From<&str> for Output {
    #[inline]
    fn from(s: &str) -> Self {
        Output::File(s.to_owned().into())
    }
}
