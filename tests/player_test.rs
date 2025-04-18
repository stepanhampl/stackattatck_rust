use rust_stackattack::player::Player;
use rust_stackattack::block::Block;

#[test]
fn test_player_creation() {
    let grid_size = 10;
    let player = Player::new(grid_size);
    
    // Player should start at bottom middle of grid
    assert_eq!(player.position.0, grid_size / 2 - 1); // For even grid size
    assert_eq!(player.position.1, grid_size - player.body_size);
    
    // Player should start on ground
    assert!(!player.in_air);
    assert!(!player.is_falling);
}

#[test]
fn test_player_jump() {
    let grid_size = 10;
    let mut player = Player::new(grid_size);
    let initial_y = player.position.1;
    
    // Player should be able to jump
    player.jump();
    assert!(player.in_air);
    assert_eq!(player.position.1, initial_y - 1);
    
    // Player should not be able to jump again while in air
    let air_y = player.position.1;
    player.jump();
    assert_eq!(player.position.1, air_y); // Position shouldn't change
}

#[test]
fn test_player_has_support() {
    let grid_size = 10;
    let player = Player::new(grid_size);
    let mut blocks = Vec::new();
    
    // Player should have support from the ground
    assert!(player.has_support(&blocks, grid_size));
    
    // Create a player in mid-air with no blocks below
    let mut mid_air_player = Player::new(grid_size);
    mid_air_player.position.1 = grid_size / 2;
    assert!(!mid_air_player.has_support(&blocks, grid_size));
    
    // Add a block below the player for support
    blocks.push(Block {
        position: (mid_air_player.position.0, mid_air_player.position.1 + mid_air_player.body_size),
        falling: false,
        carried: false,
        carrying_direction: None,
    });
    assert!(mid_air_player.has_support(&blocks, grid_size));
}

#[test]
fn test_player_horizontal_movement() {
    let grid_size = 10;
    let mut player = Player::new(grid_size);
    let mut blocks = Vec::new();
    
    // Get initial position
    let initial_position = player.position.0;
    
    // Test moving left
    player.move_left(&mut blocks);
    assert_eq!(player.position.0, initial_position - 1, "Player should move left by one block");
    
    // Test moving right from the new position
    let position_after_left = player.position.0;
    player.move_right(&mut blocks);
    assert_eq!(player.position.0, position_after_left + 1, "Player should move right by one block");
}

// Test single left movement from starting position
#[test]
fn test_player_move_left_only() {
    let grid_size = 10;
    let mut player = Player::new(grid_size);
    let mut blocks = Vec::new();
    
    // Get initial position
    let initial_position = player.position.0;
    
    // Test moving left
    player.move_left(&mut blocks);
    assert_eq!(player.position.0, initial_position - 1, "Player should move left by one block");
}

// Test single right movement from starting position
#[test]
fn test_player_move_right_only() {
    let grid_size = 10;
    let mut player = Player::new(grid_size);
    let mut blocks = Vec::new();
    
    // Get initial position
    let initial_position = player.position.0;
    
    // Test moving right
    player.move_right(&mut blocks);
    assert_eq!(player.position.0, initial_position + 1, "Player should move right by one block");
}

// Test multiple left movements
#[test]
fn test_player_multiple_left_movements() {
    let grid_size = 10;
    let mut player = Player::new(grid_size);
    let mut blocks = Vec::new();
    
    // Get initial position
    let initial_position = player.position.0;
    
    // Ensure we don't go out of bounds
    let move_count = initial_position.min(3);
    
    // Move left multiple times
    for i in 1..=move_count {
        player.move_left(&mut blocks);
        assert_eq!(player.position.0, initial_position - i, 
                   "Player position incorrect after {} left moves", i);
    }
}

// Test multiple right movements
#[test]
fn test_player_multiple_right_movements() {
    let grid_size = 10;
    let mut player = Player::new(grid_size);
    let mut blocks = Vec::new();
    
    // Get initial position
    let initial_position = player.position.0;
    
    // Ensure we don't go out of bounds
    let move_count = 3.min(grid_size - 1 - initial_position);
    
    // Move right multiple times
    for i in 1..=move_count {
        player.move_right(&mut blocks);
        assert_eq!(player.position.0, initial_position + i, 
                   "Player position incorrect after {} right moves", i);
    }
}

