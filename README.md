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
    -s, --symbol <SYMBOL>    [default: BTC/USDC]
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
        "type_op": "CREATE",
        "account_id": "d4f79484-fe48-41e9-9bc5-45bb4cfedaf4",
        "quantity": "0.00230",
        "order_id": "75637317-8d86-436e-93bc-befc4a4ed830",
        "symbol": "BTC/USDC",
        "limit_price": "63500.00",
        "side": "SELL"
    },
    {
        "type_op": "CREATE",
        "account_id": "e26cd32c-e41e-4131-8dbe-489b5804beb3",
        "quantity": "0.00230",
        "order_id": "b69c93dc-d9d0-438c-88f9-6f34f9af53a7",
        "symbol": "BTC/USDC",
        "limit_price": "63500.00",
        "side": "BUY"
    }
]
```
