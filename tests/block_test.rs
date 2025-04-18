use rust_stackattack::block::{Block, spawn_random_block};

#[test]
fn test_block_creation() {
    let block = Block::new((5, 10));
    assert_eq!(block.position, (5, 10));
    assert!(block.falling);
    assert!(!block.carried);
    assert_eq!(block.carrying_direction, None);
}

#[test]
fn test_spawn_random_block() {
    let grid_size = 10;
    let block = spawn_random_block(grid_size);
    
    // Check that x position is within range
    assert!(block.position.0 < grid_size);
    // Check that y position is 0 (top of grid)
    assert_eq!(block.position.1, 0);
    // Check that block is falling
    assert!(block.falling);
}
