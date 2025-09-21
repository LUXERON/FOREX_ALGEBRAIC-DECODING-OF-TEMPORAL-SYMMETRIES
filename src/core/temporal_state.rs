//! # Temporal State Representation
//! 
//! Representation of market states across past, present, and future coordinates.

use anyhow::Result;
use nalgebra::DVector;
use serde::{Deserialize, Serialize};

use crate::data::ForexDataPoint;

/// Temporal state representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemporalState {
    pub past_encoding: Vec<f64>,
    pub present_transform: Vec<f64>,
    pub future_extension: Vec<f64>,
    pub pattern_strength: Option<f64>,
    pub coherence_score: f64,
}

impl TemporalState {
    /// Create temporal state from forex data
    pub fn from_forex_data(
        current: &ForexDataPoint,
        past_context: Option<&[ForexDataPoint]>,
        future_context: Option<&[ForexDataPoint]>,
    ) -> Result<Self> {
        let past_encoding = if let Some(past) = past_context {
            encode_price_sequence(past)?
        } else {
            vec![0.0; 10] // Default encoding
        };
        
        let present_transform = vec![
            current.open,
            current.high,
            current.low,
            current.close,
            current.close - current.open, // Change
            (current.high - current.low) / current.close, // Volatility
        ];
        
        let future_extension = if let Some(future) = future_context {
            encode_price_sequence(future)?
        } else {
            vec![0.0; 10] // Default encoding
        };
        
        let coherence_score = compute_coherence_score(&past_encoding, &present_transform, &future_extension);
        
        Ok(Self {
            past_encoding,
            present_transform,
            future_extension,
            pattern_strength: None,
            coherence_score,
        })
    }
    
    /// Compute similarity with another temporal state
    pub fn compute_similarity(&self, other: &TemporalState) -> f64 {
        let past_sim = compute_vector_similarity(&self.past_encoding, &other.past_encoding);
        let present_sim = compute_vector_similarity(&self.present_transform, &other.present_transform);
        let future_sim = compute_vector_similarity(&self.future_extension, &other.future_extension);
        
        (past_sim + present_sim + future_sim) / 3.0
    }
}

/// Temporal state space manager
pub struct TemporalStateSpace {
    coherence_window: usize,
    states: Vec<TemporalState>,
}

impl TemporalStateSpace {
    pub fn new(coherence_window: usize) -> Result<Self> {
        Ok(Self {
            coherence_window,
            states: Vec::new(),
        })
    }
    
    pub async fn initialize(&mut self) -> Result<()> {
        self.states.reserve(self.coherence_window);
        Ok(())
    }
    
    pub fn add_state(&mut self, state: TemporalState) {
        self.states.push(state);
        if self.states.len() > self.coherence_window {
            self.states.remove(0);
        }
    }
    
    pub fn get_coherence_trend(&self) -> Option<f64> {
        if self.states.len() < 2 {
            return None;
        }
        
        let recent_coherence: f64 = self.states.iter()
            .rev()
            .take(10)
            .map(|s| s.coherence_score)
            .sum::<f64>() / 10.0;
            
        Some(recent_coherence)
    }
}

/// Encode price sequence into vector representation
fn encode_price_sequence(prices: &[ForexDataPoint]) -> Result<Vec<f64>> {
    if prices.is_empty() {
        return Ok(vec![0.0; 10]);
    }
    
    let mut encoding = Vec::new();
    
    // Basic price statistics
    let closes: Vec<f64> = prices.iter().map(|p| p.close).collect();
    let mean = closes.iter().sum::<f64>() / closes.len() as f64;
    let variance = closes.iter().map(|&x| (x - mean).powi(2)).sum::<f64>() / closes.len() as f64;
    
    encoding.push(mean);
    encoding.push(variance.sqrt()); // Standard deviation
    encoding.push(closes.first().unwrap() - closes.last().unwrap()); // Total change
    
    // Price momentum features
    if closes.len() > 1 {
        let momentum = closes.windows(2)
            .map(|w| w[1] - w[0])
            .collect::<Vec<f64>>();
        
        let momentum_mean = momentum.iter().sum::<f64>() / momentum.len() as f64;
        encoding.push(momentum_mean);
        
        // Volatility
        let volatility = prices.iter()
            .map(|p| (p.high - p.low) / p.close)
            .sum::<f64>() / prices.len() as f64;
        encoding.push(volatility);
    } else {
        encoding.push(0.0);
        encoding.push(0.0);
    }
    
    // Pad to fixed size
    while encoding.len() < 10 {
        encoding.push(0.0);
    }
    
    Ok(encoding)
}

/// Compute coherence score between temporal components
fn compute_coherence_score(past: &[f64], present: &[f64], future: &[f64]) -> f64 {
    let past_present_sim = compute_vector_similarity(past, present);
    let present_future_sim = compute_vector_similarity(present, future);
    let past_future_sim = compute_vector_similarity(past, future);
    
    (past_present_sim + present_future_sim + past_future_sim) / 3.0
}

/// Compute similarity between two vectors
fn compute_vector_similarity(v1: &[f64], v2: &[f64]) -> f64 {
    if v1.is_empty() || v2.is_empty() {
        return 0.0;
    }
    
    let min_len = v1.len().min(v2.len());
    let mut dot_product = 0.0;
    let mut norm1 = 0.0;
    let mut norm2 = 0.0;
    
    for i in 0..min_len {
        dot_product += v1[i] * v2[i];
        norm1 += v1[i] * v1[i];
        norm2 += v2[i] * v2[i];
    }
    
    if norm1 == 0.0 || norm2 == 0.0 {
        return 0.0;
    }
    
    dot_product / (norm1.sqrt() * norm2.sqrt())
}
