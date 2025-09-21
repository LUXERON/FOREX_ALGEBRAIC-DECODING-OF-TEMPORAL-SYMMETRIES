//! # Forex Data Management
//!
//! Data loading, processing, and real-time feed management for forex analysis.

use anyhow::Result;
use chrono::{DateTime, Utc, NaiveDateTime, NaiveDate};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::collections::HashMap;
use csv::ReaderBuilder;
use polars::prelude::*;

/// Forex data point structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForexDataPoint {
    pub timestamp: DateTime<Utc>,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: Option<f64>,
}

/// Data configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DataConfig {
    pub data_directory: PathBuf,
    pub cache_enabled: bool,
    pub max_cache_size: usize,
}

impl Default for DataConfig {
    fn default() -> Self {
        Self {
            data_directory: PathBuf::from("FOREX DATA"),
            cache_enabled: true,
            max_cache_size: 1000000,
        }
    }
}

/// Forex data manager
pub struct ForexDataManager {
    config: DataConfig,
}

impl ForexDataManager {
    pub fn new(config: DataConfig) -> Result<Self> {
        Ok(Self { config })
    }

    /// Load historical forex data from various sources
    pub async fn load_data(
        &mut self,
        input: &PathBuf,
        pair: &str,
        timeframe: &str,
    ) -> Result<Vec<ForexDataPoint>> {
        if input.is_file() {
            self.load_csv_file(input)
        } else if input.is_dir() {
            self.load_from_directory(input, pair, timeframe).await
        } else {
            Err(anyhow::anyhow!("Invalid input path: {}", input.display()))
        }
    }

    /// Load EUR/USD data from the comprehensive dataset
    pub async fn load_eur_usd_data(&mut self, data_file: &PathBuf) -> Result<Vec<ForexDataPoint>> {
        // Try to load from the daily dataset first (1980-2023)
        let daily_path = PathBuf::from("FOREX DATA/Forex Daily (1980) - 2023/archive(4)/Forex_D1/Major/EURUSD.csv");
        if daily_path.exists() {
            return self.load_csv_file(&daily_path);
        }

        // Fallback to hourly data (2002-2020)
        let hourly_path = PathBuf::from("FOREX DATA/EUR USD Forex Pair Historical Data (2002 - 2020)/archive/eurusd_hour.csv");
        if hourly_path.exists() {
            return self.load_oanda_csv(&hourly_path).await;
        }

        // Fallback to provided file
        self.load_csv_file(data_file)
    }

    /// Load standard CSV format (time,open,high,low,close,volume)
    pub fn load_csv_file(&self, file_path: &PathBuf) -> Result<Vec<ForexDataPoint>> {
        let mut data = Vec::new();
        let mut reader = ReaderBuilder::new()
            .has_headers(true)
            .from_path(file_path)?;

        for result in reader.deserialize() {
            let record: CsvRecord = result?;
            let data_point = self.parse_csv_record(record)?;
            data.push(data_point);
        }

        // Sort by timestamp
        data.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));

        Ok(data)
    }

    /// Load Oanda format CSV (Date,Time,BO,BH,BL,BC,BCh,AO,AH,AL,AC,ACh)
    pub async fn load_oanda_csv(&self, file_path: &PathBuf) -> Result<Vec<ForexDataPoint>> {
        let mut data = Vec::new();
        let mut reader = ReaderBuilder::new()
            .has_headers(true)
            .from_path(file_path)?;

        for result in reader.deserialize() {
            let record: OandaCsvRecord = result?;
            let data_point = self.parse_oanda_record(record)?;
            data.push(data_point);
        }

        // Sort by timestamp
        data.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));

        Ok(data)
    }

    /// Load data from directory structure
    async fn load_from_directory(
        &self,
        dir_path: &PathBuf,
        pair: &str,
        timeframe: &str,
    ) -> Result<Vec<ForexDataPoint>> {
        // Look for specific pair file in directory
        let pair_file = dir_path.join(format!("{}.csv", pair));
        if pair_file.exists() {
            return self.load_csv_file(&pair_file);
        }

        // Look in subdirectories
        let major_dir = dir_path.join("Major");
        if major_dir.exists() {
            let pair_file = major_dir.join(format!("{}.csv", pair));
            if pair_file.exists() {
                return self.load_csv_file(&pair_file);
            }
        }

        Err(anyhow::anyhow!("Could not find data for pair {} in directory {}", pair, dir_path.display()))
    }

    /// Parse standard CSV record
    fn parse_csv_record(&self, record: CsvRecord) -> Result<ForexDataPoint> {
        let timestamp = self.parse_timestamp(&record.time)?;

        Ok(ForexDataPoint {
            timestamp,
            open: record.open,
            high: record.high,
            low: record.low,
            close: record.close,
            volume: record.tick_volume,
        })
    }

    /// Parse Oanda CSV record
    fn parse_oanda_record(&self, record: OandaCsvRecord) -> Result<ForexDataPoint> {
        let datetime_str = format!("{} {}", record.date, record.time);
        let timestamp = self.parse_oanda_timestamp(&datetime_str)?;

        Ok(ForexDataPoint {
            timestamp,
            open: record.bo,  // Bid Open
            high: record.bh,  // Bid High
            low: record.bl,   // Bid Low
            close: record.bc, // Bid Close
            volume: None,     // No volume in Oanda format
        })
    }

    /// Parse timestamp from various formats
    fn parse_timestamp(&self, time_str: &str) -> Result<DateTime<Utc>> {
        // Try different timestamp formats
        if let Ok(dt) = DateTime::parse_from_rfc3339(time_str) {
            return Ok(dt.with_timezone(&Utc));
        }

        if let Ok(naive_dt) = NaiveDateTime::parse_from_str(time_str, "%Y-%m-%d %H:%M:%S") {
            return Ok(DateTime::from_naive_utc_and_offset(naive_dt, Utc));
        }

        if let Ok(naive_date) = NaiveDate::parse_from_str(time_str, "%Y-%m-%d") {
            let naive_dt = naive_date.and_hms_opt(0, 0, 0).unwrap();
            return Ok(DateTime::from_naive_utc_and_offset(naive_dt, Utc));
        }

        Err(anyhow::anyhow!("Could not parse timestamp: {}", time_str))
    }

    /// Parse Oanda timestamp format
    fn parse_oanda_timestamp(&self, datetime_str: &str) -> Result<DateTime<Utc>> {
        let naive_dt = NaiveDateTime::parse_from_str(datetime_str, "%Y-%m-%d %H:%M")?;
        Ok(DateTime::from_naive_utc_and_offset(naive_dt, Utc))
    }

    /// Get available data summary
    pub async fn get_data_summary(&self) -> Result<DataSummary> {
        let mut summary = DataSummary {
            available_pairs: HashMap::new(),
            total_files: 0,
            date_ranges: HashMap::new(),
        };

        // Scan available data files
        self.scan_data_directory(&self.config.data_directory, &mut summary)?;

        Ok(summary)
    }

    fn scan_data_directory(&self, dir: &PathBuf, summary: &mut DataSummary) -> Result<()> {
        if !dir.exists() {
            return Ok(());
        }

        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() && path.extension().map_or(false, |ext| ext == "csv") {
                summary.total_files += 1;

                if let Some(stem) = path.file_stem() {
                    if let Some(pair_name) = stem.to_str() {
                        let pair_name = pair_name.to_uppercase();
                        summary.available_pairs.insert(pair_name.clone(), path.clone());

                        // Try to get date range
                        if let Ok(data) = self.load_csv_file(&path) {
                            if !data.is_empty() {
                                let start_date = data.first().unwrap().timestamp;
                                let end_date = data.last().unwrap().timestamp;
                                summary.date_ranges.insert(pair_name, (start_date, end_date));
                            }
                        }
                    }
                }
            } else if path.is_dir() {
                self.scan_data_directory(&path, summary)?;
            }
        }

        Ok(())
    }
}

