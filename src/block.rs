use rand::Rng;

pub struct Block {
    pub position: (usize, usize),
    pub falling: bool,
}

impl Block {
    pub fn new(position: (usize, usize)) -> Self {
        Self {
            position,
            falling: true,
        }
    }
}

pub fn spawn_random_block(grid_size: usize) -> Block {
    let mut rng = rand::rng();
    let x = rng.random_range(0..grid_size);
    
    Block::new((x, 0))
}
