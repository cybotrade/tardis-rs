use async_stream::stream;
use futures_util::{SinkExt, Stream, StreamExt};
use tokio_tungstenite::{
    connect_async,
    tungstenite::{self, protocol::frame::coding::CloseCode},
};

use super::{Message, ReplayNormalizedRequestOptions};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Options cannot be empty")]
    EmptyOptions,

    #[error("Failed to connect: {0}")]
    ConnectFailed(#[from] tungstenite::Error),

    #[error("Connection closed: {reason}")]
    ConnectionClosed { reason: String },

    #[error("Failed to deserialize message: {0}")]
    Deserialization(#[from] serde_json::Error),
}

pub struct Client {
    url: String,
}

impl Client {
    pub fn new(url: impl ToString) -> Self {
        Self {
            url: url.to_string(),
        }
    }

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
        let (mut ws_stream, _) = connect_async(url).await?;

        Ok(stream! {
            loop {
                match ws_stream.next().await {
                    Some(msg) => {
                        let msg = msg?;
                        match msg {
                            tungstenite::Message::Frame(_)
                            | tungstenite::Message::Binary(_)
                            | tungstenite::Message::Pong(_) => {}
                            tungstenite::Message::Ping(_) => {
                                tracing::debug!("Received PING frame");
                                ws_stream
                                    .send(tungstenite::Message::Pong(vec![]))
                                    .await
                                    .ok();
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
                                tracing::debug!("Received message from tardis: {}", msg);
                                yield Ok(serde_json::from_str::<Message>(&msg)?);
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
    async fn test_replay_normalized() {
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
            messages.push(msg);
        }

        assert_eq!(messages.len(), 23);
    }
}
