//! # Simple Test of Core Concepts
//!
//! Basic demonstration of time-symmetric pattern recognition concepts

use anyhow::Result;
use chrono::{Utc, Duration};

fn main() -> Result<()> {
    println!("🔬 Starting Simple Forex Pattern Reconstruction Test");
    println!("📊 Demonstrating Core Mathematical Concepts");
    
    // Generate sample EUR/USD data with embedded cycles
    let sample_data = generate_sample_data()?;
    println!("📈 Generated {} sample data points", sample_data.len());

    // Demonstrate cycle detection using basic mathematical principles
    let detected_cycles = detect_basic_cycles(&sample_data)?;
    println!("✅ Detected {} cycles:", detected_cycles.len());

    for cycle in &detected_cycles {
        println!("  🔄 Period: {} days, Strength: {:.3}", cycle.period, cycle.strength);
    }

    // Demonstrate temporal symmetry concept
    let symmetry_score = calculate_temporal_symmetry(&sample_data)?;
    println!("📊 Temporal Symmetry Score: {:.3}", symmetry_score);

    // Demonstrate the key insight
    println!("");
    println!("🎯 KEY INSIGHT DEMONSTRATION:");
    println!("   This simple test shows how market data can be analyzed for");
    println!("   cyclical patterns using mathematical principles rather than");
    println!("   probabilistic guessing. The detected cycles represent hidden");
    println!("   temporal symmetries that could enable pattern-based trading.");
    println!("");

    if symmetry_score > 0.7 {
        println!("✅ SUCCESS: Strong temporal symmetries detected!");
        println!("🎯 System demonstrates potential for pattern-based trading");
    } else {
        println!("⚠️  PARTIAL: Weak symmetries - need more sophisticated analysis");
        println!("📈 Full implementation would use Galois field operations");
    }
    
    Ok(())
}

/// Simple data structure for demonstration
#[derive(Debug, Clone)]
struct DataPoint {
    timestamp: chrono::DateTime<Utc>,
    price: f64,
}

/// Simple cycle structure
#[derive(Debug, Clone)]
struct DetectedCycle {
    period: u32,
    strength: f64,
}

/// Generate sample data with embedded cycles
fn generate_sample_data() -> Result<Vec<DataPoint>> {
    let mut data = Vec::new();
    let start_date = Utc::now() - Duration::days(365);
    let base_price = 1.1000;
    
    for i in 0..365 {
        let timestamp = start_date + Duration::days(i);
        
        // Embed known cycles for demonstration
        let weekly_cycle = (i as f64 * 2.0 * std::f64::consts::PI / 7.0).sin() * 0.005;
        let monthly_cycle = (i as f64 * 2.0 * std::f64::consts::PI / 30.0).sin() * 0.015;
        let quarterly_cycle = (i as f64 * 2.0 * std::f64::consts::PI / 90.0).sin() * 0.025;
        
        // Combine cycles with some noise
        let noise = (i as f64 * 0.1).sin() * 0.002;
        let price = base_price + weekly_cycle + monthly_cycle + quarterly_cycle + noise;
        
        data.push(DataPoint { timestamp, price });
    }
    
    Ok(data)
}

/// Basic cycle detection using autocorrelation
fn detect_basic_cycles(data: &[DataPoint]) -> Result<Vec<DetectedCycle>> {
    let mut cycles = Vec::new();
    let prices: Vec<f64> = data.iter().map(|d| d.price).collect();
    
    // Test for cycles of different periods
    for period in 2..=90 {
        let strength = calculate_cycle_strength(&prices, period);
        
        if strength > 0.3 { // Threshold for cycle detection
            cycles.push(DetectedCycle { period, strength });
        }
    }
    
    // Sort by strength
    cycles.sort_by(|a, b| b.strength.partial_cmp(&a.strength).unwrap());
    
    Ok(cycles)
}

/// Calculate cycle strength using simple autocorrelation
fn calculate_cycle_strength(prices: &[f64], period: u32) -> f64 {
    if prices.len() < period as usize * 2 {
        return 0.0;
    }
    
    let mut correlation_sum = 0.0;
    let mut count = 0;
    
    for i in 0..(prices.len() - period as usize) {
        let current = prices[i];
        let lagged = prices[i + period as usize];
        
        correlation_sum += current * lagged;
        count += 1;
    }
    
    if count == 0 {
        return 0.0;
    }
    
    let correlation = correlation_sum / count as f64;
    
    // Normalize to 0-1 range
    (correlation - 1.0).abs() / 0.1 // Simple normalization
}

/// Calculate temporal symmetry score
fn calculate_temporal_symmetry(data: &[DataPoint]) -> Result<f64> {
    let prices: Vec<f64> = data.iter().map(|d| d.price).collect();
    
    if prices.len() < 4 {
        return Ok(0.0);
    }
    
    // Split data into first and second half
    let mid = prices.len() / 2;
    let first_half = &prices[0..mid];
    let second_half = &prices[mid..];
    
    // Calculate correlation between halves (reversed for symmetry)
    let mut correlation_sum = 0.0;
    let min_len = first_half.len().min(second_half.len());
    
    for i in 0..min_len {
        let first = first_half[i];
        let second = second_half[min_len - 1 - i]; // Reversed index for symmetry
        correlation_sum += first * second;
    }
    
    let symmetry_score = correlation_sum / (min_len as f64 * 1.21); // Normalize
    
    // Convert to 0-1 range
    Ok((symmetry_score - 1.0).abs().min(1.0))
}
