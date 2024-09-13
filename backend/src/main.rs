use ::time::{macros::format_description, Date, OffsetDateTime};

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use time::Duration;
use yahoo_finance_api::{self as yahoo};

use axum::{extract::Query, http::StatusCode, routing::get, Json, Router};
use fred::prelude::*;
use tokio::{sync::OnceCell, task, time};

static RC: OnceCell<RedisClient> = OnceCell::const_new();

#[derive(Serialize, Deserialize)]
struct StockData {
    symbol: String,
    open: f64,
    close: f64,
    volume: u64,
    high: f64,
    low: f64,
}
#[derive(Serialize, Deserialize)]
struct DayStockData {
    data: HashMap<String, StockData>,
}
#[derive(Deserialize)]
struct FQuery {
    symbol: String,
    day: Option<String>,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let config = RedisConfig::from_url("redis://db:6379").unwrap();
    let client = RedisClient::new(config, None, None, None);
    client.init().await.unwrap();
    RC.set(client).unwrap();
    let app = Router::new()
        .route("/", get(root))
        .route("/fetch", get(fetch_price))
        .route("/fetchday", get(fetch_price_day));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    task::spawn(fetching_data());
    axum::serve(listener, app).await.unwrap();
}

async fn fetch_price(Query(query): Query<FQuery>) -> Result<Json<StockData>, StatusCode> {
    let rc = RC.get().unwrap();
    let final_symbol = format!("{}.NS", query.symbol);
    let quote_str: Option<String> = rc.get(&final_symbol).await.unwrap();
    match quote_str {
        Some(val) => {
            let out: StockData = serde_json::from_str(&val).unwrap();
            Ok(Json(out))
        }
        None => {
            let provider = yahoo::YahooConnector::new().unwrap();
            let rep = provider
                .get_latest_quotes(&final_symbol, "1d")
                .await
                .unwrap();
            let quote = rep.last_quote().unwrap();
            let sd = StockData {
                symbol: final_symbol.clone(),
                open: quote.open,
                close: quote.close,
                volume: quote.volume,
                high: quote.high,
                low: quote.low,
            };
            let qstr = serde_json::to_string(&sd).unwrap();
            let _: () = rc
                .set(final_symbol, &qstr, Some(Expiration::EX(60)), None, false)
                .await
                .unwrap();

            Ok(Json(sd))
        }
    }
}

async fn fetch_price_day(Query(query): Query<FQuery>) -> Result<Json<DayStockData>, StatusCode> {
    let rc = RC.get().unwrap();
    let date = query.day.unwrap();
    let format = format_description!(
        "[year]-[month]-[day] [hour]:[minute]:[second] [offset_hour \
        sign:mandatory]:[offset_minute]:[offset_second]"
    );
    let final_symbol = format!("{}.NS", query.symbol);
    let key = format!("{}_{}", final_symbol, date);
    let quote_str: Option<String> = rc.get(&key).await.unwrap();
    match quote_str {
        Some(val) => {
            let out: DayStockData = serde_json::from_str(&val).unwrap();
            Ok(Json(out))
        }
        None => {
            let provider = yahoo::YahooConnector::new().unwrap();
            println!("{} 00:00:00", date);
            let start =
                OffsetDateTime::parse(&format!("{} 00:00:00 +05:30:00", date), format).unwrap();
            let end =
                OffsetDateTime::parse(&format!("{} 23:59:59 +05:30:00", date), format).unwrap();
            let rep = provider
                .get_quote_history_interval(&final_symbol, start, end, "1m")
                .await
                .unwrap();
            let quotes = rep.quotes().unwrap();
            let mut qs = DayStockData {
                data: HashMap::new(),
            };
            for quote in quotes.into_iter() {
                let sd = StockData {
                    symbol: final_symbol.clone(),
                    open: quote.open,
                    close: quote.close,
                    volume: quote.volume,
                    high: quote.high,
                    low: quote.low,
                };
                qs.data.insert(final_symbol.clone(), sd);
            }
            let qstr = serde_json::to_string(&qs).unwrap();
            let _: () = rc.set(&key, &qstr, None, None, false).await.unwrap();

            Ok(Json(qs))
        }
    }
}

async fn root() -> &'static str {
    "Hello, World!"
}
async fn fetching_data() {
    let nifty50 = [
        "ADANIPORTS",
        "ASIANPAINT",
        "AXISBANK",
        "BAJAJ-AUTO",
        "BAJAJFINSV",
        "BAJFINANCE",
        "BHARTIARTL",
        "BPCL",
        "BRITANNIA",
        "CIPLA",
        "COALINDIA",
        "DIVISLAB",
        "DRREDDY",
        "EICHERMOT",
        "GRASIM",
        "HCLTECH",
        "HDFC",
        "HDFCBANK",
        "HDFCLIFE",
        "HEROMOTOCO",
        "HINDALCO",
        "HINDUNILVR",
        "ICICIBANK",
        "INDUSINDBK",
        "INFY",
        "ITC",
        "JSWSTEEL",
        "KOTAKBANK",
        "LT",
        "M&M",
        "MARUTI",
        "NESTLEIND",
        "NTPC",
        "ONGC",
        "POWERGRID",
        "RELIANCE",
        "SBILIFE",
        "SBIN",
        "SUNPHARMA",
        "TATACONSUM",
        "TATAMOTORS",
        "TATASTEEL",
        "TCS",
        "TECHM",
        "TITAN",
        "ULTRACEMCO",
        "UPL",
        "WIPRO",
    ];
    let nifty50_symbol: Vec<String> = nifty50.into_iter().map(|x| format!("{}.NS", x)).collect();

    loop {
        time::sleep(Duration::from_secs(5)).await;
        println!("Fetching Data");
        match RC.get() {
            Some(rc) => {
                let _: () = rc.set("hallo", "abcd", None, None, false).await.unwrap();
                println!("Values updated");
            }
            None => {
                eprintln!("Cannot connect to redis")
            }
        }
    }
}
