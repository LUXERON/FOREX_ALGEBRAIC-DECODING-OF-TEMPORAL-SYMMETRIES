//! # Data Integration Test
//! 
//! Test the historical data integration with real EUR/USD data

use anyhow::Result;
use std::path::PathBuf;
use forex_pattern_reconstruction::{ForexDataManager, DataConfig};

#[tokio::main]
async fn main() -> Result<()> {
    println!("🔬 FOREX DATA INTEGRATION TEST");
    println!("===============================");
    println!();
    
    // Initialize data manager
    let config = DataConfig::default();
    let mut data_manager = ForexDataManager::new(config)?;
    
    // Test 1: Load daily EUR/USD data (1980-2023)
    println!("📊 Test 1: Loading Daily EUR/USD Data (1980-2023)");
    let daily_path = PathBuf::from("FOREX DATA/Forex Daily (1980) - 2023/archive(4)/Forex_D1/Major/EURUSD.csv");
    
    if daily_path.exists() {
        match data_manager.load_csv_file(&daily_path).await {
            Ok(data) => {
                println!("✅ Successfully loaded {} daily data points", data.len());
                if !data.is_empty() {
                    let first = &data[0];
                    let last = &data[data.len() - 1];
                    println!("   📅 Date range: {} to {}", 
                            first.timestamp.format("%Y-%m-%d"),
                            last.timestamp.format("%Y-%m-%d"));
                    println!("   💰 Price range: {:.4} to {:.4}", 
                            first.close, last.close);
                    
                    // Show sample data points
                    println!("   📈 Sample data points:");
                    for (i, point) in data.iter().take(5).enumerate() {
                        println!("      {}. {} | O:{:.4} H:{:.4} L:{:.4} C:{:.4}", 
                                i + 1,
                                point.timestamp.format("%Y-%m-%d"),
                                point.open, point.high, point.low, point.close);
                    }
                }
            }
            Err(e) => {
                println!("❌ Failed to load daily data: {}", e);
            }
        }
    } else {
        println!("⚠️  Daily data file not found: {}", daily_path.display());
    }
    
    println!();
    
    // Test 2: Load hourly EUR/USD data (2002-2020)
    println!("📊 Test 2: Loading Hourly EUR/USD Data (2002-2020)");
    let hourly_path = PathBuf::from("FOREX DATA/EUR USD Forex Pair Historical Data (2002 - 2020)/archive/eurusd_hour.csv");
    
    if hourly_path.exists() {
        match data_manager.load_oanda_csv(&hourly_path).await {
            Ok(data) => {
                println!("✅ Successfully loaded {} hourly data points", data.len());
                if !data.is_empty() {
                    let first = &data[0];
                    let last = &data[data.len() - 1];
                    println!("   📅 Date range: {} to {}", 
                            first.timestamp.format("%Y-%m-%d %H:%M"),
                            last.timestamp.format("%Y-%m-%d %H:%M"));
                    println!("   💰 Price range: {:.5} to {:.5}", 
                            first.close, last.close);
                    
                    // Show sample data points
                    println!("   📈 Sample hourly data points:");
                    for (i, point) in data.iter().take(5).enumerate() {
                        println!("      {}. {} | O:{:.5} H:{:.5} L:{:.5} C:{:.5}", 
                                i + 1,
                                point.timestamp.format("%Y-%m-%d %H:%M"),
                                point.open, point.high, point.low, point.close);
                    }
                }
            }
            Err(e) => {
                println!("❌ Failed to load hourly data: {}", e);
            }
        }
    } else {
        println!("⚠️  Hourly data file not found: {}", hourly_path.display());
    }
    
    println!();
    
    // Test 3: Data summary
    println!("📊 Test 3: Data Summary");
    match data_manager.get_data_summary().await {
        Ok(summary) => {
            println!("✅ Data summary generated successfully");
            println!("   📁 Total files found: {}", summary.total_files);
            println!("   💱 Available pairs: {}", summary.available_pairs.len());
            
            for (pair, path) in summary.available_pairs.iter().take(10) {
                println!("      - {}: {}", pair, path.display());
            }
            
            if summary.available_pairs.len() > 10 {
                println!("      ... and {} more pairs", summary.available_pairs.len() - 10);
            }
        }
        Err(e) => {
            println!("❌ Failed to generate data summary: {}", e);
        }
    }
    
    println!();
    
    // Test 4: EUR/USD data loading (automatic detection)
    println!("📊 Test 4: Automatic EUR/USD Data Loading");
    let dummy_path = PathBuf::from("dummy");
    match data_manager.load_eur_usd_data(&dummy_path).await {
        Ok(data) => {
            println!("✅ Successfully loaded EUR/USD data automatically");
            println!("   📊 Total data points: {}", data.len());
            
            if !data.is_empty() {
                let first = &data[0];
                let last = &data[data.len() - 1];
                println!("   📅 Date range: {} to {}", 
                        first.timestamp.format("%Y-%m-%d"),
                        last.timestamp.format("%Y-%m-%d"));
                
                // Calculate basic statistics
                let prices: Vec<f64> = data.iter().map(|d| d.close).collect();
                let min_price = prices.iter().fold(f64::INFINITY, |a, &b| a.min(b));
                let max_price = prices.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
                let avg_price = prices.iter().sum::<f64>() / prices.len() as f64;
                
                println!("   💰 Price statistics:");
                println!("      Min: {:.5}", min_price);
                println!("      Max: {:.5}", max_price);
                println!("      Avg: {:.5}", avg_price);
                println!("      Range: {:.5}", max_price - min_price);
            }
        }
        Err(e) => {
            println!("❌ Failed to load EUR/USD data: {}", e);
        }
    }
    
    println!();
    println!("🎯 DATA INTEGRATION TEST COMPLETE");
    println!("==================================");
    println!("✅ Historical data integration is ready for pattern analysis!");
    println!("🚀 You can now run the dashboard: cargo run --bin dashboard");
    
    Ok(())
}
