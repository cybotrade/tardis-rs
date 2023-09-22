use crate::{Exchange, InstrumentInfo, Response};

/// A helper Result type.
pub type Result<T> = std::result::Result<T, Error>;

/// The error that could happen while sending / receiving requests.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// The error that could happen when sending a request to Tardis.
    #[error("Failed to send request: {0}")]
    Request(#[from] reqwest::Error),

    /// The error that could happen when deserializing the response from Tardis.
    #[error("Failed to deserialize message: {0}")]
    Deserialization(#[from] serde_json::Error),
}

/// The client for interacting with [Tardis API](https://docs.tardis.dev/api/http).
pub struct Client {
    base_url: String,
    api_key: String,
    client: reqwest::Client,
}

impl Client {
    /// Creates a new instance of [`Client`].
    pub fn new(api_key: impl ToString) -> Self {
        static USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));

        Self {
            base_url: "https://api.tardis.dev/v1".to_string(),
            api_key: api_key.to_string(),
            client: reqwest::Client::builder()
                .user_agent(USER_AGENT)
                .build()
                .unwrap(),
        }
    }

    /// Returns instrument info for a given exchange and symbol.
    /// See <https://docs.tardis.dev/api/instruments-metadata-api#single-instrument-info-endpoint>
    pub async fn single_instrument_info(
        &self,
        exchange: Exchange,
        symbol: String,
    ) -> Result<Response<InstrumentInfo>> {
        Ok(self
            .client
            .get(format!(
                "{}/instruments/{}/{}",
                &self.base_url,
                exchange.to_string(),
                symbol
            ))
            .bearer_auth(&self.api_key)
            .send()
            .await?
            .json::<Response<InstrumentInfo>>()
            .await?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_single_instrument_info() {
        let client = Client::new(std::env::var("TARDIS_API_KEY").unwrap());

        let resp = client
            .single_instrument_info(Exchange::Bybit, "BTCUSDT".to_string())
            .await;
        println!("resp: {:?}", resp);
    }
}
