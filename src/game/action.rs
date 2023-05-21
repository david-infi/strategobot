use crate::game::{Direction, Position};
use crate::json_runner::MoveCommandJson;
use std::cmp::Ordering;

#[derive(Clone, Copy)]
pub struct Action {
    pub from: Position,
    pub to: Position,
}

impl Action {
    pub fn direction(&self) -> Direction {
        match self.from.x.cmp(&self.to.x) {
            Ordering::Greater => Direction::Left,
            Ordering::Equal if self.from.y < self.to.y => Direction::Down,
            Ordering::Equal => Direction::Up,
            Ordering::Less => Direction::Right,
        }
    }

    pub fn distance(&self) -> usize {
        self.from.x.abs_diff(self.to.x) as usize + self.from.y.abs_diff(self.to.y) as usize
    }

    pub fn reversed(&self) -> Action {
        Action {
            from: self.from.reversed(),
            to: self.to.reversed(),
        }
    }
}

impl From<MoveCommandJson> for Action {
    fn from(mov: MoveCommandJson) -> Self {
        Action {
            from: mov.from.into(),
            to: mov.to.into(),
        }
    }
}
