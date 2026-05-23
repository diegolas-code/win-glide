use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Copy)]
pub struct Vector2D {
    pub x: f32,
    pub y: f32,
}

pub struct PhysicsState {
    pub velocity: Vector2D,
    pub config: PhysicsConfig,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct PhysicsConfig {
    pub acceleration: f32,
    pub friction: f32,        // Coasting friction (quick stop)
    pub thrust_friction: f32, // Friction while thrusting (allows reaching high speed)
    pub top_speed: f32,
}

impl Default for PhysicsConfig {
    fn default() -> Self {
        Self {
            acceleration: 3000.0,  // pixels per second^2 (slightly slower spin-up)
            friction: 10.0,        // velocity reduction factor when coasting
            thrust_friction: 0.5,  // low friction while keys are held
            top_speed: 4000.0,     // pixels per second
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

    pub fn apply_thrust(&mut self, thrust: Vector2D, dt: f32) {
        self.velocity.x += thrust.x * self.config.acceleration * dt;
        self.velocity.y += thrust.y * self.config.acceleration * dt;

        // Limit to top speed
        let speed = (self.velocity.x.powi(2) + self.velocity.y.powi(2)).sqrt();
        if speed > self.config.top_speed {
            let factor = self.config.top_speed / speed;
            self.velocity.x *= factor;
            self.velocity.y *= factor;
        }
    }

    pub fn update(&mut self, dt: f32, is_thrusting: bool) {
        // Apply friction
        let f = if is_thrusting {
            self.config.thrust_friction
        } else {
            self.config.friction
        };
        
        let friction_factor = (-f * dt).exp();
        self.velocity.x *= friction_factor;
        self.velocity.y *= friction_factor;

        // Stop if velocity is very low
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
