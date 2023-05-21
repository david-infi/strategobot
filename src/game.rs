pub mod action;
pub mod logic;
pub mod position;
pub mod rank;
pub mod state;

pub use action::*;
pub use position::*;
pub use rank::*;
pub use state::*;

const ALL_DIRECTION_STEPPERS: [fn(&Position) -> Position; 4] = [
    Position::up,
    Position::right,
    Position::down,
    Position::left,
];

pub enum Direction {
    Up,
    Right,
    Down,
    Left,
}

impl Direction {
    fn to_stepper(&self) -> fn(&Position) -> Position {
        use Direction::*;

        match self {
            Up => Position::up,
            Right => Position::right,
            Down => Position::down,
            Left => Position::left,
        }
    }
}
