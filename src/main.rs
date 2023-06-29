#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod board;
mod game;
mod move_calculator;
mod piece;

use game::MainState;
use ggez::*;

const WINDOW_WIDTH: f32 = 800.0;
const WINDOW_HEIGHT: f32 = 800.0;

fn main() -> GameResult {
    // We add the CARGO_MANIFEST_DIR/resources to the resource paths
    // so that ggez will look in our cargo project directory for files.
    // let resource_dir = if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
    //     let mut path = path::PathBuf::from(manifest_dir);
    //     path.push("resources");
    //     path
    // } else {
    //     path::PathBuf::from("./resources")
    // };

    let c = conf::Conf::new();
    let (mut ctx, event_loop) = ContextBuilder::new("rust_chess", "cdh981009")
        .default_conf(c)
        .add_zipfile_bytes(include_bytes!("../resources.zip").to_vec())
        //.add_resource_path(resource_dir)
        .window_mode(conf::WindowMode::default().dimensions(WINDOW_WIDTH, WINDOW_HEIGHT))
        .build()
        .unwrap();

    let state = MainState::new(&mut ctx)?;

    event::run(ctx, event_loop, state);
}
