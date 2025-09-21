use anyhow::Result;
use chrono::{DateTime, Utc};
use flate2::Compression;
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::{Read, Write};

use crate::data::ForexDataPoint;

/// Compressed binary forex data point for efficient storage
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CompressedForexPoint {
    pub timestamp: i64, // Unix timestamp for efficient storage
    pub open: u32,      // Price * 100000 as integer
    pub high: u32,
    pub low: u32,
    pub close: u32,
    pub volume: u32,
}

impl From<&ForexDataPoint> for CompressedForexPoint {
    fn from(point: &ForexDataPoint) -> Self {
        Self {
            timestamp: point.timestamp.timestamp(),
            open: (point.open * 100000.0) as u32,
            high: (point.high * 100000.0) as u32,
            low: (point.low * 100000.0) as u32,
            close: (point.close * 100000.0) as u32,
            volume: point.volume.unwrap_or(0.0) as u32,
        }
    }
}

impl Into<ForexDataPoint> for CompressedForexPoint {
    fn into(self) -> ForexDataPoint {
        ForexDataPoint {
            timestamp: DateTime::from_timestamp(self.timestamp, 0).unwrap_or_else(|| Utc::now()),
            open: self.open as f64 / 100000.0,
            high: self.high as f64 / 100000.0,
            low: self.low as f64 / 100000.0,
            close: self.close as f64 / 100000.0,
            volume: Some(self.volume as f64),
        }
    }
}

/// Embedded SQLite database for forex data
pub struct EmbeddedForexDB {
    conn: Connection,
}

impl EmbeddedForexDB {
    /// Create new embedded database in memory
    pub fn new() -> Result<Self> {
        let conn = Connection::open(":memory:")?;
        
        // Create tables
        conn.execute(
            "CREATE TABLE forex_data (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                pair TEXT NOT NULL,
                data BLOB NOT NULL,
                data_points INTEGER NOT NULL,
                created_at INTEGER NOT NULL
            )",
            [],
        )?;

        conn.execute(
            "CREATE INDEX idx_pair ON forex_data(pair)",
            [],
        )?;

        conn.execute(
            "CREATE TABLE correlation_matrix (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                pair1 TEXT NOT NULL,
                pair2 TEXT NOT NULL,
                correlation REAL NOT NULL,
                timeframe TEXT NOT NULL,
                created_at INTEGER NOT NULL
            )",
            [],
        )?;

        conn.execute(
            "CREATE INDEX idx_correlation ON correlation_matrix(pair1, pair2)",
            [],
        )?;

        Ok(Self { conn })
    }

    /// Store compressed forex data for a currency pair
    pub fn store_forex_data(&self, pair: &str, data: &[ForexDataPoint]) -> Result<()> {
        println!("📦 Compressing and storing {} data points for {}", data.len(), pair);
        
        // Convert to compressed format
        let compressed_data: Vec<CompressedForexPoint> = data.iter()
            .map(|point| CompressedForexPoint::from(point))
            .collect();

        // Serialize and compress
        let serialized = bincode::serialize(&compressed_data)?;
        let mut encoder = GzEncoder::new(Vec::new(), Compression::best());
        encoder.write_all(&serialized)?;
        let compressed_blob = encoder.finish()?;

        // Store in database
        self.conn.execute(
            "INSERT INTO forex_data (pair, data, data_points, created_at) VALUES (?1, ?2, ?3, ?4)",
            params![pair, compressed_blob, data.len(), Utc::now().timestamp()],
        )?;

        let compression_ratio = (serialized.len() as f64 / compressed_blob.len() as f64) * 100.0;
        println!("✅ {} stored: {} points, {:.1}% compression ratio", 
                 pair, data.len(), compression_ratio);

        Ok(())
    }

    /// Retrieve forex data for a currency pair
    pub fn get_forex_data(&self, pair: &str) -> Result<Vec<ForexDataPoint>> {
        let mut stmt = self.conn.prepare(
            "SELECT data FROM forex_data WHERE pair = ?1 ORDER BY created_at DESC LIMIT 1"
        )?;

        let compressed_blob: Vec<u8> = stmt.query_row(params![pair], |row| {
            Ok(row.get(0)?)
        })?;

        // Decompress and deserialize
        let mut decoder = GzDecoder::new(&compressed_blob[..]);
        let mut decompressed = Vec::new();
        decoder.read_to_end(&mut decompressed)?;

        let compressed_data: Vec<CompressedForexPoint> = bincode::deserialize(&decompressed)?;
        
        // Convert back to ForexDataPoint
        let forex_data: Vec<ForexDataPoint> = compressed_data.into_iter()
            .map(|point| point.into())
            .collect();

        println!("📊 Retrieved {} data points for {}", forex_data.len(), pair);
        Ok(forex_data)
    }

    /// Store correlation matrix
    pub fn store_correlation(&self, pair1: &str, pair2: &str, correlation: f64, timeframe: &str) -> Result<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO correlation_matrix (pair1, pair2, correlation, timeframe, created_at) 
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![pair1, pair2, correlation, timeframe, Utc::now().timestamp()],
        )?;
        Ok(())
    }

    /// Get correlation matrix for all pairs
    pub fn get_correlation_matrix(&self, timeframe: &str) -> Result<HashMap<(String, String), f64>> {
        let mut stmt = self.conn.prepare(
            "SELECT pair1, pair2, correlation FROM correlation_matrix WHERE timeframe = ?1"
        )?;

        let rows = stmt.query_map(params![timeframe], |row| {
            Ok((
                (row.get::<_, String>(0)?, row.get::<_, String>(1)?),
                row.get::<_, f64>(2)?
            ))
        })?;

        let mut correlations = HashMap::new();
        for row in rows {
            let ((pair1, pair2), correlation) = row?;
            correlations.insert((pair1, pair2), correlation);
        }

        Ok(correlations)
    }

    /// Get database statistics
    pub fn get_stats(&self) -> Result<()> {
        let mut stmt = self.conn.prepare(
            "SELECT pair, data_points, LENGTH(data) as blob_size FROM forex_data ORDER BY pair"
        )?;

        let rows = stmt.query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, i64>(1)?,
                row.get::<_, i64>(2)?
            ))
        })?;

        println!("\n📊 Embedded Database Statistics:");
        println!("╔══════════╦═════════════╦═════════════╗");
        println!("║   Pair   ║ Data Points ║  Size (KB)  ║");
        println!("╠══════════╬═════════════╬═════════════╣");

        let mut total_points = 0;
        let mut total_size = 0;

        for row in rows {
            let (pair, points, size) = row?;
            println!("║ {:8} ║ {:11} ║ {:11} ║", pair, points, size / 1024);
            total_points += points;
            total_size += size;
        }

        println!("╠══════════╬═════════════╬═════════════╣");
        println!("║  TOTAL   ║ {:11} ║ {:11} ║", total_points, total_size / 1024);
        println!("╚══════════╩═════════════╩═════════════╝");

        Ok(())
    }
}
