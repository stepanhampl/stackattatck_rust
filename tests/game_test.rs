use rust_stackattack::game::GridGame;

#[test]
fn test_game_creation() {
    let grid_size = 10;
    let cell_size = 30.0;
    let refresh_rate = 200;
    let block_fall_speed = 1;
    let block_spawn_rate = 10;
    
    let game = GridGame::new(grid_size, cell_size, refresh_rate, block_fall_speed, block_spawn_rate);
    
    // Verify initial game properties
    assert_eq!(game.grid_size, grid_size);
    assert_eq!(game.cell_size, cell_size);
    assert_eq!(game.get_score(), 0);  // Score should start at 0
    assert!(!game.is_game_over());    // Game shouldn't be over initially
    
    // There should be at least one block spawned initially
    assert!(game.get_blocks().len() > 0);
}

// We would add more tests here for other game functionality
// For example, testing block spawning, collision detection, etc.
// However, many of these would require mocking ggez input and timing
// which is beyond the scope of this simple test suite
