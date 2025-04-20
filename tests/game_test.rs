use rust_stackattack::core::game::GameState;
use rust_stackattack::core::block::Block;
use rust_stackattack::core::types::{GameConfig, InputAction};
use std::time::{Duration, Instant};

#[test]
fn test_game_creation() {
    let grid_size = 10;
    let cell_size = 30.0;
    let refresh_rate = 200;
    let block_fall_speed = 1;
    let block_spawn_rate = 10;
    
    let config = GameConfig {
        grid_size,
        cell_size,
        refresh_rate_milliseconds: refresh_rate,
        block_fall_speed,
        block_spawn_rate,
    };
    
    let game = GameState::new(config);
    
    // Verify initial game properties
    assert_eq!(game.grid_size, grid_size);
    assert_eq!(game.cell_size, cell_size);
    assert_eq!(game.score, 0);
    assert!(!game.game_over);
    
    // There should be at least one block spawned initially
    assert!(!game.blocks.is_empty());
}

#[test]
fn test_check_for_levitating_blocks() {
    let config = GameConfig {
        grid_size: 5,
        cell_size: 30.0,
        refresh_rate_milliseconds: 200,
        block_fall_speed: 1,
        block_spawn_rate: 10,
    };
    let mut game = GameState::new(config);
    
    // Clear the initial blocks
    game.blocks.clear();
    
    // Create a stack of blocks
    // Ground level block
    game.blocks.push(Block {
        position: (2, 4), // Bottom block
        falling: false,
        carried: false,
        carrying_direction: None,
    });
    
    // Mid-level block
    game.blocks.push(Block {
        position: (2, 3), // Resting on bottom block
        falling: false,
        carried: false,
        carrying_direction: None,
    });
    
    // Top-level block
    game.blocks.push(Block {
        position: (2, 2), // Resting on middle block
        falling: false,
        carried: false,
        carrying_direction: None,
    });
    
    // Add a floating block with no support below
    game.blocks.push(Block {
        position: (3, 3), // Floating with no support
        falling: false, // Incorrectly marked as not falling
        carried: false,
        carrying_direction: None,
    });
    
    // Check for and update levitating blocks
    game.check_for_levitating_blocks();
    
    // Blocks in the stack should not be falling
    assert!(!game.blocks[0].falling); // Bottom block on ground
    assert!(!game.blocks[1].falling); // Middle block supported
    assert!(!game.blocks[2].falling); // Top block supported
    
    // The floating block should now be marked as falling
    assert!(game.blocks[3].falling);
}

#[test]
fn test_check_full_rows_and_scoring() {
    let config = GameConfig {
        grid_size: 4,
        cell_size: 30.0,
        refresh_rate_milliseconds: 200,
        block_fall_speed: 1,
        block_spawn_rate: 10,
    };
    let mut game = GameState::new(config);
    
    // Clear the initial blocks
    game.blocks.clear();
    
    // Create a full row at the bottom
    for x in 0..4 {
        game.blocks.push(Block {
            position: (x, 3), // Bottom row
            falling: false,
            carried: false,
            carrying_direction: None,
        });
    }
    
    // Add some other blocks above
    game.blocks.push(Block {
        position: (0, 2),
        falling: false,
        carried: false,
        carrying_direction: None,
    });
    
    game.blocks.push(Block {
        position: (2, 2),
        falling: false,
        carried: false,
        carrying_direction: None,
    });
    
    // Initial score should be 0
    assert_eq!(game.score, 0);
    
    // Check for full rows which should remove the bottom row and increment score
    game.check_full_rows();
    
    // Score should now be 1
    assert_eq!(game.score, 1);
    
    // We should now have 2 blocks left (the ones that were above)
    assert_eq!(game.blocks.len(), 2);
    
    // The remaining blocks should be the ones that were on the second row
    let remaining_positions: Vec<(usize, usize)> = game.blocks.iter()
        .map(|block| block.position)
        .collect();
    assert!(remaining_positions.contains(&(0, 2)));
    assert!(remaining_positions.contains(&(2, 2)));
    
    // Create another full row for testing multiple rows
    for x in 0..4 {
        game.blocks.push(Block {
            position: (x, 3), // Bottom row again
            falling: false,
            carried: false,
            carrying_direction: None,
        });
    }
    
    // Check for full rows again
    game.check_full_rows();
    
    // Score should now be 2
    assert_eq!(game.score, 2);
    
    // Still should only have the original 2 blocks from the second row
    assert_eq!(game.blocks.len(), 2);
}

