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
    is_mouse_down: HashMap<MouseButton, bool>,
    is_mouse_pressed: HashMap<MouseButton, bool>,
    is_mouse_released: HashMap<MouseButton, bool>,
}

impl Mouse {
    fn update(&mut self) {
        for (key, val) in self.is_mouse_pressed.iter_mut() {
            *val = false;
        }

        for (key, val) in self.is_mouse_released.iter_mut() {
            *val = false;
        }
    }

    pub fn get_mouse(&self) -> (f32, f32) {
        (self.x, self.y)
    }

    pub fn is_mouse_down(&self, mouse_button: MouseButton) -> bool {
        *self.is_mouse_down.get(&mouse_button).unwrap_or(&false)
    }

    pub fn is_mouse_pressed(&self, mouse_button: MouseButton) -> bool {
        *self.is_mouse_pressed.get(&mouse_button).unwrap_or(&false)
    }

    pub fn is_mouse_released(&self, mouse_button: MouseButton) -> bool {
        *self.is_mouse_released.get(&mouse_button).unwrap_or(&false)
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

        let board_position = (80.0, 80.0);
        let board = Board::new(board_position);

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
        self.board.update(&self.mouse);

        // update mouse at the last moment
        self.mouse.update();

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let mut canvas = graphics::Canvas::from_frame(ctx, graphics::Color::WHITE);

        self.board
            .draw(ctx, &mut canvas, &mut self.assets)?;

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
        self.mouse.is_mouse_down.insert(button, true);
        self.mouse.is_mouse_pressed.insert(button, true);
        // println!("Mouse button pressed: {button:?}, x: {x}, y: {y}");

        Ok(())
    }

    fn mouse_button_up_event(
        &mut self,
        _ctx: &mut Context,
        button: MouseButton,
        x: f32,
        y: f32,
    ) -> GameResult {
        self.mouse.is_mouse_down.insert(button, false);
        // println!("Mouse button released: {button:?}, x: {x}, y: {y}");

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
        // println!("Mouse motion, x: {x}, y: {y}, relative x: {xrel}, relative y: {yrel}");
        Ok(())
    }
}
