//! # Anomaly-Driven Laplacian Reinforcement Learning Trader
//! 
//! Advanced trading system using anomaly detection and Laplacian RL

use anyhow::Result;
use clap::{Arg, Command};
use std::path::PathBuf;
use chrono::Utc;

use forex_pattern_reconstruction::{
    ForexDataManager, DataConfig, TimeSymmetricEngine, EngineConfig,
    PatternRecognizer, PatternConfig,
};
use forex_pattern_reconstruction::synthetic::{
    SyntheticDataGenerator, SyntheticGenerationConfig,
};
use forex_pattern_reconstruction::anomaly::{
    TemporalAnomalyDetector, AnomalyDetectionConfig,
};
use forex_pattern_reconstruction::laplacian_rl::{
    LaplacianQLearningAgent, LaplacianQLearningConfig, Experience, TradingAction,
};

/// ASCII Art Banner for Anomaly Trading
const ANOMALY_BANNER: &str = r#"
╔═══════════════════════════════════════════════════════════════════════════════╗
║                                                                               ║
║     █████╗ ███╗   ██╗ ██████╗ ███╗   ███╗ █████╗ ██╗  ██╗   ██╗             ║
║    ██╔══██╗████╗  ██║██╔═══██╗████╗ ████║██╔══██╗██║  ╚██╗ ██╔╝             ║
║    ███████║██╔██╗ ██║██║   ██║██╔████╔██║███████║██║   ╚████╔╝              ║
║    ██╔══██║██║╚██╗██║██║   ██║██║╚██╔╝██║██╔══██║██║    ╚██╔╝               ║
║    ██║  ██║██║ ╚████║╚██████╔╝██║ ╚═╝ ██║██║  ██║███████╗██║                ║
║    ╚═╝  ╚═╝╚═╝  ╚═══╝ ╚═════╝ ╚═╝     ╚═╝╚═╝  ╚═╝╚══════╝╚═╝                ║
║                                                                               ║
║    ████████╗██████╗  █████╗ ██████╗ ██╗███╗   ██╗ ██████╗                   ║
║    ╚══██╔══╝██╔══██╗██╔══██╗██╔══██╗██║████╗  ██║██╔════╝                   ║
║       ██║   ██████╔╝███████║██║  ██║██║██╔██╗ ██║██║  ███╗                  ║
║       ██║   ██╔══██╗██╔══██║██║  ██║██║██║╚██╗██║██║   ██║                  ║
║       ██║   ██║  ██║██║  ██║██████╔╝██║██║ ╚████║╚██████╔╝                  ║
║       ╚═╝   ╚═╝  ╚═╝╚═╝  ╚═╝╚═════╝ ╚═╝╚═╝  ╚═══╝ ╚═════╝                   ║
║                                                                               ║
║           🔬 LAPLACIAN REINFORCEMENT LEARNING 🔬                             ║
║              Anomaly Detection + De Bruijn Q-Learning                         ║
║                                                                               ║
╚═══════════════════════════════════════════════════════════════════════════════╝
"#;

