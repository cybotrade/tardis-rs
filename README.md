# Tardis-rs

API Clients (REST, WebSocket) for [Tardis.dev](https://tardis.dev).

`tardis-rs` allows you to easily replay historical market data and stream live market data through
Tardis.dev's API.


> [!WARNING]  
> NOTE: The feature `machine` must be enabled in order to interact with [Tardis Machine Server](https://docs.tardis.dev/api/tardis-machine).

# Table of contents

- [Quickstart](#quickstart)
- [Crate features](#crate-features)

## Quickstart

Cargo.toml

```toml
[package]
name = "example"
version = "0.1.0"
edition = "2021"

[dependencies]
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
tardis-rs = { version = "0.1", features = ["machine"] }
```

main.rs

```rust
use tardis_rs::{Exchange, machine::{Client, Message}};
use chrono::NaiveDate;

#[tokio::main]
async function main() {
    let client = Client::new(std::env::var("TARDIS_MACHINE_WS_URL").unwrap());

    let stream = client
    .replay_normalized(vec![ReplayNormalizedRequestOptions {
        exchange: Exchange::Bybit,
        symbols: Some(vec!["BTCUSDT".to_string()]),
        from: NaiveDate::from_ymd_opt(2022, 10, 1).unwrap(),
        to: NaiveDate::from_ymd_opt(2022, 10, 2).unwrap(),
        data_types: vec!["trade_bar_60m".to_string()],
        with_disconnect_messages: None,
    }])
    .await
    .unwrap();

    pin_mut!(stream);

    while let Some(msg) = stream.next().await {
        println!("Received trade bar: {:?}", message);
    }
}
```

## Crate features

To avoid compiling unused dependencies, tardis-rs gates certain features, some of
which are disabled by default:

| Feature    | Description                                                                                 |
|------------|---------------------------------------------------------------------------------------------|
| machine    | Enables the client for [Tardis Machine Server](https://docs.tardis.dev/api/tardis-machine). |