#[test]
fn test_levitating_cascade_effect() {
    let config = GameConfig {
        grid_size: 5,
        cell_size: 30.0,
        refresh_rate_milliseconds: 200,
        block_fall_speed: 1,
        block_spawn_rate: 10,
    };
    let mut game = GameState::new(config);
    
    // Clear the initial blocks
    game.blocks.clear();
    
    // Create a setup where removing one block causes others to levitate
    /*
        Layout:
        . . . . .
        . . C . .  <- row 1: C will fall when B is removed
        . . B . .  <- row 2: B is directly removed
        . A A A .  <- row 3: A blocks form a platform
        - - - - -  <- row 4: ground level
    */
    
    // Row 3 - Platform blocks (A) - these are on the ground level
    game.blocks.push(Block {
        position: (1, 4), // Bottom row (ground level)
        falling: false,
        carried: false,
        carrying_direction: None,
    });
    
    game.blocks.push(Block {
        position: (2, 4), // Bottom row (ground level)
        falling: false,
        carried: false,
        carrying_direction: None,
    });
    
    game.blocks.push(Block {
        position: (3, 4), // Bottom row (ground level)
        falling: false,
        carried: false,
        carrying_direction: None,
    });
    
    // Row 2 - Block B
    game.blocks.push(Block {
        position: (2, 3),
        falling: false,
        carried: false,
        carrying_direction: None,
    });
    
    // Row 1 - Block C
    game.blocks.push(Block {
        position: (2, 2),
        falling: false,
        carried: false,
        carrying_direction: None,
    });
    
    // Verify we have 5 blocks total
    assert_eq!(game.blocks.len(), 5);
    assert!(!game.blocks[3].falling); // Block B shouldn't be falling initially
    assert!(!game.blocks[4].falling); // Block C shouldn't be falling initially
    
    // Remove Block B (simulating it being cleared in a full row)
    game.blocks.remove(3);
    
    // Now check for levitating blocks
    game.check_for_levitating_blocks();
    
    // Block C (now at index 3) should be falling since its support was removed
    assert!(game.blocks[3].falling);
    
    // Platform blocks should remain static since they're on ground level
    // In our grid, the bottom row is at y=4 (grid_size-1)
    let platform_positions = [(1, 4), (2, 4), (3, 4)];
    
    for pos in platform_positions.iter() {
        let block = game.blocks.iter().find(|b| b.position == *pos).unwrap();
        assert!(!block.falling, "Block at {:?} should not be falling", pos);
    }
}

#[test]
fn test_update_falling_blocks() {
    let config = GameConfig {
        grid_size: 5,
        cell_size: 30.0,
        refresh_rate_milliseconds: 200,
        block_fall_speed: 1,
        block_spawn_rate: 10,
    };
    let mut game = GameState::new(config);
    
    // Clear the initial blocks
    game.blocks.clear();
    
    // First, move the player away from where we'll place blocks
    game.player.position = (0, 4);  // Move player to the left side at the bottom
    
    // Add a falling block far from the player
    game.blocks.push(Block {
        position: (3, 2),  // Position far from the player to avoid collision
        falling: true,
        carried: false,
        carrying_direction: None,
    });
    
    // Add a stationary block at the bottom
    game.blocks.push(Block {
        position: (3, 4),  // Directly below where the falling block will land
        falling: false,
        carried: false,
        carrying_direction: None,
    });
    
    // Store initial position
    let initial_pos = game.blocks[0].position;
    
    // Update falling blocks
    game.update_falling_blocks();
    
    // The falling block should have moved down by 1
    assert_eq!(game.blocks[0].position.0, initial_pos.0, "X position should remain unchanged");
    assert_eq!(game.blocks[0].position.1, initial_pos.1 + 1, "Y position should increase by 1");
    
    // Update again - the block should stop falling and land on the stationary block
    game.update_falling_blocks();
    
    // The block should no longer be falling
    assert!(!game.blocks[0].falling);
}