// Test boundary condition - left edge
#[test]
fn test_player_left_boundary() {
    let grid_size = 10;
    let mut player = Player::new(grid_size);
    let mut blocks = Vec::new();
    
    // Get initial position
    let initial_position = player.position.0;
    
    // Move player to the left edge (move exactly initial_position times)
    for _ in 0..initial_position {
        player.move_left(&mut blocks);
    }
    
    // Now player should be at position 0
    assert_eq!(player.position.0, 0, "Player should be at the left boundary");
    
    // Try to move left once more (should be blocked by boundary)
    player.move_left(&mut blocks);
    assert_eq!(player.position.0, 0, "Player should not move beyond the left boundary");
}

// Test boundary condition - right edge
#[test]
fn test_player_right_boundary() {
    let grid_size = 10;
    let mut player = Player::new(grid_size);
    let mut blocks = Vec::new();
    
    // Get initial position
    let initial_position = player.position.0;
    
    // Calculate how many moves needed to reach the right boundary
    let moves_needed = grid_size - 1 - initial_position;
    
    // Move player to the right edge
    for _ in 0..moves_needed {
        player.move_right(&mut blocks);
    }
    
    // Now player should be at the right boundary
    assert_eq!(player.position.0, grid_size - 1, "Player should be at the right boundary");
    
    // Try to move right once more (should be blocked by boundary)
    player.move_right(&mut blocks);
    assert_eq!(player.position.0, grid_size - 1, "Player should not move beyond the right boundary");
}

// Test movement during fall delay
#[test]
fn test_player_movement_during_fall_delay() {
    let grid_size = 10;
    let mut player = Player::new(grid_size);
    let mut blocks = Vec::new();
    
    // Put player in mid-air with no support below to trigger fall delay
    player.position.1 = grid_size / 2;
    
    // Make sure player is not already in "in_air" state from jumping
    player.in_air = false;
    player.is_falling = false;
    
    // Update falling state to start the fall delay counter
    player.update_falling_state(&blocks, grid_size);
    
    // Try to move immediately after starting fall delay
    let position_before_move = player.position.0;
    player.move_left(&mut blocks);
    
    // Player shouldn't be able to move horizontally during fall delay
    assert_eq!(player.position.0, position_before_move, 
               "Player should not move horizontally during fall delay");
}

// Test alternating left-right movement
#[test]
fn test_player_alternating_movement() {
    let grid_size = 10;
    let mut player = Player::new(grid_size);
    let mut blocks = Vec::new();
    
    let initial_position = player.position.0;
    
    // Move left
    player.move_left(&mut blocks);
    let pos_after_left = player.position.0;
    assert_eq!(pos_after_left, initial_position - 1, "Player should move left by one block");
    
    // Move right
    player.move_right(&mut blocks);
    let pos_after_right = player.position.0;
    assert_eq!(pos_after_right, pos_after_left + 1, "Player should move right by one block");
    
    // Move left again
    player.move_left(&mut blocks);
    let pos_after_left_again = player.position.0;
    assert_eq!(pos_after_left_again, pos_after_right - 1, "Player should move left by one block again");
    
    // Move right again
    player.move_right(&mut blocks);
    let pos_after_right_again = player.position.0;
    assert_eq!(pos_after_right_again, pos_after_left_again + 1, "Player should move right by one block again");
}

// Test basic horizontal movement back and forth
#[test]
fn test_basic_horizontal_movement() {
    let grid_size = 10;
    let mut player = Player::new(grid_size);
    let mut blocks = Vec::new();
    
    // Store initial position
    let initial_x = player.position.0;
    
    // Move left and verify
    player.move_left(&mut blocks);
    let left_x = player.position.0;
    assert_eq!(left_x, initial_x - 1, "Player should move left by one");
    
    // Move right to return to initial position
    player.move_right(&mut blocks);
    assert_eq!(player.position.0, initial_x, "Player should return to initial position");
}

// Debug test with detailed position tracking
#[test]
fn test_debug_horizontal_movement() {
    let grid_size = 10;
    let mut player = Player::new(grid_size);
    let mut blocks = Vec::new();
    
    // Store initial position
    let initial_x = player.position.0;
    println!("Initial position: {}", initial_x);
    
    // Move left and verify
    player.move_left(&mut blocks);
    let after_left = player.position.0;
    println!("After move_left: {}", after_left);
    
    // Move right and verify
    player.move_right(&mut blocks);
    let after_right = player.position.0;
    println!("After move_right: {}", after_right);
    
    // Check if player returned to original position
    assert_eq!(after_right, initial_x, "Player should return to initial position after moving left then right");
}
