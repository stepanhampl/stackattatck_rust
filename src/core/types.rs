// Core types used across the game
// These types are platform-independent

// Position in the game grid
pub type Position = (usize, usize);

// Direction of movement
pub type Direction = isize;

// The platform-independent InputAction enum
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InputAction {
    Left,
    Right,
    Up,
    Restart,
    None,
}

// Game state update result
pub enum GameUpdateResult {
    Continue,
    GameOver,
    Restart,
}

// Rendering color - platform-independent representation
#[derive(Clone, Copy)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    pub const BLACK: Color = Color { r: 0.0, g: 0.0, b: 0.0, a: 1.0 };
    pub const WHITE: Color = Color { r: 1.0, g: 1.0, b: 1.0, a: 1.0 };
    pub const RED: Color = Color { r: 1.0, g: 0.0, b: 0.0, a: 1.0 };
    pub const GREEN: Color = Color { r: 0.0, g: 1.0, b: 0.0, a: 1.0 };
    pub const BLUE: Color = Color { r: 0.0, g: 0.0, b: 1.0, a: 1.0 };
}

// Core game configuration
pub struct GameConfig {
    pub grid_size: usize,
    pub cell_size: f32,
    pub refresh_rate_milliseconds: u64,
    pub block_fall_speed: usize,
    pub block_spawn_rate: u64,
}