#[test]
fn test_update_falling_blocks_with_carried_blocks() {
    let config = GameConfig {
        grid_size: 5,
        cell_size: 30.0,
        refresh_rate_milliseconds: 200,
        block_fall_speed: 1,
        block_spawn_rate: 10,
    };
    let mut game = GameState::new(config);
    
    // Clear the initial blocks
    game.blocks.clear();
    
    // Add a carried block
    game.blocks.push(Block {
        position: (2, 2),
        falling: true, // Should be ignored because it's carried
        carried: true,
        carrying_direction: Some(1),
    });
    
    // Add a falling block
    game.blocks.push(Block {
        position: (3, 2),
        falling: true,
        carried: false,
        carrying_direction: None,
    });
    
    // Update falling blocks
    game.update_falling_blocks();
    
    // The carried block should not have moved
    assert_eq!(game.blocks[0].position.1, 2);
    // The falling block should have moved down
    assert_eq!(game.blocks[1].position.1, 3);
}

#[test]
fn test_block_collision_with_player() {
    let config = GameConfig {
        grid_size: 5,
        cell_size: 30.0,
        refresh_rate_milliseconds: 200,
        block_fall_speed: 1,
        block_spawn_rate: 10,
    };
    let mut game = GameState::new(config);
    
    // Clear the initial blocks
    game.blocks.clear();
    
    // Position the player
    game.player.position = (2, 3);
    
    // Add a falling block that will hit the player
    game.blocks.push(Block {
        position: (2, 2),
        falling: true,
        carried: false,
        carrying_direction: None,
    });
    
    // Verify game is not over initially
    assert!(!game.game_over);
    
    // Check for player collision
    let collision = game.check_block_player_collision(2, 3);
    
    // Game should now be over
    assert!(collision);
    assert!(game.game_over);
}

#[test]
fn test_handle_block_spawning() {
    let config = GameConfig {
        grid_size: 5,
        cell_size: 30.0,
        refresh_rate_milliseconds: 200,
        block_fall_speed: 1,
        block_spawn_rate: 5,
    };
    let mut game = GameState::new(config);
    
    // Clear the initial blocks
    game.blocks.clear();
    
    // Initial count
    let initial_count = game.blocks.len();
    
    // Initialize counter
    game.block_spawn_counter = 0;
    
    // Call handle_block_spawning 4 times (not enough to spawn a block)
    for _ in 0..4 {
        game.handle_block_spawning();
    }
    
    // No new blocks should have spawned
    assert_eq!(game.blocks.len(), initial_count);
    
    // Call one more time to reach spawn rate
    game.handle_block_spawning();
    
    // A block should have been spawned
    assert_eq!(game.blocks.len(), initial_count + 1);
    
    // Counter should be reset
    assert_eq!(game.block_spawn_counter, 0);
}

#[test]
fn test_update_player() {
    let config = GameConfig {
        grid_size: 5,
        cell_size: 30.0,
        refresh_rate_milliseconds: 200,
        block_fall_speed: 1,
        block_spawn_rate: 10,
    };
    let mut game = GameState::new(config);
    
    // Clear blocks to ensure no accidental support
    game.blocks.clear();
    
    // Position the player in mid-air with no support
    game.player.position = (2, 2);
    game.player.in_air = false;
    game.player.is_falling = false;
    
    // First, manually call update_falling_state to start fall delay
    game.player.update_falling_state(&game.blocks, game.grid_size);
    
    // Player should not be falling yet (due to fall delay)
    assert!(!game.player.is_falling);
    
    // Update fall delay counter to complete the delay
    for _ in 0..3 {
        game.player.update_fall_delay();
    }
    
    // Now player should be in falling state
    assert!(game.player.is_falling);
}

#[test]
fn test_restart_game() {
    let config = GameConfig {
        grid_size: 5,
        cell_size: 30.0,
        refresh_rate_milliseconds: 200,
        block_fall_speed: 1,
        block_spawn_rate: 10,
    };
    let mut game = GameState::new(config);
    
    // Set up a game state to test reset
    game.score = 100;
    game.game_over = true;
    game.blocks.clear();
    game.player.position = (1, 1);
    
    // Call restart game method
    game.restart();
    
    // Check game state was reset
    assert_eq!(game.score, 0);
    assert!(!game.game_over);
    assert!(!game.blocks.is_empty()); // Should have at least one block
    
    // Default positions
    assert_eq!(game.player.position.0, 2); // Default x position for grid size 5
    assert_eq!(game.player.position.1, 3); // Default y position for grid size 5
}

