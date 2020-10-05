mod constants {

    pub const CELL_SIZE: (i16, i16) = (15, 15);
    pub const GRID_SIZE: (i16, i16) = (80, 40);
    pub const WINDOW_SIZE: (f32, f32) = (
        (CELL_SIZE.0 * GRID_SIZE.0) as f32,
        (CELL_SIZE.1 * GRID_SIZE.1) as f32,
    );
    pub const DESIRED_FPS: f32 = 8.0;
    pub const MS_PER_UPDATE: f32 = 1.0 / DESIRED_FPS * 1000.0;
}

mod grid {
    use ggez::graphics;
    #[derive(Debug, PartialEq, Clone, Copy)]
    pub struct Coordinate {
        pub x: i16,
        pub y: i16,
    }
    impl Coordinate {
        pub fn new(x: i16, y: i16) -> Coordinate {
            Coordinate { x, y }
        }
        pub fn out_of_bounds(&mut self) -> bool {
            self.x >= crate::constants::GRID_SIZE.0
                || self.y >= crate::constants::GRID_SIZE.1
                || self.x < 0
                || self.y < 0
        }
    }
    impl From<Coordinate> for graphics::Rect {
        fn from(coordinate: Coordinate) -> graphics::Rect {
            let rect = graphics::Rect::new(
                (coordinate.x * crate::constants::CELL_SIZE.0) as f32,
                (coordinate.y * crate::constants::CELL_SIZE.1) as f32,
                crate::constants::CELL_SIZE.0.into(),
                crate::constants::CELL_SIZE.1.into(),
            );
            rect
        }
    }
    #[derive(Debug, PartialEq, Clone, Copy)]
    pub enum Direction {
        Left,
        Right,
        Up,
        Down,
    }
    impl Direction {
        pub fn inverse(&self) -> Direction {
            match self {
                Direction::Up => Direction::Down,
                Direction::Down => Direction::Up,
                Direction::Left => Direction::Right,
                Direction::Right => Direction::Left,
            }
        }
    }
}
mod things {
    use super::grid::Coordinate;
    use super::grid::Direction;
    use ggez::event::KeyCode;
    use ggez::{graphics, Context, GameResult};
    use rand::Rng;
    use std::collections::VecDeque;
    pub struct Snake {
        pub coordinate: Coordinate,
        pub tail: VecDeque<TailPart>,
        pub direction: Direction,
        pub new_direction: Option<Direction>,
    }
    impl Snake {
        pub fn new() -> Snake {
            let starting_direction = Direction::Right;
            let starting_coordinate = Coordinate::new(10, 10);
            Snake {
                coordinate: starting_coordinate,
                tail: Snake::init_tail(starting_coordinate),
                direction: starting_direction,
                new_direction: None,
            }
        }
        pub fn set_new_direction(keycode: KeyCode) -> Option<Direction> {
            match keycode {
                KeyCode::W => Some(Direction::Up),
                KeyCode::S => Some(Direction::Down),
                KeyCode::A => Some(Direction::Left),
                KeyCode::D => Some(Direction::Right),
                _ => None,
            }
        }

