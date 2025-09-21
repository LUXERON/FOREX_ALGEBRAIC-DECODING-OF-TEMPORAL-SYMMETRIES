use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::sync::RwLock;
use chrono::{DateTime, Utc};

use crate::{
    core::{TimeSymmetricEngine, EngineConfig},
    data::{ForexDataManager, DataConfig, ForexDataPoint},
    patterns::{PatternRecognizer, PatternConfig, HiddenCycle},
    symmetry::TemporalSymmetry,
    synthetic::{SyntheticDataGenerator, SyntheticForexPoint, SyntheticGenerationConfig},
    anomaly::{TemporalAnomalyDetector, DetectedAnomaly, AnomalyDetectionConfig},
    laplacian_rl::{LaplacianQLearningAgent, TradingAction, LaplacianQLearningConfig},
};

/// Multi-currency trading pair configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurrencyPairConfig {
    pub symbol: String,
    pub base_currency: String,
    pub quote_currency: String,
    pub pip_value: f64,
    pub spread: f64,
    pub min_lot_size: f64,
    pub max_lot_size: f64,
    pub enabled: bool,
}

impl Default for CurrencyPairConfig {
    fn default() -> Self {
        Self {
            symbol: "EURUSD".to_string(),
            base_currency: "EUR".to_string(),
            quote_currency: "USD".to_string(),
            pip_value: 0.0001,
            spread: 0.0002,
            min_lot_size: 0.01,
            max_lot_size: 100.0,
            enabled: true,
        }
    }
}

/// Performance metrics for a currency pair
#[derive(Debug, Clone, Serialize)]
pub struct PairPerformanceMetrics {
    pub symbol: String,
    pub total_trades: u64,
    pub successful_trades: u64,
    pub total_reward: f64,
    pub average_reward: f64,
    pub max_drawdown: f64,
    pub sharpe_ratio: f64,
    pub win_rate: f64,
    pub anomalies_detected: u64,
    pub last_updated: DateTime<Utc>,
}

impl PairPerformanceMetrics {
    pub fn new(symbol: String) -> Self {
        Self {
            symbol,
            total_trades: 0,
            successful_trades: 0,
            total_reward: 0.0,
            average_reward: 0.0,
            max_drawdown: 0.0,
            sharpe_ratio: 0.0,
            win_rate: 0.0,
            anomalies_detected: 0,
            last_updated: Utc::now(),
        }
    }
    
    pub fn update_metrics(&mut self, reward: f64, is_successful: bool) {
        self.total_trades += 1;
        if is_successful {
            self.successful_trades += 1;
        }
        
        self.total_reward += reward;
        self.average_reward = self.total_reward / self.total_trades as f64;
        self.win_rate = (self.successful_trades as f64 / self.total_trades as f64) * 100.0;
        self.last_updated = Utc::now();
    }
}

/// Multi-currency trading system state
pub struct CurrencyPairState {
    pub config: CurrencyPairConfig,
    pub engine: TimeSymmetricEngine,
    pub data_manager: ForexDataManager,
    pub pattern_recognizer: PatternRecognizer,
    pub synthetic_generator: SyntheticDataGenerator,
    pub anomaly_detector: TemporalAnomalyDetector,
    pub rl_agent: LaplacianQLearningAgent,
    pub performance: PairPerformanceMetrics,
    pub historical_data: Vec<ForexDataPoint>,
    pub synthetic_data: Vec<SyntheticForexPoint>,
    pub recent_anomalies: Vec<DetectedAnomaly>,
    pub is_active: bool,
}

