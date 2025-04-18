use ggez::graphics::{self, Color, DrawParam};
use ggez::{Context, GameResult};
use crate::block::Block;

pub struct Player {
    pub position: (usize, usize),
    pub in_air: bool,  // Track jump state
    pub is_falling: bool, // Track if player is falling due to gravity
    jump_counter: u8,  // Track how long to stay in the air
    just_jumped: bool, // Flag to prevent immediate landing
    pub body_size: usize, // Store the player's vertical size
}

impl Player {
    pub fn new(grid_size: usize) -> Self {
        let body_height = 2; // Store body size as a variable
        Self {
            position: (0, grid_size - body_height), // Start at bottom left
            in_air: false,
            is_falling: false,
            jump_counter: 0,
            just_jumped: false,
            body_size: body_height,
        }
    }
    
    // Add jump method
    pub fn jump(&mut self) {
        if !self.in_air && !self.is_falling && self.position.1 > 0 {
            self.position.1 -= 1;  // Move up one block
            self.in_air = true;
            self.jump_counter = 1;  // Stay in air for 1 update cycle
            self.just_jumped = true; // Set flag to prevent immediate landing
        }
    }
    
    // Method to update jump counter
    pub fn update_jump(&mut self) {
        if self.just_jumped {
            // Reset the just_jumped flag, but don't decrement counter yet
            self.just_jumped = false;
        } else if self.in_air && self.jump_counter > 0 {
            // Only decrement counter in subsequent updates
            self.jump_counter -= 1;
        }
    }
    
    // Check if there's ground or a block beneath the player
    pub fn has_support(&self, blocks: &[Block], grid_size: usize) -> bool {
        // Check if player is at the bottom of the grid
        if self.position.1 >= grid_size - self.body_size {
            return true;
        }
        
        // Check if there's a block directly beneath the player
        blocks.iter().any(|block| {
            !block.falling && 
            block.position.0 == self.position.0 && 
            block.position.1 == self.position.1 + self.body_size
        })
    }
    
    // Update player's falling state
    pub fn update_falling_state(&mut self, blocks: &[Block], grid_size: usize) {
        // Only check for falling if not already in a jumping action
        if !self.in_air && !self.just_jumped {
            // Check if there's no support beneath the player
            if !self.has_support(blocks, grid_size) {
                self.is_falling = true;
            }
        }
    }
    
    // Apply gravity to make player fall
    pub fn apply_gravity(&mut self) {
        if self.is_falling && self.position.1 < usize::MAX - 1 {
            self.position.1 += 1;  // Move down one block
        }
    }
    
    // Modify land method to check for blocks below
    pub fn land(&mut self, blocks: &[Block], grid_size: usize) {
        // Handle landing after a jump
        if self.in_air && self.jump_counter == 0 && !self.just_jumped {
            let has_support = self.has_support(blocks, grid_size);
            
            if !has_support {
                // If there's no support after jumping, start falling
                self.in_air = false;
                self.is_falling = true;
            } else {
                // Land properly after jumping
                self.in_air = false;
            }
        }
        
        // Handle landing after falling due to gravity
        if self.is_falling && self.has_support(blocks, grid_size) {
            self.is_falling = false;
        }
    }
    
    // Private helper method to handle horizontal movement - refactored for clarity
    fn move_horizontal(&mut self, move_by: isize, grid_size: usize, blocks: &mut [Block]) {
        // Check if movement is possible based on grid boundaries
        if !self.can_move_in_direction(move_by, grid_size) {
            return;
        }
        
        let target_x = (self.position.0 as isize + move_by) as usize;
        
        // Check for collision with any part of the player's body
        if let Some(block_idx) = self.find_blocking_block(target_x, blocks) {
            self.handle_block_collision(block_idx, move_by, target_x, grid_size, blocks);
        } else {
            // No block, move freely
            self.position.0 = target_x;
        }
    }
    
    // Check if movement in a direction is possible based on grid boundaries
    fn can_move_in_direction(&self, move_by: isize, grid_size: usize) -> bool {
        if move_by < 0 {
            self.position.0 > 0
        } else {
            self.position.0 < grid_size - 1
        }
    }
    
    // Find a block that is blocking the player's movement
    fn find_blocking_block(&self, target_x: usize, blocks: &[Block]) -> Option<usize> {
        for body_part in 0..self.body_size {
            let target_pos = (target_x, self.position.1 + body_part);
            if let Some(idx) = blocks.iter().position(|block| block.position == target_pos) {
                return Some(idx);
            }
        }
        None
    }
    
