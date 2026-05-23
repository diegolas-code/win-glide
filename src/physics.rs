//! Momentum-based physics model.
//! 
//! This module implements the "Snappy & Light" movement model.
//! It uses a velocity-based approach with acceleration and dual friction:
//! - Low friction while keys are held (allows reaching high speeds).
//! - High friction when keys are released (results in a quick, satisfying stop).

use serde::{Deserialize, Serialize};

/// A simple 2D vector for position and velocity.
#[derive(Debug, Default, Clone, Copy)]
pub struct Vector2D {
    pub x: f32,
    pub y: f32,
}

/// The runtime state of the physics engine for a single window.
pub struct PhysicsState {
    pub velocity: Vector2D,
    pub config: PhysicsConfig,
}

/// Configuration parameters for the physics simulation.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct PhysicsConfig {
    /// Rate at which velocity increases (pixels/s^2).
    pub acceleration: f32,
    /// Friction factor applied when coasting (keys released).
    pub friction: f32,
    /// Friction factor applied while thrusting (allows higher top speed).
    pub thrust_friction: f32,
    /// Maximum speed allowed (pixels/s).
    pub top_speed: f32,
}

impl Default for PhysicsConfig {
    fn default() -> Self {
        Self {
            acceleration: 4000.0,  // High acceleration for "snappy" feel.
            friction: 10.0,        // High friction for quick stops.
            thrust_friction: 0.5,  // Low friction while active to maintain momentum.
            top_speed: 3000.0,     // Fast enough to cross 1080p in ~0.6s at max speed.
        }
    }
}

impl PhysicsState {
    pub fn new(config: PhysicsConfig) -> Self {
        Self {
            velocity: Vector2D::default(),
            config,
        }
    }

    /// Applies acceleration in the specified direction.
    /// 
    /// `thrust` should be a unit vector (or zero).
    /// `dt` is the time elapsed since the last update in seconds.
    pub fn apply_thrust(&mut self, thrust: Vector2D, dt: f32) {
        // Increase velocity based on acceleration and time delta.
        self.velocity.x += thrust.x * self.config.acceleration * dt;
        self.velocity.y += thrust.y * self.config.acceleration * dt;

        // Limit to top speed using vector magnitude to ensure 
        // diagonal movement isn't faster than cardinal movement.
        let speed = (self.velocity.x.powi(2) + self.velocity.y.powi(2)).sqrt();
        if speed > self.config.top_speed {
            let factor = self.config.top_speed / speed;
            self.velocity.x *= factor;
            self.velocity.y *= factor;
        }
    }

    /// Updates the velocity based on friction and time delta.
    /// 
    /// Uses exponential decay for friction: v = v0 * e^(-f * dt).
    /// This provides a smooth, natural-feeling deceleration.
    pub fn update(&mut self, dt: f32, is_thrusting: bool) {
        // Select friction coefficient based on whether thrust is being applied.
        let f = if is_thrusting {
            self.config.thrust_friction
        } else {
            self.config.friction
        };
        
        let friction_factor = (-f * dt).exp();
        self.velocity.x *= friction_factor;
        self.velocity.y *= friction_factor;

        // Threshold to zero to avoid infinitesimal floating point values
        // and unnecessary updates when effectively stationary.
        if self.velocity.x.abs() < 0.1 { self.velocity.x = 0.0; }
        if self.velocity.y.abs() < 0.1 { self.velocity.y = 0.0; }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_physics_acceleration() {
        let config = PhysicsConfig::default();
        let mut state = PhysicsState::new(config);
        
        state.apply_thrust(Vector2D { x: 1.0, y: 0.0 }, 0.1);
        assert!(state.velocity.x > 0.0);
    }

    #[test]
    fn test_physics_friction() {
        let config = PhysicsConfig::default();
        let mut state = PhysicsState::new(config);
        state.velocity.x = 100.0;
        
        state.update(0.1, false);
        assert!(state.velocity.x < 100.0);
    }
}