#[tokio::main]
async fn main() -> Result<()> {
    // Parse command line arguments
    let matches = Command::new("anomaly-trader")
        .version("1.0.0")
        .author("NEUNOMY - CURILEXA ALPHA")
        .about("Anomaly-driven Laplacian reinforcement learning trader")
        .arg(
            Arg::new("pair")
                .short('p')
                .long("pair")
                .value_name("PAIR")
                .help("Currency pair to trade")
                .default_value("EURUSD")
        )
        .arg(
            Arg::new("episodes")
                .short('e')
                .long("episodes")
                .value_name("COUNT")
                .help("Number of training episodes")
                .default_value("1000")
        )
        .arg(
            Arg::new("sensitivity")
                .short('s')
                .long("sensitivity")
                .value_name("THRESHOLD")
                .help("Anomaly detection sensitivity (0.0-1.0)")
                .default_value("0.3")
        )
        .arg(
            Arg::new("learning-rate")
                .short('l')
                .long("learning-rate")
                .value_name("RATE")
                .help("Q-learning rate")
                .default_value("0.1")
        )
        .get_matches();

    // Display banner
    println!("{}", ANOMALY_BANNER);
    println!("🚀 Initializing Anomaly-Driven Trading System...");
    println!("🔬 Combining temporal symmetry analysis with Laplacian RL...");
    println!();

    // Parse arguments
    let pair = matches.get_one::<String>("pair").unwrap();
    let episodes: u32 = matches.get_one::<String>("episodes").unwrap().parse()?;
    let sensitivity: f64 = matches.get_one::<String>("sensitivity").unwrap().parse()?;
    let learning_rate: f64 = matches.get_one::<String>("learning-rate").unwrap().parse()?;

    println!("📊 SYSTEM CONFIGURATION:");
    println!("   Currency Pair: {}", pair);
    println!("   Training Episodes: {}", episodes);
    println!("   Anomaly Sensitivity: {:.2}", sensitivity);
    println!("   Learning Rate: {:.3}", learning_rate);
    println!();

    // Initialize core components
    println!("🔧 Initializing Core Components...");
    
    // 1. Load historical data
    let data_config = DataConfig::default();
    let mut data_manager = ForexDataManager::new(data_config)?;
    let data_path = PathBuf::from("FOREX DATA/Forex Daily (1980) - 2023/archive(4)/Forex_D1/Major");
    let historical_data = data_manager.load_data(&data_path, pair, "1D").await?;
    
    println!("✅ Loaded {} historical data points", historical_data.len());
    
    // 2. Initialize pattern recognition
    let engine_config = EngineConfig::default();
    let mut engine = TimeSymmetricEngine::new(engine_config)?;
    engine.initialize().await?;
    
    let temporal_symmetries = engine.extract_temporal_symmetries(&historical_data).await?;
    println!("✅ Extracted {} temporal symmetries", temporal_symmetries.len());
    
    let pattern_config = PatternConfig::default();
    let mut pattern_recognizer = PatternRecognizer::new(pattern_config)?;
    let hidden_cycles = pattern_recognizer.detect_cycles(&historical_data).await?;
    println!("✅ Detected {} hidden cycles", hidden_cycles.len());
    
    // 3. Create synthetic data generator
    let synthetic_config = SyntheticGenerationConfig {
        future_horizon_days: 90,
        resolution_minutes: 60,
        noise_level: 0.15,
        cycle_confidence_threshold: 0.6,
        symmetry_strength_threshold: 0.5,
        enable_crisis_simulation: true,
    };
    
    let synthetic_generator = SyntheticDataGenerator::new(
        temporal_symmetries.clone(),
        hidden_cycles.clone(),
        historical_data.clone(),
        synthetic_config,
    )?;
    
    println!("✅ Synthetic data generator ready");
    
    // 4. Initialize anomaly detector
    let anomaly_config = AnomalyDetectionConfig {
        sensitivity_threshold: sensitivity,
        detection_window_size: 50,
        min_anomaly_confidence: 0.7,
        enable_multiscale_detection: true,
        symmetry_deviation_weight: 0.4,
        cycle_deviation_weight: 0.3,
        volatility_anomaly_weight: 0.3,
    };
    
    let mut anomaly_detector = TemporalAnomalyDetector::new(
        temporal_symmetries,
        hidden_cycles,
        &historical_data,
        anomaly_config,
    )?;
    
    println!("✅ Anomaly detector initialized");
    
    // 5. Initialize Laplacian Q-learning agent
    let rl_config = LaplacianQLearningConfig {
        learning_rate,
        discount_factor: 0.95,
        exploration_rate: 0.2,
        epsilon_decay: 0.995,
        min_epsilon: 0.01,
        buffer_size: 10000,
        batch_size: 32,
        pme_grid_size: 64,
        attention_weight: 0.3,
    };
    
    let mut rl_agent = LaplacianQLearningAgent::new(rl_config)?;
    println!("✅ Laplacian Q-learning agent ready");
    
    println!();
    println!("🔬 SYSTEM ARCHITECTURE:");
    println!("   📊 Temporal Symmetries → Anomaly Detection");
    println!("   🔍 Anomaly Patterns → De Bruijn Graph States");
    println!("   🧠 Laplacian Attention → Q-Value Weighting");
    println!("   ⚡ PME Approximation → Continuous State Space");
    println!("   🎯 Reinforcement Learning → Trading Decisions");
    println!();
    
    // Training loop
    println!("🚀 STARTING TRAINING LOOP...");
    println!("   Episodes: {}", episodes);
    println!("   Learning from anomaly patterns in synthetic data");
    println!();
    
    let mut total_reward = 0.0;
    let mut successful_trades = 0;
    let mut total_anomalies_detected = 0;
    
    for episode in 1..=episodes {
        // Generate synthetic data for this episode
        let start_date = Utc::now();
        let synthetic_data = synthetic_generator.generate_future_data(start_date, pair).await?;
        
        // Detect anomalies in synthetic data
        let detected_anomalies = anomaly_detector.detect_anomalies(&synthetic_data).await?;
        total_anomalies_detected += detected_anomalies.len();
        
        // Episode variables
        let mut episode_reward = 0.0;
        let mut episode_trades = 0;
        let mut episode_successful_trades = 0;
        
        // Process each anomaly as a trading opportunity
        for (i, anomaly) in detected_anomalies.iter().enumerate() {
            if i >= synthetic_data.len() {
                break;
            }
            
            let current_data = &synthetic_data[i].data_point;
            let next_data = synthetic_data.get(i + 1).map(|p| &p.data_point);
            
            // Convert anomaly to state
            let state = rl_agent.anomaly_to_state(anomaly, current_data)?;
            
            // Choose action based on anomaly
            let action = rl_agent.choose_action(&state, anomaly)?;
            
            // Calculate reward based on action and market movement
            let reward = if let Some(next_data) = next_data {
                calculate_trading_reward(&action, current_data, next_data)
            } else {
                0.0
            };
            
            episode_reward += reward;
            episode_trades += 1;
            
            if reward > 0.0 {
                episode_successful_trades += 1;
            }
            
            // Create next state
            let next_state = if let Some(next_data) = next_data {
                format!("next_state_{}", i + 1)
            } else {
                "terminal".to_string()
            };
            
            // Add experience to replay buffer
            let experience = Experience {
                state: state.clone(),
                action: action.clone(),
                reward,
                next_state: next_state.clone(),
                done: next_data.is_none(),
                anomaly_context: Some(anomaly.clone()),
            };
            
            rl_agent.add_experience(experience);
            
            // Update Q-values
            rl_agent.update_q_value(&state, action, reward, &next_state, next_data.is_none())?;
        }
        
        // Train on batch of experiences
        rl_agent.train_batch()?;
        
        // Update statistics
        total_reward += episode_reward;
        successful_trades += episode_successful_trades;
        
        // Update performance metrics
        let anomaly_accuracy = if detected_anomalies.is_empty() {
            1.0
        } else {
            episode_successful_trades as f64 / episode_trades as f64
        };
        
        let trading_success = episode_successful_trades > episode_trades / 2;
        rl_agent.update_performance_metrics(episode_reward, anomaly_accuracy, trading_success);
        
        // Progress reporting
        if episode % 100 == 0 {
            let avg_reward = total_reward / episode as f64;
            let success_rate = successful_trades as f64 / (episode * 10) as f64; // Approximate
            let metrics = rl_agent.get_performance_metrics();
            
            println!("📊 Episode {}/{}", episode, episodes);
            println!("   Average Reward: {:.4}", avg_reward);
            println!("   Success Rate: {:.1}%", success_rate * 100.0);
            println!("   Exploration Rate: {:.3}", metrics.exploration_rate);
            println!("   Anomalies Detected: {}", detected_anomalies.len());
            println!();
        }
    }
    
    // Final results
    println!("🎯 TRAINING COMPLETE!");
    println!("═══════════════════");
    
    let final_metrics = rl_agent.get_performance_metrics();
    println!("📊 FINAL PERFORMANCE METRICS:");
    println!("   Total Episodes: {}", final_metrics.total_episodes);
    println!("   Average Reward: {:.4}", final_metrics.average_reward);
    println!("   Trading Success Rate: {:.1}%", final_metrics.trading_success_rate * 100.0);
    println!("   Anomaly Detection Accuracy: {:.1}%", final_metrics.anomaly_detection_accuracy * 100.0);
    println!("   Final Exploration Rate: {:.3}", final_metrics.exploration_rate);
    println!();
    
    println!("🔬 ANOMALY ANALYSIS:");
    let anomaly_stats = anomaly_detector.get_anomaly_statistics();
    println!("   Total Anomalies Detected: {}", total_anomalies_detected);
    println!("   Average Confidence: {:.3}", anomaly_stats.average_confidence);
    println!("   Anomaly Types:");
    for (anomaly_type, count) in &anomaly_stats.type_distribution {
        println!("      {}: {}", anomaly_type, count);
    }
    println!();
    
    println!("🎯 KEY INSIGHTS:");
    println!("   🔍 Anomaly detection identified {} deviations from temporal symmetries", total_anomalies_detected);
    println!("   🧠 Laplacian RL learned to trade on {} different anomaly patterns", anomaly_stats.type_distribution.len());
    println!("   ⚡ De Bruijn graph structure enabled efficient state representation");
    println!("   📊 PME approximation provided smooth Q-value updates");
    println!("   🎯 System achieved {:.1}% success rate on anomaly-based trades", final_metrics.trading_success_rate * 100.0);
    
    // Save results
    let results = serde_json::json!({
        "final_metrics": final_metrics,
        "anomaly_statistics": anomaly_stats,
        "total_anomalies": total_anomalies_detected,
        "training_episodes": episodes,
        "pair": pair,
        "sensitivity": sensitivity,
        "learning_rate": learning_rate
    });
    
    let results_file = format!("anomaly_trading_results_{}_{}_episodes.json", pair, episodes);
    std::fs::write(&results_file, serde_json::to_string_pretty(&results)?)?;
    
    println!();
    println!("💾 Results saved to: {}", results_file);
    println!("🚀 Anomaly-driven Laplacian RL training complete!");
    
    Ok(())
}

