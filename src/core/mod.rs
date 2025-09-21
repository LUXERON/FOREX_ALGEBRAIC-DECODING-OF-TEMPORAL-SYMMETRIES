//! # Core Time-Symmetric Engine
//! 
//! Mathematical foundation for time-symmetric pattern recognition using
//! Galois field cyclicity and algebraic structures.

pub mod engine;
pub mod temporal_state;
pub mod field_operations;

pub use engine::{TimeSymmetricEngine, EngineConfig};
pub use temporal_state::{TemporalState, TemporalStateSpace};
pub use field_operations::{FieldOperations, GaloisFieldProcessor};
