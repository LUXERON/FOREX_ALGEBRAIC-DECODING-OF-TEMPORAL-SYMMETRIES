use anyhow::Result;
use nalgebra::DMatrix;
use std::collections::HashMap;
use chrono::{DateTime, Utc, Duration};

use crate::data::ForexDataPoint;

/// Cross-pair correlation analyzer for arbitrage opportunities
pub struct CrossPairAnalyzer {
    correlation_threshold: f64,
    arbitrage_threshold: f64,
}

#[derive(Debug, Clone)]
pub struct CorrelationResult {
    pub pair1: String,
    pub pair2: String,
    pub correlation: f64,
    pub strength: CorrelationStrength,
    pub arbitrage_potential: f64,
}

#[derive(Debug, Clone)]
pub enum CorrelationStrength {
    VeryStrong,  // |r| > 0.8
    Strong,      // 0.6 < |r| <= 0.8
    Moderate,    // 0.4 < |r| <= 0.6
    Weak,        // 0.2 < |r| <= 0.4
    VeryWeak,    // |r| <= 0.2
}

#[derive(Debug, Clone)]
pub struct ArbitrageOpportunity {
    pub primary_pair: String,
    pub correlated_pairs: Vec<String>,
    pub expected_move: f64,
    pub confidence: f64,
    pub time_window: Duration,
    pub profit_potential: f64,
}

impl CrossPairAnalyzer {
    pub fn new() -> Self {
        Self {
            correlation_threshold: 0.7,
            arbitrage_threshold: 0.001, // 10 pips
        }
    }

    /// Calculate correlation matrix for all currency pairs
    pub fn calculate_correlation_matrix(
        &self,
        data_map: &HashMap<String, Vec<ForexDataPoint>>
    ) -> Result<HashMap<(String, String), CorrelationResult>> {
        println!("🔗 Calculating cross-pair correlation matrix...");
        
        let pairs: Vec<String> = data_map.keys().cloned().collect();
        let mut correlations = HashMap::new();
        
        for i in 0..pairs.len() {
            for j in (i + 1)..pairs.len() {
                let pair1 = &pairs[i];
                let pair2 = &pairs[j];
                
                if let (Some(data1), Some(data2)) = (data_map.get(pair1), data_map.get(pair2)) {
                    let correlation = self.calculate_pearson_correlation(data1, data2)?;
                    let strength = self.classify_correlation_strength(correlation);
                    let arbitrage_potential = self.calculate_arbitrage_potential(data1, data2, correlation)?;
                    
                    let result = CorrelationResult {
                        pair1: pair1.clone(),
                        pair2: pair2.clone(),
                        correlation,
                        strength,
                        arbitrage_potential,
                    };
                    
                    correlations.insert((pair1.clone(), pair2.clone()), result);
                }
            }
        }
        
        println!("✅ Calculated {} correlation pairs", correlations.len());
        Ok(correlations)
    }

    /// Calculate Pearson correlation coefficient between two currency pairs
    fn calculate_pearson_correlation(
        &self,
        data1: &[ForexDataPoint],
        data2: &[ForexDataPoint]
    ) -> Result<f64> {
        // Align data by timestamp and extract close prices
        let aligned_data = self.align_data_by_timestamp(data1, data2);
        
        if aligned_data.len() < 30 {
            return Ok(0.0); // Not enough data for reliable correlation
        }
        
        let prices1: Vec<f64> = aligned_data.iter().map(|(p1, _)| p1.close).collect();
        let prices2: Vec<f64> = aligned_data.iter().map(|(_, p2)| p2.close).collect();
        
        // Calculate returns instead of raw prices for better correlation
        let returns1 = self.calculate_returns(&prices1);
        let returns2 = self.calculate_returns(&prices2);
        
        if returns1.len() != returns2.len() || returns1.is_empty() {
            return Ok(0.0);
        }
        
        // Pearson correlation formula
        let n = returns1.len() as f64;
        let mean1 = returns1.iter().sum::<f64>() / n;
        let mean2 = returns2.iter().sum::<f64>() / n;
        
        let mut numerator = 0.0;
        let mut sum_sq1 = 0.0;
        let mut sum_sq2 = 0.0;
        
        for i in 0..returns1.len() {
            let diff1 = returns1[i] - mean1;
            let diff2 = returns2[i] - mean2;
            
            numerator += diff1 * diff2;
            sum_sq1 += diff1 * diff1;
            sum_sq2 += diff2 * diff2;
        }
        
        let denominator = (sum_sq1 * sum_sq2).sqrt();
        
        if denominator == 0.0 {
            Ok(0.0)
        } else {
            Ok(numerator / denominator)
        }
    }

    /// Align two datasets by timestamp
    fn align_data_by_timestamp(
        &self,
        data1: &[ForexDataPoint],
        data2: &[ForexDataPoint]
    ) -> Vec<(ForexDataPoint, ForexDataPoint)> {
        let mut aligned = Vec::new();
        let mut i = 0;
        let mut j = 0;
        
        while i < data1.len() && j < data2.len() {
            let ts1 = data1[i].timestamp;
            let ts2 = data2[j].timestamp;
            
            if ts1 == ts2 {
                aligned.push((data1[i].clone(), data2[j].clone()));
                i += 1;
                j += 1;
            } else if ts1 < ts2 {
                i += 1;
            } else {
                j += 1;
            }
        }
        
        aligned
    }

    /// Calculate returns from price series
    fn calculate_returns(&self, prices: &[f64]) -> Vec<f64> {
        if prices.len() < 2 {
            return Vec::new();
        }
        
        prices.windows(2)
            .map(|window| (window[1] - window[0]) / window[0])
            .collect()
    }

