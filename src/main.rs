use std::{env, path};

mod board;
mod game;

use game::GameState;
use ggez::*;

fn main() -> GameResult {
    // We add the CARGO_MANIFEST_DIR/resources to the resource paths
    // so that ggez will look in our cargo project directory for files.
    let resource_dir = if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
        let mut path = path::PathBuf::from(manifest_dir);
        path.push("resources");
        path
    } else {
        path::PathBuf::from("./resources")
    };

    let c = conf::Conf::new();
    let (mut ctx, event_loop) = ContextBuilder::new("rust_chess", "cdh981009")
        .default_conf(c)
        .add_resource_path(resource_dir)
        .window_mode(conf::WindowMode::default().dimensions(800.0, 800.0))
        .build()
        .unwrap();

    let state = GameState::new(&mut ctx)?;

    event::run(ctx, event_loop, state);
}
