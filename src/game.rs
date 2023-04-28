use bitvec::{array::BitArray, slice::BitSlice};
use std::arch::x86_64::{
    __m128i, _mm_and_si128, _mm_bslli_si128, _mm_loadu_si128, _mm_or_si128, _mm_set1_epi8,
    _mm_set_epi8, _mm_setr_epi8, _mm_shuffle_epi8, _mm_slli_epi16, _mm_srli_epi16,
    _mm_storeu_si128,
};
use std::cmp::Ordering;

#[derive(Clone, Copy)]
pub struct BoardBitmap {
    store: BitArray<[u64; 2]>,
}

pub type BoardBitSlice = BitSlice<u64>;

fn reverse_128_in_place(x: &mut [u64; 2]) {
    todo!()
}

impl BoardBitmap {
    pub fn new() -> BoardBitmap {
        BoardBitmap {
            store: BitArray::ZERO,
        }
    }

    #[rustfmt::skip]
    pub fn reverse(&mut self) {
        unsafe {
            let byte_shuffle_index = _mm_setr_epi8(15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0);
            let lut_lo = _mm_setr_epi8(
                0b0000, 0b1000, 0b0100, 0b1100,
                0b0010, 0b1010, 0b0110, 0b1110,
                0b0001, 0b1001, 0b0101, 0b1101,
                0b0011, 0b1011, 0b0111, 0b1111,
            );
            let lut_hi = _mm_slli_epi16(lut_lo, 4);

            let lo_4bit_mask = _mm_set1_epi8(15);

            let ptr = self.store.as_raw_mut_slice().as_mut_ptr() as *mut __m128i;

            let acc = dbg!(_mm_loadu_si128(ptr));
            let acc = dbg!(_mm_shuffle_epi8(acc, byte_shuffle_index));

            let hi = _mm_shuffle_epi8(lut_hi, _mm_and_si128(acc, lo_4bit_mask));
            let lo = _mm_shuffle_epi8(lut_lo, _mm_and_si128(_mm_srli_epi16(acc, 4), lo_4bit_mask));

            // This is the full 128 bits in reverse. In our case the board is only 100 bits so
            // we have to shift it left by 28 positions, because the unused 28 bits are now at
            // the start.
            let res = dbg!(_mm_or_si128(hi, lo));

            // We can only shift 28 positions by first shifting 3 bytes (3 * 8 = 24 positions),
            // and then shifting the last 4 positions.
            //let res = dbg!(_mm_srli_epi16(_mm_bsrli_si128(res, 3), 4));
            let res = dbg!(_mm_bslli_si128(res, 3));

            _mm_storeu_si128(ptr, res);
        }
    }

    pub fn as_mut_bitslice(&mut self) -> &mut BitSlice<u64> {
        &mut self.store.as_mut_bitslice()[..100]
    }

    pub fn as_bitslice(&self) -> &BitSlice<u64> {
        &self.store.as_bitslice()[..100]
    }
}
#[derive(PartialEq, Eq, Clone, Copy)]
pub enum Rank {
    Spy,
    Scout,
    Miner,
    Sergeant,
    Lieutenant,
    Captain,
    Major,
    Colonel,
    General,
    Marshal,

    Bomb,
    Flag,
}

impl Rank {
    pub fn is_moveable(&self) -> bool {
        use Rank::*;
        !matches!(&self, Flag | Bomb)
    }
}

pub fn battle_casualties(defender: &Rank, attacker: &Rank) -> (bool, bool) {
    use Rank::*;

    match (*defender, *attacker) {
        (Flag, _) | (_, Bomb | Flag) => unreachable!(),
        (Marshal, Spy) | (Bomb, Miner) => (true, false),
        (Bomb, _) => (false, true),
        (defender, attacker) => {
            let def = defender as u8;
            let atk = attacker as u8;
            (atk >= def, def >= atk)
        }
    }
}

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

    pub fn reverse(&self) -> Position {
        Position {
            x: 9 - self.x,
            y: 9 - self.y,
        }
    }

    pub fn to_bit_index(self) -> usize {
        self.x as usize + 10 * self.y as usize
    }

    pub fn from_bit_index(idx: usize) -> Position {
        Position {
            x: (idx % 10) as u8,
            y: (idx / 10) as u8,
        }
    }
}

#[derive(Clone, Copy)]
pub struct Piece {
    pub rank: Rank,
    pub pos: Position,
    pub is_revealed: bool,
}

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

    pub fn reverse(&self) -> Action {
        Action {
            from: self.from.reverse(),
            to: self.to.reverse(),
        }
    }
}

