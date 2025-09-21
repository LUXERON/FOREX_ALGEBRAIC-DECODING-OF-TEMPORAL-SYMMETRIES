//! # Laplacian Reinforcement Learning for Anomaly-Based Trading
//! 
//! De Bruijn graph-based Q-learning with PME and Laplacian attention for trading decisions

use anyhow::Result;
use std::collections::{HashMap, VecDeque};
use nalgebra::{DVector, DMatrix};
use serde::{Deserialize, Serialize};

use crate::anomaly::{DetectedAnomaly, AnomalyType, AnomalySeverity};
use crate::data::ForexDataPoint;

/// De Bruijn graph-based Q-learning agent for anomaly trading
pub struct LaplacianQLearningAgent {
    /// De Bruijn graph structure for state representation
    debruijn_graph: DeBruijnGraph,
    
    /// Q-value table using PME approximation
    q_table: HashMap<StateActionPair, f64>,
    
    /// Laplacian matrix for attention mechanism
    laplacian_matrix: DMatrix<f64>,
    
    /// Agent configuration
    config: LaplacianQLearningConfig,
    
    /// Experience replay buffer
    experience_buffer: VecDeque<Experience>,
    
    /// Performance metrics
    performance_metrics: PerformanceMetrics,
}

/// Configuration for Laplacian Q-learning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LaplacianQLearningConfig {
    /// Learning rate (alpha)
    pub learning_rate: f64,
    
    /// Discount factor (gamma)
    pub discount_factor: f64,
    
    /// Exploration rate (epsilon)
    pub exploration_rate: f64,
    
    /// Epsilon decay rate
    pub epsilon_decay: f64,
    
    /// Minimum epsilon
    pub min_epsilon: f64,
    
    /// Experience buffer size
    pub buffer_size: usize,
    
    /// Batch size for learning
    pub batch_size: usize,
    
    /// PME grid size for continuous approximation
    pub pme_grid_size: usize,
    
    /// Laplacian attention weight
    pub attention_weight: f64,
}

/// De Bruijn graph for efficient state representation
#[derive(Debug, Clone)]
pub struct DeBruijnGraph {
    /// Nodes represent discretized market states
    nodes: HashMap<String, GraphNode>,
    
    /// Edges represent state transitions
    edges: HashMap<String, Vec<GraphEdge>>,
    
    /// Graph parameters
    alphabet_size: usize,
    sequence_length: usize,
}

/// Graph node representing a market state
#[derive(Debug, Clone)]
pub struct GraphNode {
    pub id: String,
    pub state_vector: DVector<f64>,
    pub anomaly_features: AnomalyFeatures,
    pub visit_count: u32,
    pub value_estimate: f64,
}

/// Graph edge representing state transition
#[derive(Debug, Clone, Serialize)]
pub struct GraphEdge {
    pub from_node: String,
    pub to_node: String,
    pub action: TradingAction,
    pub transition_probability: f64,
    pub reward_estimate: f64,
}

/// Anomaly features for state representation
#[derive(Debug, Clone)]
pub struct AnomalyFeatures {
    pub symmetry_deviation: f64,
    pub cycle_disruption: f64,
    pub volatility_spike: f64,
    pub pattern_inversion: f64,
    pub novel_pattern_strength: f64,
    pub anomaly_confidence: f64,
    pub market_context_vector: DVector<f64>,
}

/// Trading actions
#[derive(Debug, Clone, Serialize, PartialEq)]
pub enum TradingAction {
    Buy { size: u32 }, // Use integer for size to enable Hash/Eq
    Sell { size: u32 },
    Hold,
    ClosePosition,
}

impl std::hash::Hash for TradingAction {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            TradingAction::Buy { size } => {
                0u8.hash(state);
                size.hash(state);
            }
            TradingAction::Sell { size } => {
                1u8.hash(state);
                size.hash(state);
            }
            TradingAction::Hold => 2u8.hash(state),
            TradingAction::ClosePosition => 3u8.hash(state),
        }
    }
}

impl Eq for TradingAction {}

/// State-action pair for Q-table
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct StateActionPair {
    pub state_id: String,
    pub action: TradingAction,
}