impl CurrencyPairState {
    pub async fn new(config: CurrencyPairConfig) -> Result<Self> {
        let engine_config = EngineConfig::default();
        let engine = TimeSymmetricEngine::new(engine_config)?;
        
        let data_config = DataConfig::default();
        let data_manager = ForexDataManager::new(data_config)?;
        
        let pattern_config = PatternConfig::default();
        let pattern_recognizer = PatternRecognizer::new(pattern_config)?;
        
        // Initialize with empty data - will be populated during initialization
        let synthetic_generator = SyntheticDataGenerator::new(
            Vec::new(),
            Vec::new(),
            Vec::new(),
            SyntheticGenerationConfig::default()
        )?;
        
        let anomaly_detector = TemporalAnomalyDetector::new(
            Vec::new(),
            Vec::new(),
            &[],
            AnomalyDetectionConfig::default()
        )?;
        
        let rl_config = LaplacianQLearningConfig::default();
        let rl_agent = LaplacianQLearningAgent::new(rl_config)?;
        
        let performance = PairPerformanceMetrics::new(config.symbol.clone());
        
        Ok(Self {
            config,
            engine,
            data_manager,
            pattern_recognizer,
            synthetic_generator,
            anomaly_detector,
            rl_agent,
            performance,
            historical_data: Vec::new(),
            synthetic_data: Vec::new(),
            recent_anomalies: Vec::new(),
            is_active: false,
        })
    }
    
    /// Initialize the currency pair with historical data
    pub async fn initialize(&mut self) -> Result<()> {
        println!("🔄 Initializing {} trading system...", self.config.symbol);
        
        // Load historical data
        let data_path = std::path::PathBuf::from("FOREX DATA/Forex Daily (1980) - 2023/archive(4)/Forex_D1/Major");
        self.historical_data = self.data_manager.load_data(&data_path, &self.config.symbol, "1D").await?;
        println!("✅ {} - Loaded {} historical data points", self.config.symbol, self.historical_data.len());
        
        // Initialize engine
        self.engine.initialize().await?;
        
        // Extract temporal symmetries
        let symmetries = self.engine.extract_temporal_symmetries(&self.historical_data).await?;
        println!("✅ {} - Extracted {} temporal symmetries", self.config.symbol, symmetries.len());
        
        // Detect hidden cycles
        let cycles = self.pattern_recognizer.detect_cycles(&self.historical_data).await?;
        println!("✅ {} - Detected {} hidden cycles", self.config.symbol, cycles.len());
        
        // Reinitialize synthetic generator with actual data
        self.synthetic_generator = SyntheticDataGenerator::new(
            symmetries.clone(),
            cycles.clone(),
            self.historical_data.clone(),
            SyntheticGenerationConfig::default()
        )?;

        // Generate synthetic data
        let start_date = chrono::Utc::now();
        self.synthetic_data = self.synthetic_generator.generate_future_data(start_date, &self.config.symbol).await?;
        println!("✅ {} - Generated {} synthetic data points", self.config.symbol, self.synthetic_data.len());

        // Reinitialize anomaly detector with actual data
        self.anomaly_detector = TemporalAnomalyDetector::new(
            symmetries.clone(),
            cycles.clone(),
            &self.historical_data,
            AnomalyDetectionConfig::default()
        )?;

        self.is_active = true;
        println!("🎯 {} trading system initialized successfully!", self.config.symbol);
        
        Ok(())
    }
    
    /// Process new market data and generate trading signals
    pub async fn process_market_update(&mut self) -> Result<Vec<TradingAction>> {
        if !self.is_active {
            return Ok(Vec::new());
        }
        
        let mut actions = Vec::new();
        
        // Detect anomalies in recent synthetic data
        if self.synthetic_data.len() >= 10 {
            let recent_data = self.synthetic_data.iter().rev().take(50).cloned().collect::<Vec<_>>();
            let anomalies = self.anomaly_detector.detect_anomalies(&recent_data).await?;
            
            for anomaly in anomalies {
                self.performance.anomalies_detected += 1;
                self.recent_anomalies.push(anomaly.clone());
                
                // Keep only last 100 anomalies
                if self.recent_anomalies.len() > 100 {
                    self.recent_anomalies.remove(0);
                }
                
                // Generate trading action based on anomaly
                let state_id = format!("{}_{}", self.config.symbol, self.performance.total_trades);
                let action = self.rl_agent.choose_action(&state_id, &anomaly)?;
                actions.push(action);
            }
        }
        
        Ok(actions)
    }
    