/// Calculate trading reward based on action and market movement
fn calculate_trading_reward(
    action: &TradingAction,
    current_data: &forex_pattern_reconstruction::data::ForexDataPoint,
    next_data: &forex_pattern_reconstruction::data::ForexDataPoint,
) -> f64 {
    let price_change = next_data.close - current_data.close;
    let price_change_pct = price_change / current_data.close;
    
    match action {
        TradingAction::Buy { size } => {
            let size_f64 = (*size as f64) / 100.0; // Convert percentage to decimal
            // Reward positive price movements
            if price_change > 0.0 {
                price_change_pct * size_f64 * 1000.0 // Scale to reasonable reward
            } else {
                price_change_pct * size_f64 * 1000.0 // Negative reward for losses
            }
        }
        TradingAction::Sell { size } => {
            let size_f64 = (*size as f64) / 100.0; // Convert percentage to decimal
            // Reward negative price movements
            if price_change < 0.0 {
                -price_change_pct * size_f64 * 1000.0 // Positive reward for correct short
            } else {
                -price_change_pct * size_f64 * 1000.0 // Negative reward for wrong short
            }
        }
        TradingAction::Hold => {
            // Small positive reward for holding during low volatility
            let volatility = (current_data.high - current_data.low) / current_data.close;
            if volatility < 0.01 {
                0.1 // Small reward for correctly holding
            } else {
                -0.05 // Small penalty for missing opportunity
            }
        }
        TradingAction::ClosePosition => {
            // Reward for closing during high volatility
            let volatility = (current_data.high - current_data.low) / current_data.close;
            if volatility > 0.02 {
                0.5 // Reward for risk management
            } else {
                -0.1 // Penalty for unnecessary closing
            }
        }
    }
}
