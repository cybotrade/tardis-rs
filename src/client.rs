use crate::{Exchange, InstrumentInfo, Response};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Failed to send request: {0}")]
    Request(#[from] reqwest::Error),

    #[error("Failed to deserialize message: {0}")]
    Deserialization(#[from] serde_json::Error),
}

pub struct Client {
    base_url: String,
    api_key: String,
    client: reqwest::Client,
}

impl Client {
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
