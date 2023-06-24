use std::collections::HashMap;

use ggez::{event::MouseButton, graphics::Image, *};

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
            self.images
                .insert(key.clone(), Image::from_path(ctx, path)?);
        }

        Ok(self.images.get(key).expect("cannot load the image"))
    }
}

#[derive(Default)]
pub struct Mouse {
    x: f32,
    y: f32,
    is_mouse_down: bool,
    is_mouse_pressed: bool,
    is_mouse_released: bool,
}

impl Mouse {
    fn update(&mut self) {
        self.is_mouse_pressed = false;
        self.is_mouse_released = false;
    }

    pub fn get_mouse(&self) -> (f32, f32) {
        (self.x, self.y)
    }

    pub fn is_mouse_down(&self) -> bool {
        self.is_mouse_down
    }

    pub fn is_mouse_pressed(&self) -> bool {
        self.is_mouse_pressed
    }

    pub fn is_mouse_released(&self) -> bool {
        self.is_mouse_released
    }
}

pub struct GameState {
    screen_width: f32,
    screen_height: f32,
    board: Board,
    assets: Assets,
    mouse: Mouse,
}

impl GameState {
    pub fn new(ctx: &mut Context) -> GameResult<GameState> {
        let (screen_width, screen_height) = ctx.gfx.drawable_size();
        let board = Board::new();
        let assets = Assets::new(ctx);
        let mouse = Default::default();

        Ok(GameState {
            screen_width,
            screen_height,
            board,
            assets,
            mouse,
        })
    }
}

impl ggez::event::EventHandler<GameError> for GameState {
    fn update(&mut self, _ctx: &mut Context) -> GameResult {
        // update things here:

        // update mouse at the last moment
        self.mouse.update();

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let mut canvas = graphics::Canvas::from_frame(ctx, graphics::Color::WHITE);
        let offset = (80.0, 80.0);

        self.board
            .draw(ctx, &mut canvas, &mut self.assets, offset)?;

        canvas.finish(ctx)?;

        Ok(())
    }

    fn mouse_button_down_event(
        &mut self,
        _ctx: &mut Context,
        button: MouseButton,
        x: f32,
        y: f32,
    ) -> GameResult {
        self.mouse.is_mouse_down = true;
        println!("Mouse button pressed: {button:?}, x: {x}, y: {y}");

        Ok(())
    }

    fn mouse_button_up_event(
        &mut self,
        _ctx: &mut Context,
        button: MouseButton,
        x: f32,
        y: f32,
    ) -> GameResult {
        self.mouse.is_mouse_down = false;
        println!("Mouse button released: {button:?}, x: {x}, y: {y}");

        Ok(())
    }

    fn mouse_motion_event(
        &mut self,
        _ctx: &mut Context,
        x: f32,
        y: f32,
        xrel: f32,
        yrel: f32,
    ) -> GameResult {
        self.mouse.x = x;
        self.mouse.y = y;

        // If you change your screen coordinate system you need to calculate the
        // logical coordinates like this:
        /*
        let screen_rect = graphics::screen_coordinates(_ctx);
        let size = graphics::window(_ctx).inner_size();
        self.pos_x = (x / (size.width  as f32)) * screen_rect.w + screen_rect.x;
        self.pos_y = (y / (size.height as f32)) * screen_rect.h + screen_rect.y;
        */
        println!("Mouse motion, x: {x}, y: {y}, relative x: {xrel}, relative y: {yrel}");
        Ok(())
    }
}
