use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
/// The response format for Tardis.dev API.
pub enum Response<T> {
    /// The error response.
    Error {
        /// Error code
        code: u64,

        /// Error message
        message: String,
    },

    /// The success response.
    Success(T),
}

#[allow(missing_docs)]
#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
/// Supported exchanges on Tardis
/// Visit <https://api.tardis.dev/v1/exchanges> to get the list of all supported exchanges that
/// historical market data is available for.
pub enum Exchange {
    /// [Bitmex](https://www.bitmex.com/) exchange.
    Bitmex,

    /// [Deribit](https://www.deribit.com/) exchange.
    Deribit,

    /// [Binance](https://binance.com/) exchange.
    BinanceFutures,

    /// [Binance](https://binance.com/) exchange.
    BinanceDelivery,

    /// [Binance](https://binance.com/) exchange.
    BinanceOptions,

    /// [Binance](https://binance.com/) exchange.
    Binance,
    Ftx,
    OkexFutures,
    OkexOptions,
    OkexSwap,
    Okex,
    HuobiDm,
    HuobiDmSwap,
    HuobiDmLinearSwap,
    Huobi,
    BitfinexDerivatives,
    Bitfinex,
    Coinbase,
    Cryptofacilities,
    Kraken,
    Bitstamp,
    Gemini,
    Poloniex,
    Bybit,
    BybitSpot,
    BybitOptions,
    Phemex,
    Delta,
    FtxUs,
    BinanceUs,
    GateIoFutures,
    GateIo,
    Okcoin,
    Bitflyer,
    Hitbtc,
    Coinflex,
    BinanceJersey,
    BinanceDex,
    Upbit,
    Ascendex,
    Dydx,
    Serum,
    Mango,
    HuobiDmPptions,
    StarAtlas,
    CryptoCom,
    CryptoComDerivatives,
    Kucoin,
    Bitnomial,
    WooX,
    BlockchainCom,
}

impl ToString for Exchange {
    fn to_string(&self) -> String {
        serde_json::to_value(self)
            .unwrap()
            .as_str()
            .unwrap()
            .to_string()
    }
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
/// The type of the symbol eg. Spot, Perpetual, Future, Option.
pub enum SymbolType {
    /// Spot market.
    Spot,

    /// Perpetual market (also known as Linear).
    Perpetual,

    /// Futures market (also known as Delivery).
    Future,

    /// Option market.
    Option,
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
/// The type of an option symbol eg. Call, Put
pub enum OptionType {
    /// Call option.
    Call,

    /// Put option.
    Put,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
/// The changes info returned by exchanges API. Note that is meant to be accurate and complete only for
/// contractMultiplier values (we monitor exchanges announcements for that), rest of the
/// changes are done on best effort basis and not always complete
pub struct InstrumentChanges {
    /// Date in ISO format
    pub until: String,

    /// Price tick size, price precision can be calculated from it
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub price_increment: Option<f64>,

    /// Amount tick size, amount/size precision can be calculated from it
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub amount_increment: Option<f64>,

    /// Only for derivatives
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub contract_multiplier: Option<f64>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
/// The metadata of a particular instrument, see <https://docs.tardis.dev/api/instruments-metadata-api>.
pub struct InstrumentInfo {
    /// Symbol ID
    pub id: String,

    /// Exchange
    pub exchange: String,

    /// Normalized, so for example bitmex XBTUSD has base currency set to BTC not XBT
    pub base_currency: String,

    /// Normalized, so for example bitfinex BTCUST has quote currency set to USDT, not UST
    pub quote_currency: String,

    /// Type of the symbol eg. Spot, Perpetual, Future, Option
    #[serde(rename = "type")]
    pub symbol_type: SymbolType,

    /// Indicates if the instrument can currently be traded
    pub active: bool,

    /// Date in ISO format
    pub available_since: String,

    /// Date in ISO format
    pub available_to: Option<String>,

    /// Date in ISO format, only for futures and options
    pub expiry: Option<String>,

    /// Price tick size, price precision can be calculated from it
    pub price_increment: f64,

    /// Amount tick size, amount/size precision can be calculated from it
    pub amount_increment: f64,

    /// Min order size
    pub min_trade_amount: f64,

    /// Consider it as illustrative only, as it depends in practice on account traded volume levels, different categories, VIP levels, owning exchange currency etc
    pub maker_fee: f64,

    /// Consider it as illustrative only, as it depends in practice on account traded volume levels, different categories, VIP levels, owning exchange currency etc
    pub taker_fee: f64,

    /// Only for derivatives
    pub inverse: Option<bool>,

    /// Only for derivatives
    pub contract_multiplier: Option<f64>,

    /// Only for quanto instruments
    pub quanto: Option<bool>,

    /// Only for quanto instruments as settlement currency is different base/quote currency
    pub settlement_currency: Option<String>,

    /// Strike price, only for options
    pub strike_price: Option<f64>,

    /// Option type, only for options
    pub option_type: Option<OptionType>,

    /// changes info returned by the API is meant to be accurate and complete only for
    /// contractMultiplier values (we monitor exchanges announcements for that), rest of the
    /// changes are done on best effort basis and not always complete.
    pub changes: Option<Vec<InstrumentChanges>>,
}