/// Experience for replay buffer
#[derive(Debug, Clone)]
pub struct Experience {
    pub state: String,
    pub action: TradingAction,
    pub reward: f64,
    pub next_state: String,
    pub done: bool,
    pub anomaly_context: Option<DetectedAnomaly>,
}

/// Performance metrics
#[derive(Debug, Clone, Serialize)]
pub struct PerformanceMetrics {
    pub total_episodes: u32,
    pub total_reward: f64,
    pub average_reward: f64,
    pub exploration_rate: f64,
    pub q_value_convergence: f64,
    pub anomaly_detection_accuracy: f64,
    pub trading_success_rate: f64,
}

impl Default for LaplacianQLearningConfig {
    fn default() -> Self {
        Self {
            learning_rate: 0.1,
            discount_factor: 0.95,
            exploration_rate: 0.1,
            epsilon_decay: 0.995,
            min_epsilon: 0.01,
            buffer_size: 10000,
            batch_size: 32,
            pme_grid_size: 64,
            attention_weight: 0.3,
        }
    }
}

impl LaplacianQLearningAgent {
    /// Create new Laplacian Q-learning agent
    pub fn new(config: LaplacianQLearningConfig) -> Result<Self> {
        let debruijn_graph = DeBruijnGraph::new(4, 3)?; // 4-symbol alphabet, length 3
        let laplacian_matrix = Self::compute_graph_laplacian(&debruijn_graph)?;
        
        Ok(Self {
            debruijn_graph,
            q_table: HashMap::new(),
            laplacian_matrix,
            config: config.clone(),
            experience_buffer: VecDeque::with_capacity(config.buffer_size),
            performance_metrics: PerformanceMetrics::default(),
        })
    }
    
    /// Compute graph Laplacian for attention mechanism
    fn compute_graph_laplacian(graph: &DeBruijnGraph) -> Result<DMatrix<f64>> {
        let n = graph.nodes.len();
        let mut adjacency = DMatrix::zeros(n, n);
        let mut degree = DVector::zeros(n);
        
        // Build adjacency matrix
        let node_indices: HashMap<String, usize> = graph.nodes.keys()
            .enumerate()
            .map(|(i, k)| (k.clone(), i))
            .collect();
        
        for (from_node, edges) in &graph.edges {
            if let Some(&from_idx) = node_indices.get(from_node) {
                for edge in edges {
                    if let Some(&to_idx) = node_indices.get(&edge.to_node) {
                        adjacency[(from_idx, to_idx)] = edge.transition_probability;
                        degree[from_idx] += edge.transition_probability;
                    }
                }
            }
        }
        
        // Compute Laplacian: L = D - A
        let mut laplacian = DMatrix::zeros(n, n);
        for i in 0..n {
            laplacian[(i, i)] = degree[i];
            for j in 0..n {
                if i != j {
                    laplacian[(i, j)] = -adjacency[(i, j)];
                }
            }
        }
        
        Ok(laplacian)
    }
    