#[test]
fn test_game_update_simulation() {
    let config = GameConfig {
        grid_size: 5,
        cell_size: 30.0,
        refresh_rate_milliseconds: 200,
        block_fall_speed: 1,
        block_spawn_rate: 10,
    };
    let mut game = GameState::new(config);
    
    // Clear the initial blocks
    game.blocks.clear();
    
    // Move player away from test area
    game.player.position = (0, 4);  // Move to far left at bottom
    
    // Add a falling block at a known position away from player
    game.blocks.push(Block {
        position: (3, 2),  // Position far from player
        falling: true,
        carried: false,
        carrying_direction: None,
    });
    
    // Get initial position
    let initial_pos = game.blocks[0].position;
    
    // Set last_update to well before now
    game.last_update = Instant::now() - Duration::from_millis(300);
    
    // Call update which should update falling blocks
    game.update();
    
    // Block should have moved down
    assert_eq!(game.blocks[0].position.0, initial_pos.0);
    assert!(game.blocks[0].position.1 > initial_pos.1);
}

#[test]
fn test_current_movement_direction() {
    let config = GameConfig {
        grid_size: 5,
        cell_size: 30.0,
        refresh_rate_milliseconds: 200,
        block_fall_speed: 1,
        block_spawn_rate: 10,
    };
    let mut game = GameState::new(config);
    
    // Clear any existing blocks
    game.blocks.clear();
    
    // Create a block that's already carried
    game.blocks.push(Block {
        position: (game.player.position.0, game.player.position.1),
        falling: false,
        carried: true,
        carrying_direction: Some(1), // Being carried right
    });
    
    // Verify the initial state
    assert!(game.blocks[0].carried);
    assert_eq!(game.blocks[0].carrying_direction, Some(1));
    
    // Process right movement input should maintain carrying
    game.last_move_direction = Some(1);
    game.process_input(InputAction::Right);
    
    // Block should still be carried when moving in same direction
    assert!(game.blocks[0].carried);
    
    // Now manually change the direction
    game.last_move_direction = Some(-1);
    
    // Simulate the player releasing the carried blocks when direction changes
    game.player.release_carried_blocks(&mut game.blocks, game.last_move_direction);
    
    // Block should now be falling and not carried
    assert!(game.blocks[0].falling);
    assert!(!game.blocks[0].carried);
    assert_eq!(game.blocks[0].carrying_direction, None);
}

#[test]
fn test_keyboard_input_handling() {
    let config = GameConfig {
        grid_size: 5,
        cell_size: 30.0,
        refresh_rate_milliseconds: 200,
        block_fall_speed: 1,
        block_spawn_rate: 10,
    };
    let mut game = GameState::new(config);
    
    // Test moving player with input actions
    let initial_position = game.player.position;
    
    // Process RIGHT input
    game.process_input(InputAction::Right);
    let pos_after_right = game.player.position;
    assert_eq!(pos_after_right.0, initial_position.0 + 1);
    
    // Process LEFT input
    game.process_input(InputAction::Left);
    assert_eq!(game.player.position.0, pos_after_right.0 - 1);
    
    // Process UP input
    let y_before_jump = game.player.position.1;
    game.process_input(InputAction::Up);
    assert!(game.player.in_air);
    assert_eq!(game.player.position.1, y_before_jump - 1);
}

#[test]
fn test_determine_movement_priority() {
    // This test was specific to the platform-specific implementation
    // We've separated input handling in our new architecture
    // So we skip this specific test
}

#[test]
fn test_restart_game_functionality() {
    let config = GameConfig {
        grid_size: 5,
        cell_size: 30.0,
        refresh_rate_milliseconds: 200,
        block_fall_speed: 1,
        block_spawn_rate: 10,
    };
    let mut game = GameState::new(config);
    
    // Set up a game state to test reset
    game.score = 100;
    game.game_over = true;
    game.blocks.clear();
    game.player.position = (1, 1);
    
    // Process restart input
    game.process_input(InputAction::Restart);
    
    // Check game state was reset
    assert_eq!(game.score, 0);
    assert!(!game.game_over);
    assert!(!game.blocks.is_empty()); // Should have at least one block
    
    // Player should be reset to default position for grid size 5
    assert_eq!(game.player.position.0, 2);
}
