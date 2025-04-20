// Platform-specific implementation for ggez
use std::collections::{HashSet, VecDeque};

use ggez::event::EventHandler;
use ggez::graphics::{self, Canvas, Color, DrawParam, Mesh, Rect, Text};
use ggez::input::keyboard::{KeyCode, KeyInput};
use ggez::input::mouse::MouseButton;
use ggez::{Context, GameResult};

use crate::core::game::GameState;
use crate::core::types::{GameConfig, InputAction};

// Game adapter that wraps the core game state and handles ggez-specific functionality
pub struct GameAdapter {
    game_state: GameState,
    held_keys: HashSet<KeyCode>,
    keys_pressed_since_update: Vec<KeyCode>,
    direction_press_order: VecDeque<KeyCode>,
    restart_button: Rect,
    score_bar_height: f32,
}

impl GameAdapter {
    pub fn new(grid_size: usize, cell_size: f32, refresh_rate: u64, block_fall_speed: usize, block_spawn_rate: u64) -> Self {
        let config = GameConfig {
            grid_size,
            cell_size,
            refresh_rate_milliseconds: refresh_rate,
            block_fall_speed,
            block_spawn_rate,
        };

        Self {
            game_state: GameState::new(config),
            held_keys: HashSet::new(),
            keys_pressed_since_update: Vec::new(),
            direction_press_order: VecDeque::new(),
            restart_button: Rect::new(0.0, 0.0, 0.0, 0.0),
            score_bar_height: cell_size,
        }
    }

    // Convert from platform-specific representation to core representation
    fn determine_movement(&mut self) -> InputAction {
        // If no keys were pressed, return None
        if self.keys_pressed_since_update.is_empty() {
            return InputAction::None;
        }
        
        // Check if "Up" was pressed, prioritize jump
        if self.keys_pressed_since_update.contains(&KeyCode::Up) {
            return InputAction::Up;
        }
        
        // If we have direction keys in the order queue, return the last one
        if !self.direction_press_order.is_empty() {
            let last = self.direction_press_order.back().cloned();
            return match last {
                Some(KeyCode::Left) => InputAction::Left,
                Some(KeyCode::Right) => InputAction::Right,
                _ => InputAction::None,
            };
        }
        
        InputAction::None
    }

    // Determine the current movement direction based on held keys
    fn get_current_movement(&self) -> InputAction {
        if self.held_keys.contains(&KeyCode::Left) {
            InputAction::Left
        } else if self.held_keys.contains(&KeyCode::Right) {
            InputAction::Right
        } else {
            InputAction::None
        }
    }

    // Draw methods
    fn draw_grid(&self, ctx: &mut Context, canvas: &mut Canvas, y_offset: f32) -> GameResult {
        // Draw the grid lines
        for i in 0..=self.game_state.grid_size {
            let position = i as f32 * self.game_state.cell_size;

            // Horizontal line
            let h_line = Mesh::new_line(
                ctx,
                &[
                    ggez::glam::Vec2::new(0.0, position + y_offset),
                    ggez::glam::Vec2::new(self.game_state.cell_size * self.game_state.grid_size as f32, position + y_offset),
                ],
                1.0,
                Color::BLACK,
            )?;

            // Vertical line
            let v_line = Mesh::new_line(
                ctx,
                &[
                    ggez::glam::Vec2::new(position, y_offset),
                    ggez::glam::Vec2::new(position, self.game_state.cell_size * self.game_state.grid_size as f32 + y_offset),
                ],
                1.0,
                Color::BLACK,
            )?;

            canvas.draw(&h_line, DrawParam::default());
            canvas.draw(&v_line, DrawParam::default());
        }

        Ok(())
    }