    /// Convert anomaly to state representation
    pub fn anomaly_to_state(&self, anomaly: &DetectedAnomaly, market_data: &ForexDataPoint) -> Result<String> {
        let anomaly_features = AnomalyFeatures {
            symmetry_deviation: match &anomaly.anomaly_type {
                AnomalyType::SymmetryBreakdown { expected_strength, actual_strength, .. } => {
                    (expected_strength - actual_strength).abs()
                }
                _ => 0.0,
            },
            cycle_disruption: match &anomaly.anomaly_type {
                AnomalyType::CycleDisruption { expected_phase, actual_phase, .. } => {
                    (expected_phase - actual_phase).abs()
                }
                _ => 0.0,
            },
            volatility_spike: match &anomaly.anomaly_type {
                AnomalyType::VolatilitySpike { expected_volatility, actual_volatility } => {
                    actual_volatility / expected_volatility
                }
                _ => 1.0,
            },
            pattern_inversion: match &anomaly.anomaly_type {
                AnomalyType::PatternInversion { .. } => 1.0,
                _ => 0.0,
            },
            novel_pattern_strength: match &anomaly.anomaly_type {
                AnomalyType::NovelPattern { emergence_confidence, .. } => *emergence_confidence,
                _ => 0.0,
            },
            anomaly_confidence: anomaly.confidence,
            market_context_vector: DVector::from_vec(vec![
                market_data.close,
                market_data.high - market_data.low, // Range
                (market_data.close - market_data.open) / market_data.open, // Return
            ]),
        };
        
        // Discretize features to create state ID
        let state_id = format!(
            "s_{:.2}_{:.2}_{:.2}_{:.2}_{:.2}_{:.2}",
            (anomaly_features.symmetry_deviation * 100.0).round() / 100.0,
            (anomaly_features.cycle_disruption * 100.0).round() / 100.0,
            (anomaly_features.volatility_spike * 100.0).round() / 100.0,
            (anomaly_features.pattern_inversion * 100.0).round() / 100.0,
            (anomaly_features.novel_pattern_strength * 100.0).round() / 100.0,
            (anomaly_features.anomaly_confidence * 100.0).round() / 100.0,
        );
        
        // Add node to graph if not exists
        if !self.debruijn_graph.nodes.contains_key(&state_id) {
            // This would require mutable access - in practice, we'd pre-build the graph
            // or use a different approach for dynamic state space
        }
        
        Ok(state_id)
    }
    
    /// Choose action using epsilon-greedy with Laplacian attention
    pub fn choose_action(&self, state_id: &str, anomaly: &DetectedAnomaly) -> Result<TradingAction> {
        // Epsilon-greedy exploration
        if rand::random::<f64>() < self.config.exploration_rate {
            return Ok(self.random_action(anomaly));
        }
        
        // Get all possible actions for this state
        let possible_actions = self.get_possible_actions(state_id, anomaly);
        
        // Apply Laplacian attention to weight Q-values
        let mut best_action = possible_actions[0].clone();
        let mut best_q_value = f64::NEG_INFINITY;
        
        for action in possible_actions {
            let state_action = StateActionPair {
                state_id: state_id.to_string(),
                action: action.clone(),
            };
            
            let base_q_value = self.q_table.get(&state_action).unwrap_or(&0.0);
            let attention_weight = self.compute_laplacian_attention(state_id)?;
            let weighted_q_value = base_q_value * (1.0 + self.config.attention_weight * attention_weight);
            
            if weighted_q_value > best_q_value {
                best_q_value = weighted_q_value;
                best_action = action;
            }
        }
        
        Ok(best_action)
    }
    
    /// Compute Laplacian attention weight for state
    fn compute_laplacian_attention(&self, state_id: &str) -> Result<f64> {
        // Find state index in graph
        let state_index = self.debruijn_graph.nodes.keys()
            .position(|k| k == state_id)
            .unwrap_or(0);
        
        // Use Laplacian eigenvector for attention (simplified)
        let attention_weight = if state_index < self.laplacian_matrix.nrows() {
            self.laplacian_matrix[(state_index, state_index)].abs()
        } else {
            1.0
        };
        
        Ok(attention_weight)
    }
    
    /// Get possible actions for state and anomaly
    fn get_possible_actions(&self, state_id: &str, anomaly: &DetectedAnomaly) -> Vec<TradingAction> {
        let mut actions = vec![TradingAction::Hold];
        
        // Generate actions based on anomaly type and severity
        match &anomaly.anomaly_type {
            AnomalyType::SymmetryBreakdown { expected_strength, actual_strength, .. } => {
                if actual_strength < expected_strength {
                    actions.push(TradingAction::Sell { size: 10 }); // 10% position
                    if matches!(anomaly.severity, AnomalySeverity::High | AnomalySeverity::Critical) {
                        actions.push(TradingAction::Sell { size: 20 }); // 20% position
                    }
                } else {
                    actions.push(TradingAction::Buy { size: 10 });
                    if matches!(anomaly.severity, AnomalySeverity::High | AnomalySeverity::Critical) {
                        actions.push(TradingAction::Buy { size: 20 });
                    }
                }
            }
            AnomalyType::VolatilitySpike { .. } => {
                // High volatility suggests caution
                actions.push(TradingAction::ClosePosition);
            }
            AnomalyType::PatternInversion { .. } => {
                // Pattern inversion suggests reversal
                actions.push(TradingAction::Buy { size: 15 });
                actions.push(TradingAction::Sell { size: 15 });
            }
            _ => {
                // Default actions for other anomaly types
                actions.push(TradingAction::Buy { size: 10 });
                actions.push(TradingAction::Sell { size: 10 });
            }
        }
        
        actions
    }
    
