use ggez::glam::Vec2;
use ggez::graphics::{self, Color, DrawParam, Mesh};
use ggez::{Context, GameResult};

pub fn draw_grid(
    ctx: &mut Context, 
    canvas: &mut graphics::Canvas,
    grid_size: usize,
    cell_size: f32,
    y_offset: f32, // New parameter for vertical offset
) -> GameResult {
    // Draw the grid lines
    for i in 0..=grid_size {
        let position = i as f32 * cell_size;

        // Horizontal line
        let h_line = Mesh::new_line(
            ctx,
            &[
                Vec2::new(0.0, position + y_offset),
                Vec2::new(cell_size * grid_size as f32, position + y_offset),
            ],
            1.0,
            Color::BLACK,
        )?;

        // Vertical line
        let v_line = Mesh::new_line(
            ctx,
            &[
                Vec2::new(position, y_offset),
                Vec2::new(position, cell_size * grid_size as f32 + y_offset),
            ],
            1.0,
            Color::BLACK,
        )?;

        canvas.draw(&h_line, DrawParam::default());
        canvas.draw(&v_line, DrawParam::default());
    }

    Ok(())
}
