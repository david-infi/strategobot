use crate::json_runner::PositionJson;

#[derive(Debug, PartialEq, Eq, Clone, Copy, PartialOrd, Ord)]
pub struct Position {
    pub x: u8,
    pub y: u8,
}

impl Position {
    pub fn down(&self) -> Position {
        Position {
            x: self.x,
            y: self.y + 1,
        }
    }

    pub fn up(&self) -> Position {
        Position {
            x: self.x,
            y: self.y.overflowing_sub(1).0,
        }
    }

    pub fn left(&self) -> Position {
        Position {
            x: self.x.overflowing_sub(1).0,
            y: self.y,
        }
    }

    pub fn right(&self) -> Position {
        Position {
            x: self.x + 1,
            y: self.y,
        }
    }

    pub fn neighbours(&self) -> [Position; 4] {
        [self.up(), self.right(), self.down(), self.left()]
    }

    pub fn is_valid_map_position(&self) -> bool {
        // Must be inside the board boundaries.
        if self.x > 9 || self.y > 9 {
            return false;
        }

        // Position cannot be inside a lake.
        if ((2..4).contains(&self.x) || (6..8).contains(&self.x)) && (4..6).contains(&self.y) {
            return false;
        }

        true
    }

    pub fn reversed(&self) -> Position {
        Position {
            x: 9 - self.x,
            y: 9 - self.y,
        }
    }

    pub fn to_bit_index(self) -> usize {
        self.x as usize + 10 * self.y as usize
    }

    pub fn manhattan_distance(&self, other: &Position) -> u8 {
        self.x.abs_diff(other.x) + self.y.abs_diff(other.y)
    }
}

impl From<PositionJson> for Position {
    fn from(pos: PositionJson) -> Self {
        Position { x: pos.x, y: pos.y }
    }
}