    /// Classify correlation strength
    fn classify_correlation_strength(&self, correlation: f64) -> CorrelationStrength {
        let abs_corr = correlation.abs();
        
        if abs_corr > 0.8 {
            CorrelationStrength::VeryStrong
        } else if abs_corr > 0.6 {
            CorrelationStrength::Strong
        } else if abs_corr > 0.4 {
            CorrelationStrength::Moderate
        } else if abs_corr > 0.2 {
            CorrelationStrength::Weak
        } else {
            CorrelationStrength::VeryWeak
        }
    }

    /// Calculate arbitrage potential between two pairs
    fn calculate_arbitrage_potential(
        &self,
        data1: &[ForexDataPoint],
        data2: &[ForexDataPoint],
        correlation: f64
    ) -> Result<f64> {
        let aligned_data = self.align_data_by_timestamp(data1, data2);
        
        if aligned_data.len() < 10 {
            return Ok(0.0);
        }
        
        // Calculate price ratio deviations
        let ratios: Vec<f64> = aligned_data.iter()
            .map(|(p1, p2)| p1.close / p2.close)
            .collect();
        
        let mean_ratio = ratios.iter().sum::<f64>() / ratios.len() as f64;
        let std_dev = {
            let variance = ratios.iter()
                .map(|r| (r - mean_ratio).powi(2))
                .sum::<f64>() / ratios.len() as f64;
            variance.sqrt()
        };
        
        // Arbitrage potential based on correlation strength and ratio volatility
        let potential = correlation.abs() * std_dev * 1000.0; // Convert to pips
        
        Ok(potential)
    }

    /// Find arbitrage opportunities
    pub fn find_arbitrage_opportunities(
        &self,
        correlations: &HashMap<(String, String), CorrelationResult>,
        data_map: &HashMap<String, Vec<ForexDataPoint>>
    ) -> Result<Vec<ArbitrageOpportunity>> {
        println!("🎯 Analyzing arbitrage opportunities...");
        
        let mut opportunities = Vec::new();
        
        // Group highly correlated pairs
        let strong_correlations: Vec<&CorrelationResult> = correlations.values()
            .filter(|result| {
                matches!(result.strength, CorrelationStrength::VeryStrong | CorrelationStrength::Strong)
                && result.arbitrage_potential > self.arbitrage_threshold
            })
            .collect();
        
        for correlation in strong_correlations {
            let opportunity = ArbitrageOpportunity {
                primary_pair: correlation.pair1.clone(),
                correlated_pairs: vec![correlation.pair2.clone()],
                expected_move: correlation.arbitrage_potential,
                confidence: correlation.correlation.abs(),
                time_window: Duration::minutes(15), // 15-minute window
                profit_potential: correlation.arbitrage_potential * 0.7, // 70% of potential
            };
            
            opportunities.push(opportunity);
        }
        
        // Sort by profit potential
        opportunities.sort_by(|a, b| b.profit_potential.partial_cmp(&a.profit_potential).unwrap());
        
        println!("✅ Found {} arbitrage opportunities", opportunities.len());
        Ok(opportunities)
    }

    /// Print correlation analysis results
    pub fn print_correlation_analysis(&self, correlations: &HashMap<(String, String), CorrelationResult>) {
        println!("\n🔗 Cross-Pair Correlation Analysis:");
        println!("╔════════════╦════════════╦═════════════╦═══════════════╦═══════════════╗");
        println!("║   Pair 1   ║   Pair 2   ║ Correlation ║   Strength    ║ Arbitrage Pot ║");
        println!("╠════════════╬════════════╬═════════════╬═══════════════╬═══════════════╣");
        
        let mut sorted_correlations: Vec<_> = correlations.values().collect();
        sorted_correlations.sort_by(|a, b| b.correlation.abs().partial_cmp(&a.correlation.abs()).unwrap());
        
        for result in sorted_correlations.iter().take(15) {
            let strength_str = match result.strength {
                CorrelationStrength::VeryStrong => "Very Strong",
                CorrelationStrength::Strong => "Strong",
                CorrelationStrength::Moderate => "Moderate",
                CorrelationStrength::Weak => "Weak",
                CorrelationStrength::VeryWeak => "Very Weak",
            };
            
            println!("║ {:10} ║ {:10} ║ {:11.3} ║ {:13} ║ {:13.1} ║",
                     result.pair1, result.pair2, result.correlation, 
                     strength_str, result.arbitrage_potential * 10000.0); // Convert to pips
        }
        
        println!("╚════════════╩════════════╩═════════════╩═══════════════╩═══════════════╝");
    }

    /// Print arbitrage opportunities
    pub fn print_arbitrage_opportunities(&self, opportunities: &[ArbitrageOpportunity]) {
        println!("\n🎯 Arbitrage Opportunities:");
        println!("╔════════════╦═══════════════╦════════════╦═════════════╦═══════════════╗");
        println!("║ Primary    ║ Correlated    ║ Confidence ║ Time Window ║ Profit Pot.   ║");
        println!("╠════════════╬═══════════════╬════════════╬═════════════╬═══════════════╣");
        
        for opp in opportunities.iter().take(10) {
            let correlated = opp.correlated_pairs.join(", ");
            println!("║ {:10} ║ {:13} ║ {:10.1}% ║ {:11} ║ {:13.1} ║",
                     opp.primary_pair, 
                     if correlated.len() > 13 { &correlated[..10] } else { &correlated },
                     opp.confidence * 100.0,
                     format!("{}min", opp.time_window.num_minutes()),
                     opp.profit_potential * 10000.0); // Convert to pips
        }
        
        println!("╚════════════╩═══════════════╩════════════╩═════════════╩═══════════════╝");
    }
}
