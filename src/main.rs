//! # Forex Pattern Reconstruction System
//! 
//! Time-Symmetric Pattern Recognition using Galois Field Cyclicity
//! 
//! This system transforms forex trading from probabilistic guessing into 
//! algebraic decoding of temporal symmetries embedded in market data.

use anyhow::Result;
use clap::{Parser, Subcommand};
use tracing::{info, warn, error};
use std::path::PathBuf;

mod core;
mod data;
mod patterns;
mod galois;
mod symmetry;
mod backtest;
mod visualization;

use crate::core::TimeSymmetricEngine;
use crate::data::ForexDataManager;
use crate::patterns::PatternRecognizer;

/// Forex Pattern Reconstruction System
#[derive(Parser)]
#[command(name = "forex-pattern-analyzer")]
#[command(about = "Time-Symmetric Forex Pattern Recognition using Galois Field Cyclicity")]
#[command(version = "0.1.0")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
    
    /// Enable verbose logging
    #[arg(short, long)]
    verbose: bool,
    
    /// Configuration file path
    #[arg(short, long, default_value = "config.toml")]
    config: PathBuf,
}

#[derive(Subcommand)]
enum Commands {
    /// Analyze forex data for temporal symmetries
    Analyze {
        /// Input data file or directory
        #[arg(short, long)]
        input: PathBuf,
        
        /// Currency pair (e.g., EURUSD)
        #[arg(short, long, default_value = "EURUSD")]
        pair: String,
        
        /// Analysis timeframe
        #[arg(short, long, default_value = "1D")]
        timeframe: String,
        
        /// Output directory for results
        #[arg(short, long, default_value = "output")]
        output: PathBuf,
    },
    
    /// Run backtesting to validate temporal symmetries
    Backtest {
        /// Strategy configuration file
        #[arg(short, long)]
        strategy: PathBuf,
        
        /// Start date (YYYY-MM-DD)
        #[arg(long)]
        start_date: String,
        
        /// End date (YYYY-MM-DD)  
        #[arg(long)]
        end_date: String,
        
        /// Initial capital
        #[arg(long, default_value = "10000.0")]
        capital: f64,
    },
    
    /// Launch real-time pattern recognition dashboard
    Dashboard {
        /// Data feed configuration
        #[arg(short, long)]
        feed_config: Option<PathBuf>,
        
        /// Dashboard port
        #[arg(short, long, default_value = "8080")]
        port: u16,
    },
    
