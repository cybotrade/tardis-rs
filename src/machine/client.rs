use std::time::Duration;

use crate::machine::StreamNormalizedRequestOptions;
use async_stream::stream;
use futures_util::{stream::SplitSink, SinkExt, Stream, StreamExt};
use serde::de::DeserializeOwned;
use tokio::net::TcpStream;
use tokio_tungstenite::{
    connect_async,
    tungstenite::{self, protocol::frame::coding::CloseCode},
    MaybeTlsStream, WebSocketStream,
};

use super::{Message, ReplayNormalizedRequestOptions};

/// A helper Result type.
pub type Result<T> = std::result::Result<T, Error>;

/// The error that could happen while interacting with Tardis Machine Server.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// The error that could happen when an empty options array was given.
    #[error("Options cannot be empty")]
    EmptyOptions,

    /// The error when failed to connect to Tardis' websocket connection.
    #[error("Failed to connect: {0}")]
    ConnectFailed(#[from] tungstenite::Error),

    /// The error when WS connection to the machine server got rejected.
    #[error("Connection rejected: {reason}")]
    ConnectRejected {
        /// The status code for the initial WS connection.
        status: tungstenite::http::StatusCode,
        /// The reason why the connection was rejected.
        reason: String,
    },

    /// The error where the websocket connection was closed unexpectedly by Tardis.
    #[error("Connection closed: {reason}")]
    ConnectionClosed {
        /// The reason why the connection was closed.
        reason: String,
    },

    /// The error that could happen when deserializing the response from Tardis.
    #[error("Failed to deserialize message: {0}")]
    Deserialization(#[from] serde_json::Error),
}

/// The client for connecting to [Tardis Machine Server](https://docs.tardis.dev/api/tardis-machine).
pub struct Client {
    url: String,
}

impl Client {
    /// Creates a new instance of [`Client`].
    pub fn new(url: impl ToString) -> Self {
        Self {
            url: url.to_string(),
        }
    }

    /// Replays [normalized](https://docs.tardis.dev/api/tardis-machine#normalized-data-types)
    /// historical market data for [data types](https://docs.tardis.dev/api/tardis-machine#replay-normalized-options-1)
    /// specified in options. See [supported data types](https://docs.tardis.dev/api/tardis-machine#normalized-data-types)
    /// which include normalized [trade](https://docs.tardis.dev/api/tardis-machine#trade),
    /// [order book change](https://docs.tardis.dev/api/tardis-machine#book_change),
    /// [customizable order book snapshots](https://docs.tardis.dev/api/tardis-machine#book_snapshot_-number_of_levels-_-snapshot_interval-time_unit), etc.
    pub async fn replay_normalized(
        &self,
        options: Vec<ReplayNormalizedRequestOptions>,
    ) -> Result<impl Stream<Item = Result<Message>>> {
        if options.len() == 0 {
            return Err(Error::EmptyOptions);
        }

        let options = serde_json::to_string(&options)?;
        let url = format!(
            "{}/ws-replay-normalized?options={}",
            &self.url,
            urlencoding::encode(&options)
        );

        tracing::info!("[replay_normalized] url to tardis {}", url);
        websocket_conn(&url).await
    }

    /// Streams [normalized](https://docs.tardis.dev/api/tardis-machine#normalized-data-types)
    /// real-time market data for [data types](https://docs.tardis.dev/api/tardis-machine#replay-normalized-options-1)
    /// specified in options. See [supported data types](https://docs.tardis.dev/api/tardis-machine#normalized-data-types)
    /// which include normalized [trade](https://docs.tardis.dev/api/tardis-machine#trade),
    /// [order book change](https://docs.tardis.dev/api/tardis-machine#book_change),
    /// [customizable order book snapshots](https://docs.tardis.dev/api/tardis-machine#book_snapshot_-number_of_levels-_-snapshot_interval-time_unit), etc.
    ///
    /// It doesn't requires API key as it connects directly to exchanges real-time WebSocket APIs
    /// and transparently restarts closed, broken or stale connections (open connections without
    /// data being send for specified amount of time).
    ///
    /// Provides consolidated real-time market data streaming functionality with options as
    /// an array - provides single consolidated real-time data stream for all exchanges specified
    /// in options array.
    pub async fn stream_normalized(
        &self,
        options: Vec<StreamNormalizedRequestOptions>,
    ) -> Result<impl Stream<Item = Result<Message>>> {
        if options.len() == 0 {
            return Err(Error::EmptyOptions);
        }

        let options = serde_json::to_string(&options)?;
        let url = format!(
            "{}/ws-stream-normalized?options={}",
            &self.url,
            urlencoding::encode(&options)
        );

        tracing::info!("[stream_normalized] url to tardis {}", url);
        websocket_conn(&url).await
    }
}

async fn websocket_conn<T>(url: &str) -> Result<impl Stream<Item = Result<T>>>
where
    T: DeserializeOwned,
{
    let (ws_stream, ws_resp) = connect_async(url).await?;

    // Return the error response if the status code is not 101.
    // (meaning the HTTP connection is not being upgraded to a WS connection)
    if ws_resp.status() != tungstenite::http::StatusCode::SWITCHING_PROTOCOLS {
        return match ws_resp.body() {
            Some(resp) => Err(Error::ConnectRejected {
                status: ws_resp.status(),
                reason: String::from_utf8_lossy(resp).to_string(),
            }),
            None => Err(Error::ConnectRejected {
                status: ws_resp.status(),
                reason: "Unknown reason".to_string(),
            }),
        };
    }

    Ok(stream! {
        let (writer, mut reader) = ws_stream.split();
        tokio::spawn(heartbeat(writer));

        loop {
            match reader.next().await {
                Some(msg) => {
                    let msg = msg?;
                    match msg {
                        tungstenite::Message::Frame(_)
                        | tungstenite::Message::Binary(_)
                        | tungstenite::Message::Pong(_) => {}
                        tungstenite::Message::Ping(_) => {
                            tracing::debug!("Received PING frame");
                            // ws_stream
                            //     .send(tungstenite::Message::Pong(vec![]))
                            //     .await
                            //     .ok();
                        }
                        tungstenite::Message::Close(frame) => {
                            if let Some(frame) = frame {
                                if frame.code != CloseCode::Normal {
                                    tracing::error!(
                                        "Connection closed abnormally: {}",
                                        frame.reason
                                    );
                                    yield Err(Error::ConnectionClosed { reason: frame.reason.to_string() })
                                }
                                tracing::debug!("Connection closed normally: {}", frame.reason);
                            }
                            break;
                        }
                        tungstenite::Message::Text(msg) => {
                            tracing::debug!("Received websocket message: {}", msg);
                            yield Ok(serde_json::from_str::<T>(&msg)?);
                        }
                    }
                }
                None => {
                    tracing::error!("Connection closed unexpectedly");
                    yield Err(Error::ConnectionClosed { reason: "Unknown reason".to_string() });
                    break;
                }
            }
        }
    })
}

async fn heartbeat(
    mut sender: SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, tungstenite::Message>,
) {
    // create an interval.
    let mut interval = tokio::time::interval(Duration::from_secs(10));

    loop {
        // wait for the interval to arrive.
        interval.tick().await;

        // create a copy of the retries count.
        let mut count = 3;
        // the duration to wait before each retry.
        let mut interval = tokio::time::interval(Duration::from_secs(1));

        // keep trying until we run out of count.
        while count > 0 {
            interval.tick().await;

            // send native ping frame.
            let _ = sender.send(tungstenite::Message::Ping(vec![]));

            count -= 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::Exchange;
    use chrono::{TimeZone, Utc};
    use futures_util::pin_mut;
    use tracing_test::traced_test;

    use super::*;

    #[tokio::test]
    #[traced_test]
    async fn test_replay_normalized_trade() {
        let client = Client::new(std::env::var("TARDIS_MACHINE_WS_URL").unwrap());

        let stream = client
            .replay_normalized(vec![ReplayNormalizedRequestOptions {
                exchange: Exchange::Bybit,
                symbols: Some(vec!["BTCUSDT".to_string()]),
                from: Utc.with_ymd_and_hms(2022, 10, 1, 0, 0, 0).unwrap(),
                to: Utc.with_ymd_and_hms(2022, 10, 2, 0, 0, 0).unwrap(),
                data_types: vec!["trade".to_string()],
                with_disconnect_messages: None,
            }])
            .await
            .unwrap();

        pin_mut!(stream);

        let mut messages = vec![];

        while let Some(msg) = stream.next().await {
            if messages.len() == 10 {
                break;
            }
            messages.push(msg.unwrap())
        }

        for message in messages {
            assert!(matches!(message, Message::Trade(_)))
        }
    }

    #[tokio::test]
    #[traced_test]
    async fn test_replay_normalized_book_change() {
        let client = Client::new(std::env::var("TARDIS_MACHINE_WS_URL").unwrap());

        let stream = client
            .replay_normalized(vec![ReplayNormalizedRequestOptions {
                exchange: Exchange::Bybit,
                symbols: Some(vec!["BTCUSDT".to_string()]),
                from: Utc.with_ymd_and_hms(2022, 10, 1, 0, 0, 0).unwrap(),
                to: Utc.with_ymd_and_hms(2022, 10, 2, 0, 0, 0).unwrap(),
                data_types: vec!["book_change".to_string()],
                with_disconnect_messages: None,
            }])
            .await
            .unwrap();

        pin_mut!(stream);

        let mut messages = vec![];

        while let Some(msg) = stream.next().await {
            if messages.len() == 10 {
                break;
            }
            messages.push(msg.unwrap())
        }

        for message in messages {
            assert!(matches!(message, Message::BookChange(_)))
        }
    }

    #[tokio::test]
    #[traced_test]
    async fn test_replay_normalized_derivative_ticker() {
        let client = Client::new(std::env::var("TARDIS_MACHINE_WS_URL").unwrap());

        let stream = client
            .replay_normalized(vec![ReplayNormalizedRequestOptions {
                exchange: Exchange::Bybit,
                symbols: Some(vec!["BTCUSDT".to_string()]),
                from: Utc.with_ymd_and_hms(2022, 10, 1, 0, 0, 0).unwrap(),
                to: Utc.with_ymd_and_hms(2022, 10, 2, 0, 0, 0).unwrap(),
                data_types: vec!["derivative_ticker".to_string()],
                with_disconnect_messages: None,
            }])
            .await
            .unwrap();

        pin_mut!(stream);

        let mut messages = vec![];

        while let Some(msg) = stream.next().await {
            if messages.len() == 10 {
                break;
            }
            messages.push(msg.unwrap())
        }

        for message in messages {
            assert!(matches!(message, Message::DerivativeTicker(_)))
        }
    }

    #[tokio::test]
    #[traced_test]
    async fn test_replay_normalized_book_snapshot() {
        let client = Client::new(std::env::var("TARDIS_MACHINE_WS_URL").unwrap());

        let stream = client
            .replay_normalized(vec![ReplayNormalizedRequestOptions {
                exchange: Exchange::Bybit,
                symbols: Some(vec!["BTCUSDT".to_string()]),
                from: Utc.with_ymd_and_hms(2022, 10, 1, 0, 0, 0).unwrap(),
                to: Utc.with_ymd_and_hms(2022, 10, 2, 0, 0, 0).unwrap(),
                data_types: vec!["book_snapshot_2_50ms".to_string()],
                with_disconnect_messages: None,
            }])
            .await
            .unwrap();

        pin_mut!(stream);

        let mut messages = vec![];

        while let Some(msg) = stream.next().await {
            if messages.len() == 10 {
                break;
            }
            messages.push(msg.unwrap())
        }

        for message in messages {
            assert!(matches!(message, Message::BookSnapshot(_)))
        }
    }

    #[tokio::test]
    #[traced_test]
    async fn test_replay_normalized_trade_bar() {
        let client = Client::new(std::env::var("TARDIS_MACHINE_WS_URL").unwrap());

        let stream = client
            .replay_normalized(vec![ReplayNormalizedRequestOptions {
                exchange: Exchange::Bybit,
                symbols: Some(vec!["BTCUSDT".to_string()]),
                from: Utc.with_ymd_and_hms(2022, 10, 1, 0, 0, 0).unwrap(),
                to: Utc.with_ymd_and_hms(2022, 10, 2, 0, 0, 0).unwrap(),
                data_types: vec!["trade_bar_60m".to_string()],
                with_disconnect_messages: None,
            }])
            .await
            .unwrap();

        pin_mut!(stream);

        let mut messages = vec![];

        while let Some(msg) = stream.next().await {
            if messages.len() == 10 {
                break;
            }
            messages.push(msg.unwrap())
        }

        for message in messages {
            assert!(matches!(message, Message::TradeBar(_)))
        }
    }

    #[tokio::test]
    #[traced_test]
    async fn test_stream_normalized_trade() {
        let client = Client::new(std::env::var("TARDIS_MACHINE_WS_URL").unwrap());

        let stream = client
            .stream_normalized(vec![StreamNormalizedRequestOptions {
                exchange: Exchange::Binance,
                symbols: Some(vec!["BTCUSDT".to_string()]),
                data_types: vec!["trade".to_string()],
                with_disconnect_messages: None,
                timeout_interval_ms: None,
            }])
            .await
            .unwrap();

        pin_mut!(stream);

        let mut messages = vec![];

        while let Some(msg) = stream.next().await {
            if messages.len() == 10 {
                break;
            }
            messages.push(msg.unwrap())
        }

        for message in messages {
            assert!(matches!(message, Message::Trade(_)))
        }
    }
}
