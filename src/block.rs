use ggez::graphics::{self, Color, DrawParam};
use ggez::{Context, GameResult};
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
    
    pub fn draw(&self, ctx: &mut Context, canvas: &mut graphics::Canvas, cell_size: f32) -> GameResult {
        let (x, y) = self.position;
        let block_mesh = graphics::Mesh::new_rectangle(
            ctx,
            graphics::DrawMode::fill(),
            graphics::Rect::new(
                x as f32 * cell_size,
                y as f32 * cell_size,
                cell_size,
                cell_size,
            ),
            Color::BLACK,
        )?;
        canvas.draw(&block_mesh, DrawParam::default());
        Ok(())
    }
}

pub fn spawn_random_block(grid_size: usize) -> Block {
    let mut rng = rand::rng();
    let x = rng.random_range(0..grid_size);
    
    Block::new((x, 0))
}
