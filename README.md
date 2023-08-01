## Example Orderbook 

Usage:

```
orderbook 0.2.0

USAGE:
    orderbook [OPTIONS]

OPTIONS:
    -h, --help               Print help information
    -i, --input <INPUT>      Orders source
    -o, --output <OUTPUT>    Orderbook events destination
    -p, --pair <PAIR>        [default: BTC/USDC]
    -V, --version            Print version information
```

You can run:

```shell 
cargo r --release --bin generator | cargo r --release
```

The expected result is:

```
    Finished release [optimized] target(s) in 0.22s
     Running `target/release/orderbook`
    Finished release [optimized] target(s) in 0.24s
     Running `target/release/generator`
       Total 10000000 order(s) in 14.33s
     Average 697990.94 orders/s

 Orderbook info 
    Spread
     Ask 569000
     Bid 530100
    Length
     Ask 1086869
     Bid 1086200
```

Example JSON:

```json
[
    {
        "type_op":"CREATE",
        "account_id":"1",
        "amount":"0.00230",
        "order_id":"1",
        "pair":"BTC/USDC",
        "limit_price":"63500.00",
        "side":"SELL"
    },
    {
        "type_op":"CREATE",
        "account_id":"2",
        "amount":"0.00230",
        "order_id":"2",
        "pair":"BTC/USDC",
        "limit_price":"63500.00",
        "side":"BUY"
    }
]
```
