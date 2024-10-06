use reqwest::Client;
use serde::Deserialize;
use std::collections::HashMap;
use std::time::Duration;
use tokio::time::sleep;

#[derive(Deserialize, Debug, Clone)]
struct Coin {
    id: String,
    symbol: String,
    #[serde(default)]  // Mark `_name` as optional, providing a default value if missing
    _name: String,  // Prefixed with underscore to suppress warning for unused field
    platforms: HashMap<String, String>,  // To filter by network platform
}

// Fetch the list of all coins from CoinGecko
async fn fetch_all_coins(client: &Client) -> Result<Vec<Coin>, Box<dyn std::error::Error>> {
    let url = "https://api.coingecko.com/api/v3/coins/list?include_platform=true";
    let response = client.get(url).send().await?.json::<Vec<Coin>>().await?;
    Ok(response)
}

// Fetch price data for a specific coin from CoinGecko
async fn fetch_price_data(client: &Client, coin_id: &str) -> Result<Option<f64>, Box<dyn std::error::Error>> {
    let url = format!("https://api.coingecko.com/api/v3/simple/price?ids={}&vs_currencies=usd", coin_id);
    let response = client.get(&url).send().await?.json::<serde_json::Value>().await?;

    // Extract the price from the JSON response
    if let Some(price) = response[coin_id].get("usd").and_then(|v| v.as_f64()) {
        Ok(Some(price))
    } else {
        Ok(None)  // Return None if the price data is not found or invalid
    }
}

// Filter coins by platform
fn filter_coins_by_network(coins: &[Coin], platform: &str) -> Vec<Coin> {
    // Print platforms for debugging purposes
    println!("Debugging platforms for first few coins:");
    for coin in coins.iter() {
        println!("Coin ID: {}, Platforms: {:?}", coin.id, coin.platforms);
    }

    coins.iter()
        .filter(|coin| coin.platforms.contains_key(platform))
        .cloned()
        .collect()  // Now this correctly returns the filtered coins
}

// MACD calculation functions
fn calculate_macd(prices: &[f64]) -> (Vec<f64>, Vec<f64>) {
    if prices.is_empty() {
        panic!("The prices vector is empty! Unable to calculate MACD.");
    }

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

    ema.push(prices[0]); // Initialize EMA with first price

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
    println!("{} signal for {}", action, token);
}

#[tokio::main]
async fn main() {
    let client = Client::new();

    // Specify the network you want to scan: "solana", "binance-smart-chain", or "the-open-network"
    let network_to_scan = "the-open-network"; // Change this line to scan a specific chain

    // Fetch all coins from CoinGecko
    let all_coins = fetch_all_coins(&client).await.unwrap();

    // Filter coins by the specified network
    let coins = filter_coins_by_network(&all_coins, network_to_scan);

    println!("Found {} coins on {}", coins.len(), network_to_scan);

    // Scan coins from the selected network
    loop {
        for coin in &coins {
            match fetch_price_data(&client, &coin.id).await {
                Ok(Some(price)) => {
                    let prices: Vec<f64> = vec![price]; // Replace with real historical data

                    if prices.len() >= 26 {
                        let (macd, signal) = calculate_macd(&prices);
                        if let Some(action) = check_macd_signal(&macd, &signal) {
                            execute_trade(&coin.symbol, action).await;
                        }
                    }
                }
                Ok(None) => {
                    // Commented out: Hide output if the price is not available
                }
                Err(_e) => {
                    // Commented out: Hide error output if there was an error fetching price
                }
            }

            // Sleep to avoid rate limits
            sleep(Duration::from_millis(100)).await;
        }

        // Sleep before the next round of price fetching
        sleep(Duration::from_secs(30)).await; // Reduced sleep time for faster iteration
    }
}