    /// Generate random action
    fn random_action(&self, anomaly: &DetectedAnomaly) -> TradingAction {
        let actions = self.get_possible_actions("", anomaly);
        let index = (rand::random::<f64>() * actions.len() as f64) as usize;
        actions[index].clone()
    }
    
    /// Update Q-value using PME approximation and Laplacian attention
    pub fn update_q_value(
        &mut self,
        state: &str,
        action: TradingAction,
        reward: f64,
        next_state: &str,
        done: bool,
    ) -> Result<()> {
        let state_action = StateActionPair {
            state_id: state.to_string(),
            action: action.clone(),
        };
        
        // Get current Q-value
        let current_q = self.q_table.get(&state_action).unwrap_or(&0.0);
        
        // Calculate target Q-value
        let next_q_max = if done {
            0.0
        } else {
            self.get_max_q_value(next_state)
        };
        
        // Apply PME approximation (simplified)
        let pme_correction = self.compute_pme_correction(state, &action)?;
        
        // Apply Laplacian attention
        let attention_weight = self.compute_laplacian_attention(state)?;
        
        // Bellman equation with PME and attention
        let target_q = reward + self.config.discount_factor * next_q_max + pme_correction;
        let attention_factor = 1.0 + self.config.attention_weight * attention_weight;
        let new_q = current_q + self.config.learning_rate * attention_factor * (target_q - current_q);
        
        // Update Q-table
        self.q_table.insert(state_action, new_q);
        
        Ok(())
    }
    
    /// Compute PME correction for continuous state approximation
    fn compute_pme_correction(&self, state: &str, action: &TradingAction) -> Result<f64> {
        // Simplified PME implementation
        // In practice, this would involve real-space and reciprocal-space calculations
        
        let real_space_contribution = match action {
            TradingAction::Buy { size } => (*size as f64) * 0.01,
            TradingAction::Sell { size } => -(*size as f64) * 0.01,
            _ => 0.0,
        };
        
        let reciprocal_space_contribution = real_space_contribution * 0.1; // Simplified
        
        Ok(real_space_contribution + reciprocal_space_contribution)
    }
    
    /// Get maximum Q-value for state
    fn get_max_q_value(&self, state: &str) -> f64 {
        self.q_table.iter()
            .filter(|(sa, _)| sa.state_id == state)
            .map(|(_, &q)| q)
            .fold(f64::NEG_INFINITY, f64::max)
            .max(0.0)
    }
    
    /// Add experience to replay buffer
    pub fn add_experience(&mut self, experience: Experience) {
        if self.experience_buffer.len() >= self.config.buffer_size {
            self.experience_buffer.pop_front();
        }
        self.experience_buffer.push_back(experience);
    }
    
    /// Train on batch of experiences
    pub fn train_batch(&mut self) -> Result<()> {
        if self.experience_buffer.len() < self.config.batch_size {
            return Ok(());
        }
        
        // Sample random batch
        let batch_size = self.config.batch_size.min(self.experience_buffer.len());
        let mut batch_indices = Vec::new();
        for _ in 0..batch_size {
            let idx = (rand::random::<f64>() * self.experience_buffer.len() as f64) as usize;
            batch_indices.push(idx);
        }
        
        // Train on batch - collect experiences first to avoid borrow issues
        let experiences: Vec<_> = batch_indices.iter()
            .filter_map(|&idx| self.experience_buffer.get(idx).cloned())
            .collect();

        for experience in experiences {
            self.update_q_value(
                &experience.state,
                experience.action,
                experience.reward,
                &experience.next_state,
                experience.done,
            )?;
        }
        
        // Decay exploration rate
        self.config.exploration_rate = (self.config.exploration_rate * self.config.epsilon_decay)
            .max(self.config.min_epsilon);
        
        Ok(())
    }
    
