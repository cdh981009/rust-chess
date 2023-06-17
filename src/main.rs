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
    board: Board,
}

const BOARD_WIDTH: usize = 8;
const BOARD_HEIGHT: usize = 8;

struct Board {
    pieces: Vec<Vec<Option<Piece>>>,
}

#[derive(Clone)]
struct Piece {
    piece_type: PieceType,
    color: Color,
}

#[derive(Clone)]
enum PieceType {
    Pawn,
    Rook,
    Bishop,
    Knight,
    King,
    Queen,
}

#[derive(Clone)]
enum Color {
    White,
    Black,
}

impl GameState {
    fn new(ctx: &Context) -> GameState {
        let (screen_width, screen_height) = ctx.gfx.drawable_size();
        let board = Board::new();

        GameState {
            screen_width,
            screen_height,
            board,
        }
    }
}

impl Board {
    fn new() -> Self {
        Board {
            pieces: vec![vec![None; BOARD_WIDTH]; BOARD_HEIGHT],
        }
    }

    fn draw(&self, canvas: &mut graphics::Canvas, (x, y): (f32, f32)) -> GameResult {
        let cell_size = 50.0;

        // 1. draw board
        for i in 0..BOARD_HEIGHT {
            for j in 0..BOARD_WIDTH {
                let color = if (i + j) % 2 == 0 {
                    graphics::Color::WHITE
                } else {
                    graphics::Color::BLACK
                };

                canvas.draw(
                    &graphics::Quad,
                    graphics::DrawParam::default()
                        .color(color)
                        .scale([cell_size, cell_size])
                        .dest([x + cell_size * j as f32, y + cell_size * i as f32]),
                );
            }
        }

        // 2. draw pieces on the board
        // TODO

        Ok(())
    }
}

impl ggez::event::EventHandler<GameError> for GameState {
    fn update(&mut self, _ctx: &mut Context) -> GameResult {
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let mut canvas = graphics::Canvas::from_frame(ctx, graphics::Color::BLUE);
        let offset = (100.0, 100.0);

        self.board.draw(&mut canvas, offset)?;

        canvas.finish(ctx)?;

        Ok(())
    }
}