/// CSV record structure for standard format
#[derive(Debug, Deserialize)]
struct CsvRecord {
    time: String,
    open: f64,
    high: f64,
    low: f64,
    close: f64,
    tick_volume: Option<f64>,
    #[serde(default)]
    spread: Option<f64>,
    #[serde(default)]
    real_volume: Option<f64>,
}

/// CSV record structure for Oanda format
#[derive(Debug, Deserialize)]
struct OandaCsvRecord {
    #[serde(rename = "Date")]
    date: String,
    #[serde(rename = "Time")]
    time: String,
    #[serde(rename = "BO")]
    bo: f64,  // Bid Open
    #[serde(rename = "BH")]
    bh: f64,  // Bid High
    #[serde(rename = "BL")]
    bl: f64,  // Bid Low
    #[serde(rename = "BC")]
    bc: f64,  // Bid Close
    #[serde(rename = "BCh")]
    bch: f64, // Bid Change
    #[serde(rename = "AO")]
    ao: f64,  // Ask Open
    #[serde(rename = "AH")]
    ah: f64,  // Ask High
    #[serde(rename = "AL")]
    al: f64,  // Ask Low
    #[serde(rename = "AC")]
    ac: f64,  // Ask Close
    #[serde(rename = "ACh")]
    ach: f64, // Ask Change
}

/// Data summary structure
#[derive(Debug, Clone, Serialize)]
pub struct DataSummary {
    pub available_pairs: HashMap<String, PathBuf>,
    pub total_files: usize,
    pub date_ranges: HashMap<String, (DateTime<Utc>, DateTime<Utc>)>,
}

/// Real-time data feed
pub struct RealTimeDataFeed {
    current_data: Vec<ForexDataPoint>,
    update_interval: std::time::Duration,
    pairs: Vec<String>,
}

impl RealTimeDataFeed {
    pub async fn from_config(config_path: &PathBuf) -> Result<Self> {
        // Load configuration from file
        let config_str = std::fs::read_to_string(config_path)?;
        let config: RealTimeFeedConfig = toml::from_str(&config_str)?;

        Ok(Self {
            current_data: Vec::new(),
            update_interval: std::time::Duration::from_millis(config.update_interval_ms),
            pairs: config.pairs,
        })
    }

    pub async fn default() -> Result<Self> {
        Ok(Self {
            current_data: Vec::new(),
            update_interval: std::time::Duration::from_millis(1000),
            pairs: vec!["EURUSD".to_string(), "GBPUSD".to_string(), "USDJPY".to_string()],
        })
    }

    /// Get current market data
    pub fn get_current_data(&self) -> &[ForexDataPoint] {
        &self.current_data
    }

    /// Update with new data point
    pub fn update_data(&mut self, data_point: ForexDataPoint) {
        self.current_data.push(data_point);

        // Keep only last 1000 points for performance
        if self.current_data.len() > 1000 {
            self.current_data.remove(0);
        }
    }

    /// Get update interval
    pub fn get_update_interval(&self) -> std::time::Duration {
        self.update_interval
    }

    /// Get monitored pairs
    pub fn get_pairs(&self) -> &[String] {
        &self.pairs
    }
}

/// Real-time feed configuration
#[derive(Debug, Deserialize)]
struct RealTimeFeedConfig {
    update_interval_ms: u64,
    pairs: Vec<String>,
    #[serde(default)]
    data_source: String,
}
