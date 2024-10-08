use futures::stream::TryStreamExt;
use serde::{Deserialize, Serialize};
use yahoo_finance_api::{self as yahoo};

use axum::{extract::Path, http::StatusCode, routing::get, Json, Router};
use fred::prelude::*;
use fred::types::Scanner;
use tokio::sync::OnceCell;

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
    data: Vec<StockData>,
}
#[derive(Serialize, Deserialize)]
struct AllStockData {
    data: Vec<DayStockData>,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let config = RedisConfig::from_url("redis://db:6379").unwrap();
    let client = RedisClient::new(config, None, None, None);
    client.init().await.unwrap();
    RC.set(client).unwrap();
    let app = Router::new()
        .route("/fetchcurrent/:symbol", get(fetch_price))
        .route("/fetch/all", get(fetch_all))
        .route("/fetchday/:symbol", get(fetch_price_day));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    // task::spawn(fetching_data());
    axum::serve(listener, app).await.unwrap();
}

async fn fetch_price(Path(symbol): Path<String>) -> Result<Json<StockData>, StatusCode> {
    let rc = RC.get().unwrap();
    let final_symbol = format!("{}.NS", symbol);
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
                .set(final_symbol, &qstr, Some(Expiration::EX(5)), None, false)
                .await
                .unwrap();

            Ok(Json(sd))
        }
    }
}

async fn get_day_data(symbol: String) -> Result<DayStockData, StatusCode> {
    let rc = RC.get().unwrap();
    let final_symbol = format!("{}.NS", symbol);
    let key = format!("{}_day", final_symbol);
    let quote_str: Option<String> = rc.get(&key).await.unwrap();
    match quote_str {
        Some(val) => {
            let out: DayStockData = serde_json::from_str(&val).unwrap();
            Ok(out)
        }
        None => {
            let provider = yahoo::YahooConnector::new().unwrap();
            let rep = provider
                .get_quote_range(&final_symbol, "1m", "1d")
                .await
                .unwrap();
            let quotes = rep.quotes().unwrap();
            let mut qs = DayStockData { data: Vec::new() };
            for quote in quotes.into_iter() {
                let sd = StockData {
                    symbol: final_symbol.clone(),
                    open: quote.open,
                    close: quote.close,
                    volume: quote.volume,
                    high: quote.high,
                    low: quote.low,
                };
                qs.data.push(sd);
            }
            let qstr = serde_json::to_string(&qs).unwrap();
            let _: () = rc
                .set(&key, &qstr, Some(Expiration::EX(60)), None, false)
                .await
                .unwrap();

            Ok(qs)
        }
    }
}

async fn fetch_price_day(Path(symbol): Path<String>) -> Result<Json<DayStockData>, StatusCode> {
    Ok(Json(get_day_data(symbol).await.unwrap()))
}
async fn fetch_all() -> Result<Json<AllStockData>, StatusCode> {
    let rc = RC.get().unwrap();
    let mut out: Vec<DayStockData> = Vec::new();
    let mut scan_stream = rc.scan("*_day", Some(10), None);
    while let Some(mut page) = scan_stream.try_next().await.unwrap() {
        if let Some(keys) = page.take_results() {
            for k in keys {
                let sd: String = rc.get(&k).await.unwrap();
                out.push(serde_json::from_str(&sd).unwrap());
            }
        }
    }

    let fout = AllStockData { data: out };
    Ok(Json(fout))
}

// async fn fetching_data() {
//     let nifty50 = [
//         "ADANIPORTS",
//         "ASIANPAINT",
//         "AXISBANK",
//         "BAJAJ-AUTO",
//         "BAJAJFINSV",
//         "BAJFINANCE",
//         "BHARTIARTL",
//         "BPCL",
//         "BRITANNIA",
//         "CIPLA",
//         "COALINDIA",
//         "DIVISLAB",
//         "DRREDDY",
//         "EICHERMOT",
//         "GRASIM",
//         "HCLTECH",
//         "HDFC",
//         "HDFCBANK",
//         "HDFCLIFE",
//         "HEROMOTOCO",
//         "HINDALCO",
//         "HINDUNILVR",
//         "ICICIBANK",
//         "INDUSINDBK",
//         "INFY",
//         "ITC",
//         "JSWSTEEL",
//         "KOTAKBANK",
//         "LT",
//         "M&M",
//         "MARUTI",
//         "NESTLEIND",
//         "NTPC",
//         "ONGC",
//         "POWERGRID",
//         "RELIANCE",
//         "SBILIFE",
//         "SBIN",
//         "SUNPHARMA",
//         "TATACONSUM",
//         "TATAMOTORS",
//         "TATASTEEL",
//         "TCS",
//         "TECHM",
//         "TITAN",
//         "ULTRACEMCO",
//         "UPL",
//         "WIPRO",
//     ];
//     let nifty50_symbol: Vec<String> = nifty50.into_iter().map(|x| format!("{}.NS", x)).collect();
//
//     loop {
//         time::sleep(Duration::from_secs(5)).await;
//         println!("Fetching Data");
//         match RC.get() {
//             Some(rc) => {
//                 let _: () = rc.set("hallo", "abcd", None, None, false).await.unwrap();
//                 println!("Values updated");
//             }
//             None => {
//                 eprintln!("Cannot connect to redis")
//             }
//         }
//     }
// }
