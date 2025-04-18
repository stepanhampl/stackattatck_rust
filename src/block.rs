use rand::Rng;

pub struct Block {
    pub position: (usize, usize),
    pub falling: bool,
    pub carried: bool, // Track if block is being carried
    pub carrying_direction: Option<isize>, // Track direction of carrying (positive = right, negative = left)
}

impl Block {
    pub fn new(position: (usize, usize)) -> Self {
        Self {
            position,
            falling: true,
            carried: false,
            carrying_direction: None,
        }
    }
}

pub fn spawn_random_block(grid_size: usize) -> Block {
    let mut rng = rand::rng();
    let x = rng.random_range(0..grid_size);
    
    Block::new((x, 0))
}