    /// Decompose EUR/USD data into cyclic components
    Decompose {
        /// EUR/USD data file
        #[arg(short, long)]
        data_file: PathBuf,
        
        /// Target cycles to extract (comma-separated)
        #[arg(short, long, default_value = "7,21,365,1277")]
        cycles: String,
        
        /// Output format (json, csv, plot)
        #[arg(short, long, default_value = "json")]
        format: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    
    // Initialize logging
    let log_level = if cli.verbose { "debug" } else { "info" };
    tracing_subscriber::fmt()
        .with_env_filter(format!("forex_pattern_reconstruction={}", log_level))
        .init();
    
    info!("🔬 Starting Forex Pattern Reconstruction System");
    info!("📊 Time-Symmetric Pattern Recognition Engine");
    
    // Load configuration
    let config = load_configuration(&cli.config).await?;
    
    match cli.command {
        Commands::Analyze { input, pair, timeframe, output } => {
            analyze_forex_patterns(input, pair, timeframe, output, config).await?;
        },
        
        Commands::Backtest { strategy, start_date, end_date, capital } => {
            run_backtest_validation(strategy, start_date, end_date, capital, config).await?;
        },
        
        Commands::Dashboard { feed_config, port } => {
            launch_pattern_dashboard(feed_config, port, config).await?;
        },
        
        Commands::Decompose { data_file, cycles, format } => {
            decompose_eur_usd_cycles(data_file, cycles, format, config).await?;
        },
    }
    
    Ok(())
}

/// Analyze forex data for temporal symmetries and hidden cycles
async fn analyze_forex_patterns(
    input: PathBuf,
    pair: String,
    timeframe: String,
    output: PathBuf,
    config: Configuration,
) -> Result<()> {
    info!("🔍 Analyzing {} patterns in {} timeframe", pair, timeframe);
    
    // Initialize data manager
    let mut data_manager = ForexDataManager::new(config.data_config)?;
    let forex_data = data_manager.load_data(&input, &pair, &timeframe).await?;
    
    info!("📈 Loaded {} data points from {} to {}", 
          forex_data.len(),
          forex_data.first().unwrap().timestamp,
          forex_data.last().unwrap().timestamp);
    
    // Initialize time-symmetric engine
    let mut engine = TimeSymmetricEngine::new(config.engine_config)?;
    engine.initialize().await?;
    
    // Initialize pattern recognizer
    let mut pattern_recognizer = PatternRecognizer::new(config.pattern_config)?;
    
    // Extract temporal symmetries
    info!("🔬 Extracting temporal symmetries...");
    let symmetries = engine.extract_temporal_symmetries(&forex_data).await?;
    
    info!("✅ Found {} temporal symmetries", symmetries.len());
    for symmetry in &symmetries {
        info!("  📊 {}: strength={:.3}, period={} days", 
              symmetry.name, symmetry.strength, symmetry.period_days);
    }
    
    // Detect hidden cycles
    info!("🔄 Detecting hidden cycles...");
    let cycles = pattern_recognizer.detect_cycles(&forex_data).await?;
    
    info!("✅ Detected {} hidden cycles", cycles.len());
    for cycle in &cycles {
        info!("  🔄 {}: period={} days, confidence={:.3}", 
              cycle.name, cycle.period, cycle.confidence);
    }
    
    // Generate analysis report
    let report = generate_analysis_report(&symmetries, &cycles, &forex_data)?;
    
    // Save results
    std::fs::create_dir_all(&output)?;
    let report_path = output.join(format!("{}_{}_analysis.json", pair, timeframe));
    std::fs::write(&report_path, serde_json::to_string_pretty(&report)?)?;
    
    info!("📄 Analysis report saved to: {}", report_path.display());
    
    // Generate visualizations
    if config.visualization_enabled {
        info!("📊 Generating visualizations...");
        visualization::generate_pattern_plots(&symmetries, &cycles, &forex_data, &output)?;
        info!("✅ Visualizations saved to: {}", output.display());
    }
    
    Ok(())
}

/// Run backtesting to validate temporal symmetries
async fn run_backtest_validation(
    strategy_path: PathBuf,
    start_date: String,
    end_date: String,
    initial_capital: f64,
    config: Configuration,
) -> Result<()> {
    info!("🧪 Running backtest validation from {} to {}", start_date, end_date);
    info!("💰 Initial capital: ${:.2}", initial_capital);
    
    // Load strategy configuration
    let strategy_config = backtest::load_strategy_config(&strategy_path)?;
    
    // Initialize backtesting engine
    let mut backtest_engine = backtest::BacktestEngine::new(
        strategy_config,
        initial_capital,
        config.backtest_config,
    )?;
    
    // Run temporal symmetry validation
    let validation_results = backtest_engine.validate_temporal_symmetries(
        &start_date,
        &end_date,
    ).await?;
    
    // Display results
    info!("📊 Backtest Results:");
    info!("  Total Return: {:.2}%", validation_results.total_return * 100.0);
    info!("  Sharpe Ratio: {:.2}", validation_results.sharpe_ratio);
    info!("  Max Drawdown: {:.2}%", validation_results.max_drawdown * 100.0);
    info!("  Symmetry Score: {:.3}", validation_results.symmetry_score);
    info!("  Pattern Consistency: {:.3}", validation_results.pattern_consistency);
    
    // Validate if system proves fundamental cycles
    if validation_results.proves_fundamental_cycles() {
        info!("✅ VALIDATION SUCCESS: System proves fundamental cyclical codes exist!");
        info!("🎯 Profitability achieved through decoded symmetries, not guessing");
    } else {
        warn!("⚠️  VALIDATION INCOMPLETE: Further optimization needed");
        info!("📈 Symmetry Score: {:.3} (target: >0.85)", validation_results.symmetry_score);
        info!("📊 Pattern Consistency: {:.3} (target: >0.80)", validation_results.pattern_consistency);
    }
    
    Ok(())
}

/// Launch real-time pattern recognition dashboard
async fn launch_pattern_dashboard(
    feed_config: Option<PathBuf>,
    port: u16,
    config: Configuration,
) -> Result<()> {
    info!("🚀 Launching real-time pattern recognition dashboard on port {}", port);
    
    // Initialize real-time data feed
    let data_feed = if let Some(feed_path) = feed_config {
        data::RealTimeDataFeed::from_config(&feed_path).await?
    } else {
        data::RealTimeDataFeed::default().await?
    };
    
    // Launch dashboard
    visualization::launch_tui_dashboard(data_feed, port, config.dashboard_config).await?;
    
    Ok(())
}

/// Decompose EUR/USD data into cyclic components
async fn decompose_eur_usd_cycles(
    data_file: PathBuf,
    cycles_str: String,
    format: String,
    config: Configuration,
) -> Result<()> {
    info!("🔬 Decomposing EUR/USD data into cyclic components");
    
    // Parse target cycles
    let target_cycles: Vec<u32> = cycles_str
        .split(',')
        .map(|s| s.trim().parse().unwrap())
        .collect();
    
    info!("🎯 Target cycles: {:?} days", target_cycles);
    
    // Load EUR/USD data
    let mut data_manager = ForexDataManager::new(config.data_config)?;
    let eur_usd_data = data_manager.load_eur_usd_data(&data_file).await?;
    
    info!("📈 Loaded {} EUR/USD data points", eur_usd_data.len());
    
    // Initialize cycle decomposer
    let mut decomposer = patterns::CycleDecomposer::new(config.decomposition_config)?;
    
    // Perform decomposition
    let decomposition = decomposer.decompose_cycles(&eur_usd_data, &target_cycles).await?;
    
    info!("✅ Decomposition complete:");
    for (cycle_period, component) in &decomposition.components {
        info!("  🔄 {}-day cycle: amplitude={:.4}, phase={:.2}°, strength={:.3}",
              cycle_period, component.amplitude, component.phase_degrees, component.strength);
    }
    
    // Save results in requested format
    match format.as_str() {
        "json" => {
            let json_output = serde_json::to_string_pretty(&decomposition)?;
            std::fs::write("eur_usd_decomposition.json", json_output)?;
            info!("💾 Results saved to: eur_usd_decomposition.json");
        },
        "csv" => {
            decomposition.save_to_csv("eur_usd_decomposition.csv")?;
            info!("💾 Results saved to: eur_usd_decomposition.csv");
        },
        "plot" => {
            visualization::plot_cycle_decomposition(&decomposition, "eur_usd_cycles.png")?;
            info!("📊 Plot saved to: eur_usd_cycles.png");
        },
        _ => {
            error!("❌ Unsupported format: {}", format);
            return Err(anyhow::anyhow!("Unsupported output format"));
        }
    }
    
    Ok(())
}

/// Load system configuration
async fn load_configuration(config_path: &PathBuf) -> Result<Configuration> {
    if config_path.exists() {
        info!("📋 Loading configuration from: {}", config_path.display());
        let config_str = std::fs::read_to_string(config_path)?;
        let config: Configuration = toml::from_str(&config_str)?;
        Ok(config)
    } else {
        info!("📋 Using default configuration");
        Ok(Configuration::default())
    }
}

/// Generate comprehensive analysis report
fn generate_analysis_report(
    symmetries: &[crate::symmetry::TemporalSymmetry],
    cycles: &[crate::patterns::HiddenCycle],
    data: &[crate::data::ForexDataPoint],
) -> Result<serde_json::Value> {
    let report = serde_json::json!({
        "analysis_timestamp": chrono::Utc::now(),
        "data_summary": {
            "total_points": data.len(),
            "date_range": {
                "start": data.first().unwrap().timestamp,
                "end": data.last().unwrap().timestamp,
            },
            "price_range": {
                "min": data.iter().map(|d| d.close).fold(f64::INFINITY, f64::min),
                "max": data.iter().map(|d| d.close).fold(f64::NEG_INFINITY, f64::max),
            }
        },
        "temporal_symmetries": symmetries,
        "hidden_cycles": cycles,
        "validation_metrics": {
            "symmetry_strength_avg": symmetries.iter().map(|s| s.strength).sum::<f64>() / symmetries.len() as f64,
            "cycle_confidence_avg": cycles.iter().map(|c| c.confidence).sum::<f64>() / cycles.len() as f64,
            "pattern_consistency": calculate_pattern_consistency(symmetries, cycles),
        }
    });
    
    Ok(report)
}

/// Calculate overall pattern consistency score
fn calculate_pattern_consistency(
    symmetries: &[crate::symmetry::TemporalSymmetry],
    cycles: &[crate::patterns::HiddenCycle],
) -> f64 {
    let symmetry_score = symmetries.iter().map(|s| s.strength).sum::<f64>() / symmetries.len() as f64;
    let cycle_score = cycles.iter().map(|c| c.confidence).sum::<f64>() / cycles.len() as f64;
    
    (symmetry_score + cycle_score) / 2.0
}

/// System configuration structure
#[derive(Debug, Clone, serde::Deserialize)]
struct Configuration {
    pub data_config: crate::data::DataConfig,
    pub engine_config: crate::core::EngineConfig,
    pub pattern_config: crate::patterns::PatternConfig,
    pub backtest_config: crate::backtest::BacktestConfig,
    pub dashboard_config: crate::visualization::DashboardConfig,
    pub decomposition_config: crate::patterns::DecompositionConfig,
    pub visualization_enabled: bool,
}

impl Default for Configuration {
    fn default() -> Self {
        Self {
            data_config: crate::data::DataConfig::default(),
            engine_config: crate::core::EngineConfig::default(),
            pattern_config: crate::patterns::PatternConfig::default(),
            backtest_config: crate::backtest::BacktestConfig::default(),
            dashboard_config: crate::visualization::DashboardConfig::default(),
            decomposition_config: crate::patterns::DecompositionConfig::default(),
            visualization_enabled: true,
        }
    }
}