    fn draw_score_bar(&self, ctx: &mut Context, canvas: &mut Canvas) -> GameResult {
        let score_bar = Mesh::new_rectangle(
            ctx,
            graphics::DrawMode::fill(),
            Rect::new(
                0.0,
                0.0,
                self.game_state.grid_size as f32 * self.game_state.cell_size,
                self.score_bar_height,
            ),
            Color::BLUE,
        )?;
        canvas.draw(&score_bar, DrawParam::default());
        
        let score_text = Text::new(format!("Score: {}", self.game_state.score));
        let text_x = 10.0; // Left padding
        let text_y = self.score_bar_height / 2.0;
        
        canvas.draw(
            &score_text, 
            DrawParam::default()
                .dest([text_x, text_y])
                .color(Color::WHITE)
                .offset([0.0, 0.5]) // Center vertically
        );
        
        Ok(())
    }

    fn draw_restart_button(&mut self, ctx: &mut Context, canvas: &mut Canvas) -> GameResult {
        let button_width = 80.0;
        let button_height = self.score_bar_height * 0.8;
        let button_x = self.game_state.grid_size as f32 * self.game_state.cell_size - button_width - 10.0;
        let button_y = (self.score_bar_height - button_height) / 2.0;
        
        self.restart_button = Rect::new(button_x, button_y, button_width, button_height);
        
        let restart_button_mesh = Mesh::new_rectangle(
            ctx,
            graphics::DrawMode::fill(),
            self.restart_button,
            Color::GREEN,
        )?;
        canvas.draw(&restart_button_mesh, DrawParam::default());
        
        let restart_text = Text::new("Restart");
        let restart_text_x = button_x + button_width / 2.0;
        let restart_text_y = button_y + button_height / 2.0;
        
        canvas.draw(
            &restart_text, 
            DrawParam::default()
                .dest([restart_text_x, restart_text_y])
                .color(Color::BLACK)
                .offset([0.5, 0.5])
        );
        
        Ok(())
    }

    fn draw_player(&self, ctx: &mut Context, canvas: &mut Canvas, y_offset: f32) -> GameResult {
        let player_pos = self.game_state.player.position;
        let player_mesh = Mesh::new_rectangle(
            ctx,
            graphics::DrawMode::fill(),
            Rect::new(
                player_pos.0 as f32 * self.game_state.cell_size,
                player_pos.1 as f32 * self.game_state.cell_size,
                self.game_state.cell_size,
                self.game_state.cell_size * self.game_state.player.body_size as f32,
            ),
            Color::RED,
        )?;
        canvas.draw(&player_mesh, DrawParam::default().dest([0.0, y_offset]));
        
        Ok(())
    }

    fn draw_blocks(&self, ctx: &mut Context, canvas: &mut Canvas, y_offset: f32) -> GameResult {
        for block in &self.game_state.blocks {
            let (x, y) = block.position;
            let block_mesh = Mesh::new_rectangle(
                ctx,
                graphics::DrawMode::fill(),
                Rect::new(
                    x as f32 * self.game_state.cell_size,
                    y as f32 * self.game_state.cell_size,
                    self.game_state.cell_size,
                    self.game_state.cell_size,
                ),
                Color::BLACK,
            )?;
            canvas.draw(&block_mesh, DrawParam::default().dest([0.0, y_offset]));
        }
        
        Ok(())
    }

    fn draw_game_over(&self, canvas: &mut Canvas) -> GameResult {
        if !self.game_state.game_over {
            return Ok(());
        }
        
        let window_width = self.game_state.grid_size as f32 * self.game_state.cell_size;
        let window_height = window_width + self.game_state.cell_size;
        let game_over_text = Text::new("Game Over");
        
        let text_x = window_width / 2.0;
        let text_y = window_height / 2.0;
        
        canvas.draw(
            &game_over_text, 
            DrawParam::default()
                .dest([text_x, text_y])
                .color(Color::RED)
                .scale([2.0, 2.0])
                .offset([0.5, 0.5])
        );
        
        Ok(())
    }
}

