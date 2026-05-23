use crate::input::InputEvent;
use crate::physics::{PhysicsState, PhysicsConfig, Vector2D};
use crossbeam_channel::Receiver;
use std::time::{Duration, Instant};

pub struct App {
    physics: PhysicsState,
    event_rx: Receiver<InputEvent>,
    last_update: Instant,
}

impl App {
    pub fn new(event_rx: Receiver<InputEvent>) -> Self {
        Self {
            physics: PhysicsState::new(PhysicsConfig::default()),
            event_rx,
            last_update: Instant::now(),
        }
    }

    pub fn run(&mut self) {
        let frame_duration = Duration::from_millis(16); // ~60 FPS

        loop {
            let now = Instant::now();
            let dt = now.duration_since(self.last_update).as_secs_f32();
            self.last_update = now;

            self.process_events();
            self.update(dt);
            self.render();

            let elapsed = now.elapsed();
            if elapsed < frame_duration {
                std::thread::sleep(frame_duration - elapsed);
            }
        }
    }

    fn process_events(&mut self) {
        while let Ok(event) = self.event_rx.try_recv() {
            match event {
                InputEvent::HotkeyTriggered(_) => {
                    println!("Hotkey triggered!");
                }
                InputEvent::KeyDown(vk) => {
                    self.handle_key_down(vk);
                }
                InputEvent::KeyUp(vk) => {
                    self.handle_key_up(vk);
                }
                InputEvent::MouseMove { dx, dy } => {
                    println!("Mouse moved: {}, {}", dx, dy);
                }
            }
        }
    }

    fn handle_key_down(&mut self, vk: u32) {
        // Arrow keys: 0x25 (Left), 0x26 (Up), 0x27 (Right), 0x28 (Down)
        let thrust = match vk {
            0x25 => Vector2D { x: -1.0, y: 0.0 },
            0x26 => Vector2D { x: 0.0, y: -1.0 },
            0x27 => Vector2D { x: 1.0, y: 0.0 },
            0x28 => Vector2D { x: 0.0, y: 1.0 },
            _ => return,
        };
        self.physics.apply_thrust(thrust, 0.016); // dt placeholder
    }

    fn handle_key_up(&mut self, _vk: u32) {
        // Friction handles deceleration
    }

    fn update(&mut self, dt: f32) {
        self.physics.update(dt);
    }

    fn render(&self) {
        if self.physics.velocity.x.abs() > 0.0 || self.physics.velocity.y.abs() > 0.0 {
            println!("Velocity: {:?}", self.physics.velocity);
        }
    }
}
