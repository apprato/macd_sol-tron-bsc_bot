use reqwest::Client;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs::File;
use std::time::Duration;
use tokio::time::sleep;
use log::{debug, info, error};
use simplelog::*;

const MAX_COINS_PER_BATCH: usize = 200; // Modify this variable to change the number of coins per batch

#[derive(Deserialize, Debug, Clone)]
struct Coin {
    id: String,
    symbol: String,
    #[serde(default)]
    _name: String,
    platforms: HashMap<String, String>,
}

// Fetch the list of all coins from CoinGecko
async fn fetch_all_coins(client: &Client) -> Result<Vec<Coin>, Box<dyn std::error::Error>> {
    let url = "https://api.coingecko.com/api/v3/coins/list?include_platform=true";
    debug!("Fetching all coins from CoinGecko");

    let response = client.get(url)
        .timeout(Duration::from_secs(10))  // Timeout after 10 seconds
        .send().await?;

    if response.status().is_success() {
        let coins_data = response.json::<Vec<Coin>>().await?;
        debug!("Successfully fetched coins data");
        Ok(coins_data)
    } else {
        let status = response.status();
        let body = response.text().await?;
        error!("Failed to fetch coins data. Status: {}, Body: {}", status, body);
        println!("Failed to fetch coins data. Status: {}, Body: {}", status, body);
        Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, "Failed to fetch coins data")))
    }
}

// Fetch price data for a batch of coins from CoinGecko
async fn fetch_price_data(client: &Client, coin_ids: &[&str]) -> Result<HashMap<String, f64>, Box<dyn std::error::Error>> {
    let ids = coin_ids.join(",");
    let url = format!("https://api.coingecko.com/api/v3/simple/price?ids={}&vs_currencies=usd", ids);
    debug!("Fetching price for coins: {:?}", coin_ids);

    let response = client.get(&url).send().await?.json::<serde_json::Value>().await?;

    let mut prices = HashMap::new();
    for &coin_id in coin_ids {
        if let Some(price_map) = response.get(coin_id) {
            if let Some(price) = price_map.get("usd").and_then(|v| v.as_f64()) {
                prices.insert(coin_id.to_string(), price);
            }
        }
    }
    Ok(prices)
}

// Helper function to split the coin list into batches of up to MAX_COINS_PER_BATCH
fn split_into_batches<'a>(coins: &'a [Coin]) -> Vec<Vec<&'a Coin>> {
    coins.chunks(MAX_COINS_PER_BATCH).map(|chunk| chunk.iter().collect()).collect()
}

// Function to filter coins by platform
fn filter_coins_by_network(coins: &[Coin], platform: &str) -> Vec<Coin> {
    coins
        .iter()
        .filter(|coin| coin.platforms.contains_key(platform))
        .cloned()
        .collect()
}

// MACD calculation functions remain the same...
fn calculate_macd(prices: &[f64]) -> (Vec<f64>, Vec<f64>) {
    let ema_12 = calculate_ema(prices, 12);
    let ema_26 = calculate_ema(prices, 26);

    let macd_line: Vec<f64> = ema_12.iter()
        .zip(ema_26.iter())
        .map(|(e12, e26)| e12 - e26)
        .collect();

    let signal_line = calculate_ema(&macd_line, 9);

    (macd_line, signal_line)
}

fn calculate_ema(prices: &[f64], period: usize) -> Vec<f64> {
    let mut ema = Vec::with_capacity(prices.len());
    let multiplier = 2.0 / (period as f64 + 1.0);

    ema.push(prices[0]);

    for price in prices.iter().skip(1) {
        let prev_ema = *ema.last().unwrap();
        ema.push((price - prev_ema) * multiplier + prev_ema);
    }

    ema
}

fn check_macd_signal(macd_line: &[f64], signal_line: &[f64]) -> Option<&'static str> {
    let macd_last = macd_line[macd_line.len() - 1];
    let macd_prev = macd_line[macd_line.len() - 2];

    let signal_last = signal_line[signal_line.len() - 1];
    let signal_prev = signal_line[signal_line.len() - 2];

    if macd_prev <= signal_prev && macd_last > signal_last {
        Some("buy")
    } else if macd_prev >= signal_prev && macd_last < signal_last {
        Some("sell")
    } else {
        None
    }
}

async fn execute_trade(token: &str, action: &str) {
    println!("{} signal for {}", action, token);  // Show buy/sell signals in console
}

#[tokio::main]
async fn main() {
    CombinedLogger::init(vec![
        WriteLogger::new(
            LevelFilter::Debug,
            Config::default(),
            File::create("debug.log").unwrap(),
        ),
    ])
    .unwrap();

    let client = Client::new();

    // Specify the network and wick size
    let network_to_scan = "the-open-network"; // Change this to the desired network
    let wick_duration = Duration::from_secs(60); // Set to 60 seconds for 1-minute wicks

    // Fetch all coins from CoinGecko
    match fetch_all_coins(&client).await {
        Ok(all_coins) => {
            let coins = filter_coins_by_network(&all_coins, network_to_scan);
            info!("Found {} coins on {}", coins.len(), network_to_scan);

            // Split coins into batches of up to MAX_COINS_PER_BATCH
            let coin_batches = split_into_batches(&coins);
            info!("Processing {} batches of coins", coin_batches.len());

            // Keep price history across iterations
            let mut price_history: HashMap<String, Vec<f64>> = HashMap::new();

            loop {
                for batch in coin_batches.iter() {
                    let coin_ids: Vec<&str> = batch.iter().map(|coin| coin.id.as_str()).collect();

                    match fetch_price_data(&client, &coin_ids).await {
                        Ok(prices) => {
                            for coin in batch {
                                if let Some(&price) = prices.get(&coin.id) {
                                    println!("Price data found for {}: ${}", coin.symbol, price);

                                    // Persist price history across iterations
                                    let prices = price_history.entry(coin.symbol.clone()).or_insert_with(Vec::new);
                                    prices.push(price);

                                    // Output the price history length and MACD if there are enough prices
                                    println!("Price history length for {}: {}", coin.symbol, prices.len());
                                    println!("Price history for {}: {:?}", coin.symbol, prices);

                                    if prices.len() >= 12 {
                                        let (macd, signal) = calculate_macd(&prices);

                                        // Always print MACD and Signal line to console
                                        println!("MACD line for {}: {:?}", coin.symbol, macd);
                                        println!("Signal line for {}: {:?}", coin.symbol, signal);

                                        if prices.len() >= 26 {
                                            if let Some(action) = check_macd_signal(&macd, &signal) {
                                                execute_trade(&coin.symbol, action).await;
                                            }
                                        }
                                    } else {
                                        // If not enough data, output placeholder values
                                        println!("MACD line for {}: N/A (insufficient data)", coin.symbol);
                                        println!("Signal line for {}: N/A (insufficient data)", coin.symbol);
                                    }

                                    if prices.len() > 100 {
                                        prices.remove(0);  // Keep only the latest 100 prices
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            debug!("Error fetching prices for batch: {:?}", e);
                        }
                    }

                    sleep(wick_duration).await;
                }

                sleep(Duration::from_secs(1)).await;
            }
        }
        Err(e) => {
            error!("Error fetching all coins: {:?}", e);
            println!("Failed to fetch all coins. Error: {:?}", e);
        }
    }
}