        fn init_tail(starting_coordinate: Coordinate) -> VecDeque<TailPart> {
            let mut vector: VecDeque<TailPart> = VecDeque::new();
            vector.push_back(TailPart{
                coordinate: Coordinate::new(starting_coordinate.x, starting_coordinate.y)
            });
            vector
        }
        pub fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
            let rect_mesh = graphics::Mesh::new_rectangle(
                ctx,
                ggez::graphics::DrawMode::fill(),
                self.coordinate.into(),
                graphics::BLACK,
            )?;
            graphics::draw(ctx, &rect_mesh, (ggez::mint::Point2 { x: 0.0, y: 0.0 },))?;
            for part in &self.tail {
                let part_mesh = graphics::Mesh::new_rectangle(
                    ctx,
                    ggez::graphics::DrawMode::fill(),
                    part.coordinate.into(),
                    graphics::BLACK,
                )?;
                graphics::draw(ctx, &part_mesh, (ggez::mint::Point2 { x: 0.0, y: 0.0 },))?;
            }
            Ok(())
        }
        pub fn update(&mut self) {
            match self.direction {
                super::grid::Direction::Up => self.coordinate.y = self.coordinate.y - 1,
                super::grid::Direction::Down => self.coordinate.y = self.coordinate.y + 1,
                super::grid::Direction::Left => self.coordinate.x = self.coordinate.x - 1,
                super::grid::Direction::Right => self.coordinate.x = self.coordinate.x + 1,
            }
        }
    }
    #[derive(Debug)]
    pub struct TailPart {
        coordinate: super::grid::Coordinate,
    }
    impl TailPart {
        pub fn new(x: i16, y: i16) -> TailPart {
            TailPart {
                coordinate: super::grid::Coordinate::new(x, y),
            }
        }
    }
    pub struct Food {
        pub coordinate: super::grid::Coordinate,
    }
    impl Food {
        pub fn new() -> Food {
            Food {
                coordinate: super::grid::Coordinate::new(20, 10),
            }
        }
        pub fn new_location(&mut self) {
            let mut rng = rand::thread_rng();
            let x = rng.gen_range(0, crate::constants::GRID_SIZE.0);
            let y = rng.gen_range(0, crate::constants::GRID_SIZE.1);

            self.coordinate.x = x;
            self.coordinate.y = y;
        }
        pub fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
            let color = graphics::Color::new(0.8, 0.2, 0.2, 1.0);

            let rect_mesh = graphics::Mesh::new_rectangle(
                ctx,
                ggez::graphics::DrawMode::fill(),
                self.coordinate.into(),
                color,
            )?;
            graphics::draw(ctx, &rect_mesh, (ggez::mint::Point2 { x: 0.0, y: 0.0 },))?;
            Ok(())
        }
    }
}
use ggez::conf;
use ggez::event::{self, EventHandler, KeyCode, KeyMods};
use ggez::{graphics, Context, ContextBuilder, GameResult};

use std::time::Instant;

struct MyGame {
    snake: things::Snake,
    last_update: Instant,
    score: u16,
    food: things::Food,
}

impl MyGame {
    pub fn new(_ctx: &mut Context) -> MyGame {
        MyGame {
            snake: things::Snake::new(),
            last_update: Instant::now(),
            score: 0,
            food: things::Food::new(),
        }
    }
    fn turn(&mut self) {
        match &self.snake.new_direction {
            Some(new_direction) => {
                if new_direction != &self.snake.direction.inverse() {
                    // If the direction of the snake is opposite to choosen direction we dont want to turn 180 degrees.
                    self.snake.direction = *new_direction;
                }
            }
            None => (),
        }
    }
    fn collision(&mut self) {
        if self.snake.coordinate.out_of_bounds() {
            println!("DEAD");
        }
        if self.snake.coordinate == self.food.coordinate {
            self.food.new_location();

            let new_tail_part = things::TailPart::new(self.snake.coordinate.x, self.snake.coordinate.y);
            self.snake.tail.push_back(new_tail_part);
        }else {
            let new_tail_part = things::TailPart::new(self.snake.coordinate.x, self.snake.coordinate.y);
            self.snake.tail.push_back(new_tail_part);
            self.snake.tail.pop_front();
        }
    }
}

impl EventHandler for MyGame {
    fn update(&mut self, _ctx: &mut Context) -> GameResult<()> {
        if Instant::now() - self.last_update
            >= std::time::Duration::from_millis(constants::MS_PER_UPDATE as u64)
        {
            self.last_update = Instant::now();
            self.snake.update();
            self.turn();
            self.collision();
        }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        let color = graphics::Color::from_rgba(135, 135, 235, 255);
        graphics::clear(ctx, color);
        self.food.draw(ctx)?;
        self.snake.draw(ctx)?;
        graphics::present(ctx)
    }

    fn key_down_event(
        &mut self,
        _ctx: &mut Context,
        keycode: KeyCode,
        _keymods: KeyMods,
        _repeat: bool,
    ) {
        // Get a new direction here, but set it in the update function so you cant bug it by
        // quickly turning twice to turn 180 degree.
        self.snake.new_direction = things::Snake::set_new_direction(keycode);
    }
}

fn main() -> GameResult<()> {
    let (mut ctx, mut event_loop) = ContextBuilder::new("Snake", "Henrik Zenkert")
        .window_mode(conf::WindowMode::default().dimensions(
            crate::constants::WINDOW_SIZE.0,
            crate::constants::WINDOW_SIZE.1,
        ))
        .build()?;

    let mut my_game = MyGame::new(&mut ctx);

    match event::run(&mut ctx, &mut event_loop, &mut my_game) {
        Ok(_) => println!("Exited cleanly."),
        Err(e) => println!("Error occured: {}", e),
    }
    Ok(())
}
