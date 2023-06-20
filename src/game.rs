use std::collections::HashMap;

use ggez::{graphics::Image, *};

use crate::board::*;

pub struct Assets {
    images: HashMap<String, Image>,
}

impl Assets {
    fn new(ctx: &mut Context) -> Assets {
        let images = HashMap::new();
        Assets { images }
    }

    pub fn try_get_image(&mut self, ctx: &mut Context, key: &String) -> GameResult<&Image> {
        if !self.images.contains_key(key) {
            let path = format!("/{key}.png");
            self.images.insert(key.clone(), Image::from_path(ctx, path)?);
        }

        Ok( self.images.get(key).expect("cannot load the image") )
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
        let assets = Assets::new(ctx);

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
        let mut canvas = graphics::Canvas::from_frame(ctx, graphics::Color::WHITE);
        let offset = (80.0, 80.0);

        self.board.draw(ctx, &mut canvas, &mut self.assets, offset)?;

        canvas.finish(ctx)?;

        Ok(())
    }
}
