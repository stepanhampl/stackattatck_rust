use ggez::graphics::{self, Color, DrawParam};
use ggez::{Context, GameResult};

pub struct Player {
    pub position: (usize, usize),
}

impl Player {
    pub fn new(grid_size: usize) -> Self {
        Self {
            position: (0, grid_size - 2), // Start at bottom left
        }
    }
    
    pub fn move_left(&mut self) {
        if self.position.0 > 0 {
            self.position.0 -= 1;
        }
    }
    
    pub fn move_right(&mut self, grid_size: usize) {
        if self.position.0 < grid_size - 1 {
            self.position.0 += 1;
        }
    }
    
    pub fn draw(&self, ctx: &mut Context, canvas: &mut graphics::Canvas, cell_size: f32) -> GameResult {
        let (x, y) = self.position;
        let player_mesh = graphics::Mesh::new_rectangle(
            ctx,
            graphics::DrawMode::fill(),
            graphics::Rect::new(
                x as f32 * cell_size,
                y as f32 * cell_size,
                cell_size,
                cell_size * 2.0, // draws also the lower box - body
            ),
            Color::BLACK,
        )?;

        canvas.draw(&player_mesh, DrawParam::default());
        Ok(())
    }
}
