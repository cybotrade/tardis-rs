use futures_util::StreamExt;
use tardis_rs::{
    machine::{Client, StreamNormalizedRequestOptions},
    Exchange,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    let client = Client::new(std::env::var("TARDIS_MACHINE_WS_URL").unwrap());

    let option = StreamNormalizedRequestOptions {
        exchange: Exchange::Bybit,
        symbols: Some(vec!["BTCUSDT".to_string()]),
        data_types: vec!["trade_bar_15m".to_string()],
        with_disconnect_messages: None,
        timeout_interval_ms: None,
    };

    let mut stream = Box::pin(
        client
            .stream_normalized(vec![option.clone()])
            .await
            .unwrap(),
    );

    loop {
        match stream.next().await {
            Some(Ok(message)) => {
                tracing::info!("{:?}", message)
            }
            Some(Err(e)) => {
                tracing::error!("Err: {}", e);
                stream = Box::pin(
                    client
                        .stream_normalized(vec![option.clone()])
                        .await
                        .unwrap(),
                );
            }
            None => {
                tracing::error!("Stream got to None, reconnecting");
                stream = Box::pin(
                    client
                        .stream_normalized(vec![option.clone()])
                        .await
                        .unwrap(),
                );
            }
        }
    }
}
