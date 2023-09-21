use crate::Exchange;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize, Serializer};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReplayNormalizedRequestOptions {
    pub exchange: Exchange,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub symbols: Option<Vec<String>>,

    #[serde(serialize_with = "as_iso8601_date")]
    pub from: DateTime<Utc>,

    #[serde(serialize_with = "as_iso8601_date")]
    pub to: DateTime<Utc>,
    pub data_types: Vec<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub with_disconnect_messages: Option<bool>,
}

fn as_iso8601_date<S>(v: &DateTime<Utc>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(v.to_rfc3339().split('T').collect::<Vec<&str>>()[0])
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum Message {
    Trade(serde_json::Value),
    BookChange(serde_json::Value),
    DerivativeTicker(serde_json::Value),
    BookSnapshot(serde_json::Value),
    TradeBar(TradeBar),
    Disconnect(Disconnect),
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
/// Kind of the trade bar.
pub enum TradeBarKind {
    Time,
    Volume,
    Tick,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Trades data in aggregated form, known as OHLC, candlesticks, klines etc. Not only most common
/// time based aggregation is supported, but volume and tick count based as well. Bars are computed
/// from tick-by-tick raw trade data, if in given interval no trades happened, there is no bar produced.
pub struct TradeBar {
    /// Instrument symbol as provided by exchange
    pub symbol: String,

    /// Exchange ID
    pub exchange: Exchange,

    /// name with format trade_bar_{interval}
    pub name: String,

    /// requested trade bar interval
    pub interval: u64,

    /// open price
    pub open: f64,

    /// high price
    pub high: f64,

    /// low price
    pub low: f64,

    /// close price
    pub close: f64,

    /// total volume traded in given interval
    pub volume: f64,

    /// buy volume traded in given interval
    pub buy_volume: f64,

    /// sell volume traded in given interval
    pub sell_volume: f64,

    /// trades count in given interval
    pub trades: u64,

    /// volume weighted average price
    pub vwap: f64,

    /// timestamp of first trade for given bar (ISO 8601 format)
    pub open_timestamp: String,

    /// timestamp of last trade for given bar (ISO 8601 format)
    pub close_timestamp: String,

    /// end of interval period timestamp (ISO 8601 format)
    pub timestamp: String,

    /// message arrival timestamp that triggered given bar computation (ISO 8601 format)
    pub local_timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Message that marks events when real-time WebSocket connection that was used to collect the
/// historical data got disconnected.
pub struct Disconnect {
    /// Exchange ID
    pub exchange: Exchange,

    /// message arrival timestamp that triggered given bar computation (ISO 8601 format)
    pub local_timestamp: String,
}
