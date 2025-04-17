use ggez::graphics::{self, Color, DrawParam};
use ggez::{Context, GameResult};
use crate::block::Block;

pub struct Player {
    pub position: (usize, usize),
}

impl Player {
    pub fn new(grid_size: usize) -> Self {
        Self {
            position: (0, grid_size - 2), // Start at bottom left
        }
    }
    
    // Private helper method to handle horizontal movement
    fn move_horizontal(&mut self, move_by: isize, grid_size: usize, blocks: &mut [Block]) {
        let can_move = if move_by < 0 {
            self.position.0 > 0
        } else {
            self.position.0 < grid_size - 1
        };
        
        if can_move {
            let target_x = (self.position.0 as isize + move_by) as usize;
            let target_pos = (target_x, self.position.1);
            let target_pos_body = (target_x, self.position.1 + 1);
            
            let block_idx = blocks.iter().position(|block| 
                block.position == target_pos || block.position == target_pos_body
            );
            
            if let Some(idx) = block_idx {
                let block_can_move = if move_by < 0 {
                    blocks[idx].position.0 > 0
                } else {
                    blocks[idx].position.0 < grid_size - 1
                };
                
                if block_can_move {
                    let block_target_x = (blocks[idx].position.0 as isize + move_by) as usize;
                    let block_target = (block_target_x, blocks[idx].position.1);
                    let block_blocked = blocks.iter().any(|b| b.position == block_target);
                    
                    if !block_blocked {
                        // Move the block
                        blocks[idx].position.0 = block_target_x;
                        // Then move the player
                        self.position.0 = target_x;
                    }
                }
            } else {
                // No block, move freely
                self.position.0 = target_x;
            }
        }
    }
    
    pub fn move_left(&mut self, blocks: &mut [Block]) {
        // For left movement, we only need to check position > 0
        self.move_horizontal(-1, usize::MAX, blocks); // MAX value since we only need position > 0
    }
    
    pub fn move_right(&mut self, grid_size: usize, blocks: &mut [Block]) {
        self.move_horizontal(1, grid_size, blocks);
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
            Color::RED,
        )?;

        canvas.draw(&player_mesh, DrawParam::default());
        Ok(())
    }
}
