use crate::block::Block;

// Add a constant for fall delay duration
const FALL_DELAY: u8 = 3; // Number of update cycles to wait before falling

pub struct Player {
    pub position: (usize, usize),
    pub in_air: bool,  // Track jump state
    pub is_falling: bool, // Track if player is falling due to gravity
    jump_counter: u8,  // Track how long to stay in the air
    just_jumped: bool, // Flag to prevent immediate landing
    pub body_size: usize, // Store the player's vertical size
    fall_delay_counter: u8, // Counter for delaying fall
    grid_size: usize, // Store the grid size for consistent boundary checks
}

impl Player {
    pub fn new(grid_size: usize) -> Self {
        let body_height = 2; // Store body size as a variable
        
        // Calculate starting x position (middle of grid)
        // If even grid size, place a bit to the left of center
        let start_x = if grid_size % 2 == 0 {
            grid_size / 2 - 1 // Even grid size, place left of center
        } else {
            grid_size / 2     // Odd grid size, place at center
        };
        
        Self {
            position: (start_x, grid_size - body_height), // Start at bottom middle
            in_air: false,
            is_falling: false,
            jump_counter: 0,
            just_jumped: false,
            body_size: body_height,
            fall_delay_counter: 0,
            grid_size,
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
        // Don't check for falling during active jumps
        if self.in_air {
            return;
        }
        
        // Check if there's no support beneath the player
        if !self.has_support(blocks, grid_size) {
            // If we're not already falling and not already delaying a fall
            if !self.is_falling && self.fall_delay_counter == 0 {
                // Start the fall delay
                self.fall_delay_counter = FALL_DELAY;
            }
            // Note: We don't set is_falling=true here anymore, that happens in update_fall_delay
        } else {
            // We have support, so reset falling states
            self.is_falling = false;
            self.fall_delay_counter = 0;
        }
    }
    
    // Apply gravity to make player fall
    pub fn apply_gravity(&mut self) {
        if self.is_falling && self.position.1 < usize::MAX - 1 {
            self.position.1 += 1;  // Move down one block
        }
    }
    
    // Update fall delay counter
    pub fn update_fall_delay(&mut self) {
        if self.fall_delay_counter > 0 {
            self.fall_delay_counter -= 1;
            
            // When counter reaches zero, start falling if there's no support
            if self.fall_delay_counter == 0 && !self.in_air {
                self.is_falling = true;
            }
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
        // Don't allow movement if player is about to fall (fall delay is active)
        if self.fall_delay_counter > 0 {
            return;
        }

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
        
        // Check for support after moving horizontally
        self.check_support_after_move(grid_size, blocks);
    }
    
    // New method to check support after horizontal movement
    fn check_support_after_move(&mut self, grid_size: usize, blocks: &[Block]) {
        if !self.in_air && !self.is_falling && !self.has_support(blocks, grid_size) {
            // Start the fall delay instead of immediately falling
            self.fall_delay_counter = FALL_DELAY;
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
        
        // Check if the block can move in this direction
        if !self.can_block_move_in_direction(block.position.0, move_by, grid_size) {
            return;
        }
        
        let block_target_x = (block.position.0 as isize + move_by) as usize;
        
        if block.falling {
            self.handle_falling_block_movement(block_idx, block_target_x, target_x, blocks);
        } else {
            self.handle_normal_block_movement(block.position.0, block_target_x, target_x, blocks);
        }
    }
    
    // New method to check if a block can move in a direction
    fn can_block_move_in_direction(&self, block_x: usize, move_by: isize, grid_size: usize) -> bool {
        if move_by < 0 {
            block_x > 0
        } else {
            block_x < grid_size - 1
        }
    }
    
    // Handle movement of a falling block
    fn handle_falling_block_movement(&mut self, block_idx: usize, block_target_x: usize, 
                                    player_target_x: usize, blocks: &mut [Block]) {
        let target = (block_target_x, blocks[block_idx].position.1);
        
        // Check if the carried block's target position is occupied
        let is_block_blocked = blocks.iter().any(|b| b.position == target);
        
        // Check if any part of the player's body would be blocked
        let is_player_blocked = blocks.iter().enumerate()
            .filter(|(i, _)| *i != block_idx) // Ignore the block we're trying to move
            .any(|(_, b)| {
                // For each block, check all positions along the player's body
                for body_part in 0..self.body_size {
                    // Skip the head position if that's where we're carrying a block
                    if body_part == 0 && b.position == (player_target_x, self.position.1) {
                        // This is where the carried block would be - skip this check
                        continue;
                    }
                    
                    // Check if this part of the body would collide with any block
                    if b.position == (player_target_x, self.position.1 + body_part) {
                        return true;
                    }
                }
                false
            });
        
        if !is_block_blocked && !is_player_blocked {
            // Check if the block is at the player's head level (top of the player's body)
            let is_at_head_level = blocks[block_idx].position.1 == self.position.1;
            
            if is_at_head_level {
                // Calculate move direction based on target vs current position
                let move_direction = (block_target_x as isize - blocks[block_idx].position.0 as isize).signum();
                
                // Mark the block as carried and store the direction
                blocks[block_idx].carried = true;
                blocks[block_idx].carrying_direction = Some(move_direction);
            }
            
            // Move the falling block
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
    
    // Add a new method to release carried blocks
    pub fn release_carried_blocks(&self, blocks: &mut [Block], current_direction: Option<isize>) {
        for block in blocks.iter_mut() {
            if block.carried {
                // Only release if player is not pushing in the carrying direction
                if current_direction != block.carrying_direction {
                    block.carried = false;
                    block.falling = true;
                    block.carrying_direction = None;
                }
            }
        }
    }
    
    pub fn move_left(&mut self, blocks: &mut [Block]) {
        // Use the stored grid size from the Player struct
        self.move_horizontal(-1, self.grid_size, blocks);
    }
    
    pub fn move_right(&mut self, blocks: &mut [Block]) {
        // Use the stored grid size from the Player struct
        self.move_horizontal(1, self.grid_size, blocks);
    }
}
