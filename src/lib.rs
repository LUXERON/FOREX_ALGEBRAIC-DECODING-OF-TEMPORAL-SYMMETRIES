//! # Forex Pattern Reconstruction Library
//! 
//! Time-symmetric pattern recognition for forex trading using Galois field theory

pub mod core;
pub mod data;
pub mod patterns;
pub mod galois;
pub mod symmetry;
pub mod backtest;
pub mod visualization;
pub mod dashboard;
pub mod synthetic;
pub mod anomaly;
pub mod laplacian_rl;
pub mod multi_currency;
pub mod embedded_db;
pub mod correlation;

// Re-export main types for convenience
pub use core::{TimeSymmetricEngine, EngineConfig};
pub use data::{ForexDataPoint, ForexDataManager, DataConfig};
pub use patterns::{PatternRecognizer, PatternConfig, HiddenCycle};
pub use symmetry::{TemporalSymmetry, SymmetryDetector};
pub use dashboard::{DashboardApp, render_dashboard};
