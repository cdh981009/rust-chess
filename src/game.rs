use ggez::{graphics::Image, *};

use crate::board::*;

pub struct Assets {
    pub piece_images: Vec<Image>,
}

impl Assets {
    fn new(ctx: &mut Context) -> GameResult<Assets> {
        let mut piece_images = Vec::new();

        let pieces = "prbnkq";
        let colors = "wb";

        for color in colors.chars() {
            for piece in pieces.chars() {
                let path = format!("/{color}{piece}.png");
                piece_images.push(Image::from_path(ctx, path)?);
            }
        }

        Ok(Assets { piece_images })
    }
}

pub struct GameState {
    screen_width: f32,
    screen_height: f32,
    board: Board,
    assets: Assets,
}

impl GameState {
    pub fn new(ctx: &mut Context) -> GameResult<GameState> {
        let (screen_width, screen_height) = ctx.gfx.drawable_size();
        let board = Board::new();
        let assets = Assets::new(ctx)?;

        Ok(GameState {
            screen_width,
            screen_height,
            board,
            assets,
        })
    }
}

impl ggez::event::EventHandler<GameError> for GameState {
    fn update(&mut self, _ctx: &mut Context) -> GameResult {
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let mut canvas = graphics::Canvas::from_frame(ctx, graphics::Color::BLUE);
        let offset = (80.0, 80.0);

        self.board.draw(&mut canvas, &self.assets, offset)?;

        canvas.finish(ctx)?;

        Ok(())
    }
}
