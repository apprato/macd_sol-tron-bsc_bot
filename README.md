
# MACD Trading Bot for Specific Chains (TON, Solana, BSC)

This Rust-based bot scans all coins on a **specified network** (TON, Solana, or Binance Smart Chain), calculates the **MACD (Moving Average Convergence Divergence)**, and checks for trading opportunities based on **MACD signals** (Buy/Sell).

## Features
- **Scans all coins** on a specific network (TON, Solana, or Binance Smart Chain) using the CoinGecko API.
- Fetches **real-time price data** for each coin/token.
- Calculates **MACD and Signal Line** for each token based on historical price data.
- Identifies and prints **buy/sell signals** when MACD crossovers are detected.
- Option to run the bot **on a specific chain** by modifying a line in the code.
- **Always outputs MACD values** even if no buy/sell signal is detected.

## Prerequisites

Make sure you have the following installed on your system:

- **Rust** (Install from [here](https://www.rust-lang.org/tools/install))
- **Cargo** (part of Rust)
- Internet connection to fetch price data via APIs.

## How to Install and Run

1. **Clone the repository** or download the ZIP file.
2. **Navigate to the project directory**:

   ```bash
   cd macd_bot
   ```

3. **Build the project**:

   ```bash
   cargo build
   ```

4. **Run the project**:

   ```bash
   cargo run
   ```

## Running on Specific Networks

To run the bot on a specific network (TON, Solana, or BSC), modify the following line in the `main.rs` file:

```rust
let network_to_scan = "solana"; // Options: "solana", "binance-smart-chain", or "ton"
```

Change `"solana"` to the network you want to scan.

## Customizing the Wick Duration

You can change the interval at which the bot fetches price data and recalculates the MACD by modifying the `wick_duration` variable in the `main.rs` file. This variable determines the time between each price data fetch (also referred to as "wicks").

```rust
let wick_duration = Duration::from_secs(60); // Fetches price every 60 seconds (1-minute wicks)
```

To change the wick duration:
- **1-second wicks**: `Duration::from_secs(1)`
- **5-minute wicks**: `Duration::from_secs(300)`
  
The default setting is 60 seconds for 1-minute wicks.

## How It Works

1. **Fetching Coins**: The bot uses CoinGecko's API to fetch a list of all available coins/tokens on the selected network.
   
2. **Fetching Price Data**: For each coin, the bot fetches real-time price data using CoinGecko's API.
   
3. **MACD Calculation**: The bot calculates the **MACD (12-period EMA, 26-period EMA)** and the **Signal Line (9-period EMA of MACD)** using the collected price data.
   - The MACD and Signal Line values are always printed to the console when price data is found, even if no buy or sell signal is detected.

4. **Buy/Sell Signals**: The bot checks for **MACD crossovers**:
   - **Buy signal**: When the MACD line crosses **above** the Signal Line.
   - **Sell signal**: When the MACD line crosses **below** the Signal Line.
   
5. **Iteration**: The bot runs continuously, fetching new prices and updating the MACD calculation based on the configured wick duration.

## API Usage and Rate Limiting

The bot uses the **CoinGecko API** to fetch both the list of coins/tokens and their real-time prices. To avoid hitting API rate limits:
- The bot introduces a **100 millisecond delay** between each request.
- The bot waits for the specified `wick_duration` (default: 60 seconds) between each batch of price updates for all tokens.

## Customizing the Bot

1. **Network Selection**: Modify the `network_to_scan` variable to change the network being scanned.

2. **Wick Duration**: Adjust the `wick_duration` variable to control how frequently price data is fetched and MACD is recalculated. The default is set to 60 seconds for 1-minute wicks.

3. **Historical Price Data**: The bot can be customized to store and fetch actual historical prices, either by extending the data fetching logic or by integrating with external services.

## License

This project is licensed under the MIT License.
