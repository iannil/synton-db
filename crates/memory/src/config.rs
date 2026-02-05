// Copyright 2025 SYNTON-DB Team
//
// Licensed under the Apache License, Version 2.0 (the "License");

use crate::error::MemoryResult;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Configuration for memory decay calculations.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DecayConfig {
    /// Decay rate lambda (per hour).
    /// Default: 0.0015 (approx 20% retention after 24h)
    pub lambda: f32,

    /// Minimum access score (nodes below this may be pruned).
    pub min_score: f32,

    /// Maximum access score.
    pub max_score: f32,

    /// Score boost on access (adds this to current score).
    pub access_boost: f32,

    /// Whether to clamp scores to [min_score, max_score].
    pub clamp_scores: bool,
}

impl Default for DecayConfig {
    fn default() -> Self {
        Self {
            lambda: 0.0015,
            min_score: 0.1,
            max_score: 10.0,
            access_boost: 1.0,
            clamp_scores: true,
        }
    }
}

impl DecayConfig {
    /// Create a new config with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the decay rate lambda.
    pub fn with_lambda(mut self, lambda: f32) -> MemoryResult<Self> {
        if !(0.0..=1.0).contains(&lambda) {
            return Err(crate::MemoryError::InvalidDecayRate(lambda));
        }
        self.lambda = lambda;
        Ok(self)
    }

    /// Set the minimum score.
    pub fn with_min_score(mut self, min: f32) -> Self {
        self.min_score = min;
        self
    }

    /// Set the maximum score.
    pub fn with_max_score(mut self, max: f32) -> Self {
        self.max_score = max;
        self
    }

    /// Set the access boost.
    pub fn with_access_boost(mut self, boost: f32) -> Self {
        self.access_boost = boost;
        self
    }

    /// Validate the configuration.
    pub fn validate(&self) -> MemoryResult<()> {
        if !(0.0..=1.0).contains(&self.lambda) {
            return Err(crate::MemoryError::InvalidDecayRate(self.lambda));
        }
        if self.min_score < 0.0 {
            return Err(crate::MemoryError::InvalidAccessScore(self.min_score));
        }
        if self.max_score <= self.min_score {
            return Err(crate::MemoryError::Custom(
                "max_score must be greater than min_score".to_string(),
            ));
        }
        Ok(())
    }

    /// Calculate the decay factor for a given duration.
    pub fn decay_factor(&self, duration: Duration) -> f64 {
        let hours = duration.as_secs_f64() / 3600.0;
        (-self.lambda as f64 * hours).exp()
    }

    /// Calculate the retention rate after a given time period.
    pub fn retention_after(&self, duration: Duration) -> f64 {
        self.decay_factor(duration)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = DecayConfig::default();
        assert_eq!(config.lambda, 0.0015);
        assert_eq!(config.min_score, 0.1);
        assert_eq!(config.max_score, 10.0);
    }

    #[test]
    fn test_config_builder() {
        let config = DecayConfig::new()
            .with_lambda(0.002)
            .unwrap()
            .with_access_boost(2.0);

        assert_eq!(config.lambda, 0.002);
        assert_eq!(config.access_boost, 2.0);
    }

    #[test]
    fn test_config_validation() {
        let config = DecayConfig::default();
        assert!(config.validate().is_ok());

        let bad_config = DecayConfig::default().with_lambda(2.0);
        assert!(bad_config.is_err());
    }

    #[test]
    fn test_decay_factor() {
        let config = DecayConfig::new().with_lambda(0.001).unwrap();

        // After 1 hour with lambda=0.001: e^(-0.001) ≈ 0.999
        let one_hour = Duration::from_secs(3600);
        let factor = config.decay_factor(one_hour);
        assert!((factor - 0.999).abs() < 0.001);

        // After 100 hours: e^(-0.1) ≈ 0.905
        let hundred_hours = Duration::from_secs(3600 * 100);
        let factor2 = config.decay_factor(hundred_hours);
        assert!((factor2 - 0.905).abs() < 0.001);
    }
}
