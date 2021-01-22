# Data fetcher

This application converts different external API requests into unified endpoind
to simplify usage in pallet side.

## Usage

before running this program, please acquire API key and secret from kylin network.

```bash
export KYLIN_API_KEY=<replace_me>
export KYLIN_API_SECRET=<replace_me>
cargo run
```

if no error here, the data proxy service will be started at 8080 port. Currenly only four queries are supported:

* liquidation_order_list
* bitmex_perpetual_contract_rate
* bitmex_large_order_list
* bitfinex_holdings_minutes

Please refer [Kylin Contract Data API document](https://docs-api.kylin.network/#contract-data-api) for respond detail.

Here is a sample response sent via [*httpie*](https://httpie.io/):

```bash
$ http post 127.0.0.1:8080/ name=bitfinex_holdings_minutes
```

