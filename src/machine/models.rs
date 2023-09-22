use crate::Exchange;
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};

/// The options that can be specified for calling Tardis Machine Server's replay-normalized.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReplayNormalizedRequestOptions {
    pub exchange: Exchange,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub symbols: Option<Vec<String>>,
    pub from: NaiveDate,
    pub to: NaiveDate,
    pub data_types: Vec<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub with_disconnect_messages: Option<bool>,
}

/// The possible type of message returned from Tardis Machine Server.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum Message {
    Trade(Trade),
    BookChange(BookChange),
    DerivativeTicker(DerivativeTicker),
    BookSnapshot(BookSnapshot),
    TradeBar(TradeBar),
    Disconnect(Disconnect),
}

/// Side of the trade.
#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TradeSide {
    Buy,
    Sell,
    Unknown,
}

/// Individual trade.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Trade {
    /// Instrument symbol as provided by exchange
    symbol: String,

    /// Exchange ID
    exchange: Exchange,

    /// Trade id if provided by exchange
    id: Option<String>,

    /// Trade price as provided by exchange
    price: f64,

    /// Trade amount as provided by exchange
    amount: f64,

    /// Liquidity taker side (aggressor)
    side: TradeSide,

    /// Trade timestamp provided by exchange (ISO 8601 format)
    timestamp: DateTime<Utc>,

    /// Message arrival timestamp (ISO 8601 format)
    local_timestamp: DateTime<Utc>,
}

/// Initial L2 (market by price) order book snapshot (isSnapshot=true) plus incremental updates for
/// each order book change.  Please note that amount is the updated amount at that price level,
/// not a delta. An amount of 0 indicates the price level can be removed.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BookChange {
    /// Instrument symbol as provided by exchange
    symbol: String,

    /// Exchange ID
    exchange: Exchange,

    /// If true marks initial order book snapshot
    is_snapshot: bool,

    /// Updated bids price-amount levels
    bids: Vec<BookLevel>,

    /// Updated asks price-amount levels
    asks: Vec<BookLevel>,

    /// Order book update timestamp if provided by exchange,
    /// otherwise equals to localTimestamp, (ISO 8601 format)
    timestamp: DateTime<Utc>,

    /// Message arrival timestamp (ISO 8601 format)
    local_timestamp: DateTime<Utc>,
}

/// Derivative instrument ticker info sourced from real-time ticker & instrument channels.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DerivativeTicker {
    /// Instrument symbol as provided by exchange
    symbol: String,

    /// Exchange ID
    exchange: Exchange,

    /// Last instrument price if provided by exchange
    last_price: Option<f64>,

    /// Last open interest if provided by exchange
    open_interest: Option<f64>,

    /// Last funding rate if provided by exchange
    funding_rate: Option<f64>,

    /// Last index price if provided by exchange
    index_price: Option<f64>,

    /// Last mark price if provided by exchange
    mark_price: Option<f64>,

    /// Message timestamp provided by exchange (ISO 8601 format)
    timestamp: DateTime<Utc>,

    /// Message arrival timestamp (ISO 8601 format)
    local_timestamp: DateTime<Utc>,
}

/// A particular level in the order book.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BookLevel {
    pub price: f64,
    pub amount: f64,
}

/// Order book snapshot for selected number_of_levels (top bids and asks), snapshot_interval and time_unit.
/// When snapshot_interval is set to 0 , snapshots are taken anytime order book state within specified
/// levels has changed, otherwise snapshots are taken anytime snapshot_interval time has passed and
/// there was an order book state change within specified levels. Order book snapshots are computed
/// from exchanges' real-time order book streaming L2 data (market by price).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BookSnapshot {
    /// Instrument symbol as provided by exchange
    pub symbol: String,

    /// Exchange ID
    pub exchange: Exchange,

    /// Name with format book_snapshot_{depth}_{interval}{time_unit}
    pub name: String,

    /// Requested number of levels (top bids/asks)
    pub depth: u64,

    /// Requested snapshot interval in milliseconds
    pub interval: u64,

    /// Top "depth" bids price-amount levels
    pub bids: Vec<BookLevel>,

    /// Top "depth" asks price-amount levels
    pub asks: Vec<BookLevel>,

    /// Snapshot timestamp based on last book_change message processed timestamp adjusted to snapshot interval
    pub timestamp: DateTime<Utc>,

    /// Message arrival timestamp that triggered snapshot (ISO 8601 format)
    pub local_timestamp: DateTime<Utc>,
}

/// Kind of the trade bar.
#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TradeBarKind {
    Time,
    Volume,
    Tick,
}

/// Trades data in aggregated form, known as OHLC, candlesticks, klines etc. Not only most common
/// time based aggregation is supported, but volume and tick count based as well. Bars are computed
/// from tick-by-tick raw trade data, if in given interval no trades happened, there is no bar produced.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
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
    pub open_timestamp: DateTime<Utc>,

    /// timestamp of last trade for given bar (ISO 8601 format)
    pub close_timestamp: DateTime<Utc>,

    /// end of interval period timestamp (ISO 8601 format)
    pub timestamp: DateTime<Utc>,

    /// message arrival timestamp that triggered given bar computation (ISO 8601 format)
    pub local_timestamp: DateTime<Utc>,
}

/// Message that marks events when real-time WebSocket connection that was used to collect the
/// historical data got disconnected.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Disconnect {
    /// Exchange ID
    pub exchange: Exchange,

    /// message arrival timestamp that triggered given bar computation (ISO 8601 format)
    pub local_timestamp: DateTime<Utc>,
}