    /// Get performance metrics
    pub fn get_performance_metrics(&self) -> &PerformanceMetrics {
        &self.performance_metrics
    }
    
    /// Update performance metrics
    pub fn update_performance_metrics(&mut self, episode_reward: f64, anomaly_accuracy: f64, trading_success: bool) {
        self.performance_metrics.total_episodes += 1;
        self.performance_metrics.total_reward += episode_reward;
        self.performance_metrics.average_reward = 
            self.performance_metrics.total_reward / self.performance_metrics.total_episodes as f64;
        self.performance_metrics.exploration_rate = self.config.exploration_rate;
        self.performance_metrics.anomaly_detection_accuracy = anomaly_accuracy;
        
        if trading_success {
            self.performance_metrics.trading_success_rate = 
                (self.performance_metrics.trading_success_rate * (self.performance_metrics.total_episodes - 1) as f64 + 1.0)
                / self.performance_metrics.total_episodes as f64;
        } else {
            self.performance_metrics.trading_success_rate = 
                (self.performance_metrics.trading_success_rate * (self.performance_metrics.total_episodes - 1) as f64)
                / self.performance_metrics.total_episodes as f64;
        }
    }
}

impl DeBruijnGraph {
    /// Create new De Bruijn graph
    pub fn new(alphabet_size: usize, sequence_length: usize) -> Result<Self> {
        let mut nodes = HashMap::new();
        let mut edges = HashMap::new();
        
        // Generate all possible sequences of given length
        let total_sequences = alphabet_size.pow(sequence_length as u32);
        
        for i in 0..total_sequences {
            let sequence = Self::index_to_sequence(i, alphabet_size, sequence_length);
            let node_id = format!("node_{}", sequence);
            
            nodes.insert(node_id.clone(), GraphNode {
                id: node_id.clone(),
                state_vector: DVector::zeros(sequence_length),
                anomaly_features: AnomalyFeatures {
                    symmetry_deviation: 0.0,
                    cycle_disruption: 0.0,
                    volatility_spike: 1.0,
                    pattern_inversion: 0.0,
                    novel_pattern_strength: 0.0,
                    anomaly_confidence: 0.0,
                    market_context_vector: DVector::zeros(3),
                },
                visit_count: 0,
                value_estimate: 0.0,
            });
            
            // Create edges to successor nodes
            let mut node_edges = Vec::new();
            for symbol in 0..alphabet_size {
                let next_sequence = Self::get_successor_sequence(&sequence, symbol, alphabet_size);
                let next_node_id = format!("node_{}", next_sequence);
                
                node_edges.push(GraphEdge {
                    from_node: node_id.clone(),
                    to_node: next_node_id,
                    action: TradingAction::Hold, // Default action
                    transition_probability: 1.0 / alphabet_size as f64,
                    reward_estimate: 0.0,
                });
            }
            
            edges.insert(node_id, node_edges);
        }
        
        Ok(Self {
            nodes,
            edges,
            alphabet_size,
            sequence_length,
        })
    }
    
    /// Convert index to sequence
    fn index_to_sequence(mut index: usize, alphabet_size: usize, length: usize) -> String {
        let mut sequence = String::new();
        for _ in 0..length {
            sequence.push_str(&(index % alphabet_size).to_string());
            index /= alphabet_size;
        }
        sequence
    }
    
    /// Get successor sequence by shifting and adding new symbol
    fn get_successor_sequence(sequence: &str, new_symbol: usize, _alphabet_size: usize) -> String {
        let mut chars: Vec<char> = sequence.chars().collect();
        chars.remove(0); // Remove first character
        chars.push(char::from_digit(new_symbol as u32, 10).unwrap_or('0'));
        chars.into_iter().collect()
    }
}

impl Default for PerformanceMetrics {
    fn default() -> Self {
        Self {
            total_episodes: 0,
            total_reward: 0.0,
            average_reward: 0.0,
            exploration_rate: 0.1,
            q_value_convergence: 0.0,
            anomaly_detection_accuracy: 0.0,
            trading_success_rate: 0.0,
        }
    }
}