// Implement ggez EventHandler for the GameAdapter
impl EventHandler for GameAdapter {
    fn update(&mut self, _ctx: &mut Context) -> GameResult {
        // Skip updates if the game is over
        if self.game_state.game_over {
            return Ok(());
        }

        // Add held direction keys to the keys_pressed_since_update for continuous movement
        if self.held_keys.contains(&KeyCode::Left) {
            self.keys_pressed_since_update.push(KeyCode::Left);
            if !self.direction_press_order.contains(&KeyCode::Left) {
                self.direction_press_order.push_back(KeyCode::Left);
            }
        }
        if self.held_keys.contains(&KeyCode::Right) {
            self.keys_pressed_since_update.push(KeyCode::Right);
            if !self.direction_press_order.contains(&KeyCode::Right) {
                self.direction_press_order.push_back(KeyCode::Right);
            }
        }
        
        // Process the key presses according to priority rules
        let action = self.determine_movement();
        
        // Process the input in the game state
        self.game_state.process_input(action);
        
        // Clear keys pressed since update and direction order
        self.keys_pressed_since_update.clear();
        self.direction_press_order.clear();

        // Update game state
        self.game_state.update();

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let mut canvas = graphics::Canvas::from_frame(ctx, Color::WHITE);

        self.draw_score_bar(ctx, &mut canvas)?;
        self.draw_restart_button(ctx, &mut canvas)?;

        // Define the offset for all game elements
        let y_offset = self.score_bar_height;

        self.draw_grid(ctx, &mut canvas, y_offset)?;
        self.draw_player(ctx, &mut canvas, y_offset)?;
        self.draw_blocks(ctx, &mut canvas, y_offset)?;
        self.draw_game_over(&mut canvas)?;

        canvas.finish(ctx)?;
        Ok(())
    }

    fn key_down_event(
        &mut self,
        _ctx: &mut Context,
        key_input: KeyInput,
        _repeat: bool,
    ) -> GameResult {
        // Ignore input if game is over
        if self.game_state.game_over {
            return Ok(());
        }

        if let Some(keycode) = key_input.keycode {
            match keycode {
                KeyCode::Left | KeyCode::Right | KeyCode::Up => {
                    // Add to held keys
                    self.held_keys.insert(keycode);
                    
                    // Add to keys pressed since update
                    self.keys_pressed_since_update.push(keycode);
                    
                    // Update direction order for left/right keys
                    if keycode == KeyCode::Left || keycode == KeyCode::Right {
                        // Remove the key if it's already in the queue (to update its position)
                        if let Some(pos) = self.direction_press_order.iter().position(|&k| k == keycode) {
                            self.direction_press_order.remove(pos);
                        }
                        // Add it to the back (most recent)
                        self.direction_press_order.push_back(keycode);
                    }
                },
                _ => {}
            }
        }
        Ok(())
    }
    
    fn mouse_button_down_event(
        &mut self,
        _ctx: &mut Context,
        button: MouseButton,
        x: f32,
        y: f32,
    ) -> GameResult {
        if button == MouseButton::Left {
            // Check if click was inside the restart button
            if self.restart_button.contains([x, y]) {
                self.game_state.restart();
            }
        }
        Ok(())
    }

    // Add key up event handler to clear direction when keys are released
    fn key_up_event(
        &mut self,
        _ctx: &mut Context,
        key_input: KeyInput,
    ) -> GameResult {
        if let Some(keycode) = key_input.keycode {
            // Remove from held keys when released
            self.held_keys.remove(&keycode);
            
            // If up arrow is released and a direction key is still held,
            // add that direction key to keys_pressed_since_update to continue movement
            if keycode == KeyCode::Up {
                if self.held_keys.contains(&KeyCode::Left) {
                    self.keys_pressed_since_update.push(KeyCode::Left);
                    // Make sure it's also in the direction queue
                    if !self.direction_press_order.contains(&KeyCode::Left) {
                        self.direction_press_order.push_back(KeyCode::Left);
                    }
                } else if self.held_keys.contains(&KeyCode::Right) {
                    self.keys_pressed_since_update.push(KeyCode::Right);
                    // Make sure it's also in the direction queue
                    if !self.direction_press_order.contains(&KeyCode::Right) {
                        self.direction_press_order.push_back(KeyCode::Right);
                    }
                }
            }
        }
        Ok(())
    }
}
