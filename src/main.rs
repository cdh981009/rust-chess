use ggez::*;

fn main() {
    let c = conf::Conf::new();
    let (ctx, event_loop) = ContextBuilder::new("rust_chess", "cdh981009")
        .default_conf(c)
        .build()
        .unwrap();

    let state = GameState::new(&ctx);

    event::run(ctx, event_loop, state);
}

struct GameState {
    screen_width: f32,
    screen_height: f32,
}

impl GameState {
    fn new(ctx: &Context) -> GameState {
        let (screen_width, screen_height) = ctx.gfx.drawable_size();
        GameState {
            screen_width,
            screen_height,
        }
    }
}

impl ggez::event::EventHandler<GameError> for GameState {
    fn update(&mut self, _ctx: &mut Context) -> GameResult {
        Ok(())
    }

    fn draw(&mut self, _ctx: &mut Context) -> GameResult {
        Ok(())
    }
}
