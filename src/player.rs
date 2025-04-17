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
    
    pub fn move_left(&mut self, blocks: &mut [Block]) {
        if self.position.0 > 0 {
            // Check if there's a block at the position we want to move to
            let target_pos = (self.position.0 - 1, self.position.1);
            let target_pos_body = (self.position.0 - 1, self.position.1 + 1); // Position for player's body
            
            // Find if there's a block at either the head or body position
            let block_idx = blocks.iter().position(|block| 
                block.position == target_pos || block.position == target_pos_body
            );
            
            if let Some(idx) = block_idx {
                // There's a block, check if we can push it
                if blocks[idx].position.0 > 0 {
                    // Check if there's another block or wall preventing movement
                    let block_target = (blocks[idx].position.0 - 1, blocks[idx].position.1);
                    let block_blocked = blocks.iter().any(|b| b.position == block_target);
                    
                    if !block_blocked {
                        // Move the block
                        blocks[idx].position.0 -= 1;
                        // Then move the player
                        self.position.0 -= 1;
                    }
                }
            } else {
                // No block, move freely
                self.position.0 -= 1;
            }
        }
    }
    
    pub fn move_right(&mut self, grid_size: usize, blocks: &mut [Block]) {
        if self.position.0 < grid_size - 1 {
            // Check if there's a block at the position we want to move to
            let target_pos = (self.position.0 + 1, self.position.1);
            let target_pos_body = (self.position.0 + 1, self.position.1 + 1); // Position for player's body
            
            // Find if there's a block at either the head or body position
            let block_idx = blocks.iter().position(|block| 
                block.position == target_pos || block.position == target_pos_body
            );
            
            if let Some(idx) = block_idx {
                // There's a block, check if we can push it
                if blocks[idx].position.0 < grid_size - 1 {
                    // Check if there's another block or wall preventing movement
                    let block_target = (blocks[idx].position.0 + 1, blocks[idx].position.1);
                    let block_blocked = blocks.iter().any(|b| b.position == block_target);
                    
                    if !block_blocked {
                        // Move the block
                        blocks[idx].position.0 += 1;
                        // Then move the player
                        self.position.0 += 1;
                    }
                }
            } else {
                // No block, move freely
                self.position.0 += 1;
            }
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
