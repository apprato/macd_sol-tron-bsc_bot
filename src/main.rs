use reqwest::Client;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs::File;
use std::time::Duration;
use tokio::time::sleep;
use log::{debug, info, error};
use simplelog::*;

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

    let response = client.get(url).send().await?;

    // Capture the status and body if the request fails
    if response.status().is_success() {
        let coins_data = response.json::<Vec<Coin>>().await?;
        debug!("Successfully fetched coins data");
        Ok(coins_data)
    } else {
        let status = response.status();
        let body = response.text().await?;
        error!("Failed to fetch coins data. Status: {}, Body: {}", status, body);
        Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, "Failed to fetch coins data")))
    }
}

// Fetch price data for a specific coin from CoinGecko
async fn fetch_price_data(client: &Client, coin_id: &str) -> Result<Option<f64>, Box<dyn std::error::Error>> {
    let url = format!("https://api.coingecko.com/api/v3/simple/price?ids={}&vs_currencies=usd", coin_id);
    debug!("Fetching price for coin: {}", coin_id);
    
    let response = client.get(&url).send().await?.json::<serde_json::Value>().await?;

    if let Some(price_map) = response.get(coin_id) {
        if let Some(price) = price_map.get("usd").and_then(|v| v.as_f64()) {
            debug!("Successfully fetched price for coin {}: {}", coin_id, price);
            return Ok(Some(price));
        } else {
            debug!("Price for {} not found in expected format.", coin_id);
            return Ok(None);
        }
    } else {
        debug!("Coin {} not found in API response.", coin_id);
        return Ok(None);
    }
}

// Filter coins by platform
fn filter_coins_by_network(coins: &[Coin], platform: &str) -> Vec<Coin> {
    debug!("Filtering coins for platform: {}", platform);
    let filtered_coins: Vec<Coin> = coins
        .iter()
        .filter(|coin| coin.platforms.contains_key(platform))
        .cloned()
        .collect();
    debug!("Found {} coins for platform {}", filtered_coins.len(), platform);
    filtered_coins
}

// MACD calculation functions
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

    // Delay to avoid potential rate limits
    sleep(Duration::from_secs(1)).await; 

    // Specify the network and wick size
    let network_to_scan = "the-open-network"; // Change this to the desired network
    let wick_duration = Duration::from_secs(1); // Set to 60 seconds for 1-minute wicks, 1 second can be used for testing price.history is working.

    // Fetch all coins from CoinGecko
    match fetch_all_coins(&client).await {
        Ok(all_coins) => {
            let coins = filter_coins_by_network(&all_coins, network_to_scan);

            info!("Found {} coins on {}", coins.len(), network_to_scan);

            let mut price_history: HashMap<String, Vec<f64>> = HashMap::new();

            loop {
                for coin in &coins {
                    match fetch_price_data(&client, &coin.id).await {
                        Ok(Some(price)) => {
                            println!("Price data found for {}: ${}", coin.symbol, price);

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
                            }

                            if prices.len() > 100 {
                                prices.remove(0);  // Keep only the latest 100 prices
                            }
                        }
                        Ok(None) => {
                            debug!("No price data for {}", coin.id);
                        }
                        Err(e) => {
                            debug!("Error fetching price for {}: {:?}", coin.id, e);
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
