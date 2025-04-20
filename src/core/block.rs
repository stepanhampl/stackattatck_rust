// Core block implementation - platform-independent
use rand::Rng;
use crate::core::types::Position;
use crate::core::types::Direction;

pub struct Block {
    pub position: Position,
    pub falling: bool,
    pub carried: bool, // Track if block is being carried
    pub carrying_direction: Option<Direction>, // Track direction of carrying (positive = right, negative = left)
}

impl Block {
    pub fn new(position: Position) -> Self {
        Self {
            position,
            falling: true,
            carried: false,
            carrying_direction: None,
        }
    }
}

pub fn spawn_random_block(grid_size: usize) -> Block {
    let mut rng = rand::thread_rng();
    let x = rng.gen_range(0..grid_size);
    
    Block::new((x, 0))
}