    // Handle collision with a block
    fn handle_block_collision(&mut self, block_idx: usize, move_by: isize, target_x: usize, 
                             grid_size: usize, blocks: &mut [Block]) {
        let block = &blocks[block_idx];
        let block_x = block.position.0;
        
        // Check if the block can move in this direction based on grid boundaries
        let block_can_move = if move_by < 0 {
            block_x > 0
        } else {
            block_x < grid_size - 1
        };
        
        if !block_can_move {
            return;
        }
        
        let block_target_x = (block_x as isize + move_by) as usize;
        
        if block.falling {
            self.handle_falling_block_movement(block_idx, block_target_x, target_x, blocks);
        } else {
            self.handle_normal_block_movement(block_x, block_target_x, target_x, blocks);
        }
    }
    
    // Handle movement of a falling block
    fn handle_falling_block_movement(&mut self, block_idx: usize, block_target_x: usize, 
                                    player_target_x: usize, blocks: &mut [Block]) {
        let target = (block_target_x, blocks[block_idx].position.1);
        
        // Check if the target position is occupied
        let is_blocked = blocks.iter().any(|b| b.position == target);
        
        if !is_blocked {
            // Move only this falling block
            blocks[block_idx].position.0 = block_target_x;
            // Then move the player
            self.position.0 = player_target_x;
        }
    }
    
    // Handle movement of normal (non-falling) blocks
    fn handle_normal_block_movement(&mut self, block_x: usize, 
                                   block_target_x: usize, player_target_x: usize, 
                                   blocks: &mut [Block]) {
        let pushable_indices = self.find_pushable_blocks(block_x, blocks);
        
        if pushable_indices.is_empty() {
            return;
        }
        
        // Check if any pushable block would be blocked in its new position
        if !self.is_path_clear_for_blocks(&pushable_indices, block_target_x, blocks) {
            return;
        }
        
        // Move all pushable blocks
        for &idx in &pushable_indices {
            blocks[idx].position.0 = block_target_x;
        }
        
        // Then move the player
        self.position.0 = player_target_x;
    }
    
    // Find which blocks are pushable in a column
    fn find_pushable_blocks(&self, block_x: usize, blocks: &[Block]) -> Vec<usize> {
        // Define the player's body range
        let player_top = self.position.1;
        let player_bottom = self.position.1 + self.body_size - 1;
        
        // Collect all non-falling blocks in this column
        let mut column_blocks: Vec<(usize, usize)> = blocks.iter()
            .enumerate()
            .filter_map(|(i, b)| {
                if b.position.0 == block_x && !b.falling {
                    Some((i, b.position.1))
                } else {
                    None
                }
            })
            .collect();
        
        // Sort by y-coordinate (top to bottom)
        column_blocks.sort_by_key(|&(_, y)| y);
        
        let mut pushable_indices = Vec::new();
        let mut pushable_y_coords = Vec::new();
        
        // First, mark blocks at player's body level as pushable
        for &(idx, y) in &column_blocks {
            if y >= player_top && y <= player_bottom {
                pushable_indices.push(idx);
                pushable_y_coords.push(y);
            }
        }
        
        // If we found some blocks at the player's level
        if !pushable_indices.is_empty() {
            // Now check all blocks ABOVE to see if they form a connected column
            let mut new_pushable_found = true;
            while new_pushable_found {
                new_pushable_found = false;
                
                for &(idx, y) in &column_blocks {
                    // Skip if already marked as pushable
                    if pushable_indices.contains(&idx) {
                        continue;
                    }
                    
                    // Only consider blocks ABOVE the player's level
                    if y > player_bottom {
                        continue;
                    }
                    
                    // Check if this block is connected to a pushable block directly below
                    if y > 0 && pushable_y_coords.contains(&(y + 1)) {
                        pushable_indices.push(idx);
                        pushable_y_coords.push(y);
                        new_pushable_found = true;
                    }
                }
            }
        }
        
        pushable_indices
    }
    
    // Check if the path is clear for all blocks to move
    fn is_path_clear_for_blocks(&self, pushable_indices: &[usize], target_x: usize, blocks: &[Block]) -> bool {
        for &idx in pushable_indices {
            let (_, y) = blocks[idx].position;
            let target = (target_x, y);
            
            // Check if target position is occupied by a block not in our pushable set
            for (i, b) in blocks.iter().enumerate() {
                if b.position == target && !pushable_indices.contains(&i) {
                    return false;
                }
            }
        }
        true
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
                cell_size * self.body_size as f32, // Use body_size variable instead of hardcoded 2.0
            ),
            Color::RED,
        )?;

        canvas.draw(&player_mesh, DrawParam::default());
        Ok(())
    }
}
