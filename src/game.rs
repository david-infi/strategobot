pub mod action;
pub mod logic;
pub mod position;
pub mod rank;
pub mod state;

pub use action::*;
pub use position::*;
pub use rank::*;
pub use state::*;

pub enum Direction {
    Up,
    Right,
    Down,
    Left,
}

const ALL_DIRECTIONS: [Direction; 4] = [
    Direction::Up,
    Direction::Right,
    Direction::Down,
    Direction::Left,
];

const ALL_DIRECTION_STEPPERS: [fn(&Position) -> Position; 4] = [
    Position::up,
    Position::right,
    Position::down,
    Position::left,
];