    /// Update performance metrics with trade result
    pub fn update_performance(&mut self, reward: f64) {
        let is_successful = reward > 0.0;
        self.performance.update_metrics(reward, is_successful);
    }
}

/// Multi-currency trading system manager
pub struct MultiCurrencyManager {
    pub pairs: RwLock<HashMap<String, CurrencyPairState>>,
    pub active_pairs: Vec<String>,
    pub global_performance: RwLock<HashMap<String, PairPerformanceMetrics>>,
}

impl MultiCurrencyManager {
    /// Create new multi-currency manager
    pub fn new() -> Self {
        Self {
            pairs: RwLock::new(HashMap::new()),
            active_pairs: Vec::new(),
            global_performance: RwLock::new(HashMap::new()),
        }
    }
    
    /// Initialize with major currency pairs
    pub async fn initialize_major_pairs(&mut self) -> Result<()> {
        let major_pairs = vec![
            CurrencyPairConfig { symbol: "EURUSD".to_string(), base_currency: "EUR".to_string(), quote_currency: "USD".to_string(), ..Default::default() },
            CurrencyPairConfig { symbol: "GBPUSD".to_string(), base_currency: "GBP".to_string(), quote_currency: "USD".to_string(), ..Default::default() },
            CurrencyPairConfig { symbol: "USDJPY".to_string(), base_currency: "USD".to_string(), quote_currency: "JPY".to_string(), pip_value: 0.01, ..Default::default() },
            CurrencyPairConfig { symbol: "USDCHF".to_string(), base_currency: "USD".to_string(), quote_currency: "CHF".to_string(), ..Default::default() },
            CurrencyPairConfig { symbol: "USDCAD".to_string(), base_currency: "USD".to_string(), quote_currency: "CAD".to_string(), ..Default::default() },
            CurrencyPairConfig { symbol: "EURGBP".to_string(), base_currency: "EUR".to_string(), quote_currency: "GBP".to_string(), ..Default::default() },
            CurrencyPairConfig { symbol: "EURJPY".to_string(), base_currency: "EUR".to_string(), quote_currency: "JPY".to_string(), pip_value: 0.01, ..Default::default() },
        ];
        
        let mut pairs_map = self.pairs.write().await;
        let mut performance_map = self.global_performance.write().await;
        
        for config in major_pairs {
            let symbol = config.symbol.clone();
            self.active_pairs.push(symbol.clone());
            
            let pair_state = CurrencyPairState::new(config).await?;
            performance_map.insert(symbol.clone(), PairPerformanceMetrics::new(symbol.clone()));
            pairs_map.insert(symbol, pair_state);
        }
        
        println!("🌍 Multi-currency manager initialized with {} major pairs", self.active_pairs.len());
        Ok(())
    }
    
    /// Initialize all currency pairs with historical data
    pub async fn initialize_all_pairs(&mut self) -> Result<()> {
        let mut pairs_map = self.pairs.write().await;
        
        for symbol in &self.active_pairs {
            if let Some(pair_state) = pairs_map.get_mut(symbol) {
                pair_state.initialize().await?;
            }
        }
        
        println!("🚀 All currency pairs initialized successfully!");
        Ok(())
    }
    
    /// Get performance summary for all pairs
    pub async fn get_performance_summary(&self) -> HashMap<String, PairPerformanceMetrics> {
        let performance_map = self.global_performance.read().await;
        performance_map.clone()
    }
    
    /// Process market updates for all active pairs
    pub async fn process_all_market_updates(&mut self) -> Result<HashMap<String, Vec<TradingAction>>> {
        let mut all_actions = HashMap::new();
        let mut pairs_map = self.pairs.write().await;
        
        for symbol in &self.active_pairs {
            if let Some(pair_state) = pairs_map.get_mut(symbol) {
                let actions = pair_state.process_market_update().await?;
                if !actions.is_empty() {
                    all_actions.insert(symbol.clone(), actions);
                }
            }
        }
        
        Ok(all_actions)
    }
}
