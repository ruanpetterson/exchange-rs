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
cargo run --release -- < orders.json
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