pub enum Direction {
    Up,
    Right,
    Down,
    Left,
}

impl Direction {
    const ALL_DIRECTIONS: [Direction; 4] = [
        Direction::Up,
        Direction::Right,
        Direction::Down,
        Direction::Left,
    ];
}

pub fn scout_max_steps(
    pos: &Position,
    direction: &Direction,
    piece_bitmap: &BitSlice<u64>,
    enemy_bitmap: &BitSlice<u64>,
) -> usize {
    use Direction::*;

    let f = match direction {
        Up => Position::up,
        Right => Position::right,
        Down => Position::down,
        Left => Position::left,
    };

    let mut steps = 0;
    let mut to = f(pos);

    while to.is_valid_map_position() {
        if piece_bitmap[to.to_bit_index()] {
            break;
        }

        steps += 1;

        if enemy_bitmap[to.to_bit_index()] {
            break;
        }

        to = f(&to);
    }

    steps
}

pub fn has_a_possible_move(
    pieces: &[Piece],
    piece_bitmap: &BitSlice<u64>,
    enemy_bitmap: &BitSlice<u64>,
) -> bool {
    for Piece { rank, pos, .. } in pieces {
        if !rank.is_moveable() {
            continue;
        }

        if *rank == Rank::Scout {
            for dir in Direction::ALL_DIRECTIONS {
                use Direction::*;

                let steps = scout_max_steps(pos, &dir, piece_bitmap, enemy_bitmap);
                if steps > 0 {
                    return true;
                }
            }

            continue;
        }

        for neighbour in IntoIterator::into_iter(pos.neighbours()) {
            if !neighbour.is_valid_map_position() || piece_bitmap[neighbour.to_bit_index()] {
                continue;
            }

            return true;
        }
    }

    false
}

pub fn all_possible_moves(
    pieces: &[Piece],
    piece_bitmap: &BitSlice<u64>,
    enemy_bitmap: &BitSlice<u64>,
) -> Vec<Action> {
    let mut moves = Vec::new();

    for Piece { rank, pos, .. } in pieces {
        if !rank.is_moveable() {
            continue;
        }

        if *rank == Rank::Scout {
            for dir in Direction::ALL_DIRECTIONS {
                use Direction::*;

                let steps = scout_max_steps(pos, &dir, piece_bitmap, enemy_bitmap);

                let f = match dir {
                    Up => Position::up,
                    Right => Position::right,
                    Down => Position::down,
                    Left => Position::left,
                };

                let mut to = *pos;
                for _ in 0..steps {
                    to = f(&to);
                    moves.push(Action { from: *pos, to });
                }
            }

            continue;
        }

        for neighbour in IntoIterator::into_iter(pos.neighbours()) {
            if !neighbour.is_valid_map_position() || piece_bitmap[neighbour.to_bit_index()] {
                continue;
            }

            moves.push(Action {
                from: *pos,
                to: neighbour,
            });
        }
    }

    moves
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn positions() {
        let p = Position { x: 4, y: 9 };

        assert!(p.is_valid_map_position());
        assert_eq!(p, p.reverse().reverse());

        let mut bitmap = BoardBitmap::new();
        let board = bitmap.as_mut_bitslice();

        assert_eq!(board.len(), 100);

        board.set(p.to_bit_index(), true);

        assert_eq!(p.to_bit_index(), board.first_one().unwrap());

        board.reverse();

        assert_eq!(p.reverse().to_bit_index(), board.first_one().unwrap());
    }

    #[test]
    fn bitmap_bit_order() {
        let mut a = BoardBitmap::new();
        for i in 0..50 {
            a.as_mut_bitslice().set(i, true);
        }

        let mut b = BoardBitmap::new();
        let slice = b.store.as_raw_mut_slice();

        slice[0] = 0x0003ffffffffffff;
        slice[1] = 0x0000000000000000;

        assert_eq!(a.store, b.store);
    }

    #[test]
    fn bitmap_reverse() {
        let mut bitmap = BoardBitmap::new();
        let slice = bitmap.store.as_raw_mut_slice();

        slice[1] = 0x3333333333333333;
        slice[0] = 0x0000000333333333;

        let mut a = bitmap;
        a.as_mut_bitslice().reverse();

        let mut b = bitmap;
        b.reverse();

        assert_eq!(a.store, b.store);
    }
}
