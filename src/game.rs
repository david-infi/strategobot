use std::arch::x86_64::{
    __m128i, _mm_and_si128, _mm_bslli_si128, _mm_cmpeq_epi16, _mm_or_si128, _mm_set1_epi8,
    _mm_set_epi8, _mm_setr_epi8, _mm_setzero_si128, _mm_shuffle_epi8, _mm_slli_epi16,
    _mm_srli_epi16, _mm_test_all_ones, _mm_bsrli_si128, _mm_slli_epi64, _mm_srli_si128,
};
use std::cmp::Ordering;

#[rustfmt::skip]
unsafe fn reverse_byte_order_m128i(x: __m128i) -> __m128i {
    let byte_shuffle_index = _mm_setr_epi8(15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0);
    _mm_shuffle_epi8(x, byte_shuffle_index)
}

#[rustfmt::skip]
unsafe fn m128i_reverse_epi8(x: __m128i) -> __m128i {
    let lut_lo = _mm_setr_epi8(
        0b0000, 0b1000, 0b0100, 0b1100, 
        0b0010, 0b1010, 0b0110, 0b1110, 
        0b0001, 0b1001, 0b0101, 0b1101, 
        0b0011, 0b1011, 0b0111, 0b1111,
    );
    let lut_hi = _mm_slli_epi16(lut_lo, 4);
    let lo_4bit_mask = _mm_set1_epi8(0x0f);

    let hi = _mm_shuffle_epi8(lut_hi, _mm_and_si128(x, lo_4bit_mask));
    let lo = _mm_shuffle_epi8(lut_lo, _mm_and_si128(_mm_srli_epi16(x, 4), lo_4bit_mask));

    _mm_or_si128(hi, lo)
}

#[rustfmt::skip]
unsafe fn m128i_reverse_bits(x: __m128i) -> __m128i {
    m128i_reverse_epi8(reverse_byte_order_m128i(x))
}

unsafe fn m128i_as_slice_u64(x: &__m128i) -> &[u64] {
    unsafe { std::slice::from_raw_parts(x as *const __m128i as *const u64, 2) }
}

unsafe fn m128i_as_slice_u8(x: &__m128i) -> &[u8] {
    unsafe { std::slice::from_raw_parts(x as *const __m128i as *const u8, 16) }
}

unsafe fn m128i_as_mut_slice_u8(x: &mut __m128i) -> &mut [u8] {
    unsafe { std::slice::from_raw_parts_mut(x as *mut __m128i as *mut u8, 16) }
}

#[derive(Clone, Copy, Debug)]
pub struct BoardBitmap {
    data: __m128i,
}

impl PartialEq for BoardBitmap {
    fn eq(&self, other: &Self) -> bool {
        unsafe {
            _mm_test_all_ones(_mm_cmpeq_epi16(self.data, other.data)) == 1
        }
    }
}

impl BoardBitmap {
    const OFFSET: usize = 14;

    pub fn new() -> BoardBitmap {
        unsafe {
            BoardBitmap {
                data: _mm_setzero_si128(),
            }
        }
    }

    pub fn set(&mut self, idx: usize, val: bool) {
        debug_assert!(idx < 100);
        let idx = idx + Self::OFFSET;

        let i = idx / 8;
        let j = idx % 8;
        
        let val_mask = (val as u8) << j;
        let clear_mask = !(1u8 << j);

        let data = unsafe { m128i_as_mut_slice_u8(&mut self.data) };

        data[i] &= clear_mask;
        data[i] |= val_mask;
    }

    pub fn get(&self, idx: usize) -> bool {
        debug_assert!(idx < 100);

        let idx = idx + Self::OFFSET;

        let i = idx / 8;
        let j = idx % 8;
        
        let val_mask = 1u8 << j;

        let data = unsafe { m128i_as_slice_u8(&self.data) };

        data[i] & val_mask != 0
    }

    pub fn reversed(&self) -> BoardBitmap {
        BoardBitmap {
            data: unsafe { m128i_reverse_bits(self.data) }
        }
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
    piece_bitmap: &BoardBitmap,
    enemy_bitmap: &BoardBitmap,
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
        if piece_bitmap.get(to.to_bit_index()) {
            break;
        }

        steps += 1;

        if enemy_bitmap.get(to.to_bit_index()) {
            break;
        }

        to = f(&to);
    }

    steps
}

pub fn has_a_possible_move(
    pieces: &[Piece],
    piece_bitmap: &BoardBitmap,
    enemy_bitmap: &BoardBitmap,
) -> bool {
    for Piece { rank, pos, .. } in pieces {
        if !rank.is_moveable() {
            continue;
        }

        if *rank == Rank::Scout {
            for dir in Direction::ALL_DIRECTIONS {
                let steps = scout_max_steps(pos, &dir, piece_bitmap, enemy_bitmap);
                if steps > 0 {
                    return true;
                }
            }

            continue;
        }

        for neighbour in IntoIterator::into_iter(pos.neighbours()) {
            if !neighbour.is_valid_map_position() || piece_bitmap.get(neighbour.to_bit_index()) {
                continue;
            }

            return true;
        }
    }

    false
}

pub fn all_possible_moves(
    pieces: &[Piece],
    piece_bitmap: &BoardBitmap,
    enemy_bitmap: &BoardBitmap,
    moves: &mut Vec<Action>,
) {
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
            if !neighbour.is_valid_map_position() || piece_bitmap.get(neighbour.to_bit_index()) {
                continue;
            }

            moves.push(Action {
                from: *pos,
                to: neighbour,
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[rustfmt::skip]
    #[test]
    fn test_board_prototype() {
        const OFFSET: usize = (128 - 100) / 2;

        fn set(bitmap: &mut __m128i, idx: usize, val: bool) {
            assert!(idx < 100);

            let idx = idx + OFFSET;

            let i = idx / 8;
            let j = idx % 8;
            
            let val_mask = (val as u8) << j;
            let clear_mask = !(1u8 << j);

            let data = unsafe { m128i_as_mut_slice_u8(bitmap) };

            data[i] &= clear_mask;
            data[i] |= val_mask;
        }

        fn get(bitmap: &__m128i, idx: usize) -> bool {
            assert!(idx < 100);

            let idx = idx + OFFSET;

            let i = idx / 8;
            let j = idx % 8;
            
            let val_mask = 1u8 << j;

            let data = unsafe { m128i_as_slice_u8(bitmap) };

            data[i] & val_mask != 0
        }

        unsafe {
            let mut x = _mm_setzero_si128();

            for i in 0..50 {
                set(&mut x, i * 2, true);
            }

            for i in 0..50 {
                assert!(!get(&x, i * 2 + 1));
                assert!(get(&x, i * 2));
            }

            let x = m128i_reverse_bits(x);

            for i in 0..50 {
                assert!(get(&x, i * 2 + 1));
                assert!(!get(&x, i * 2));
            }
        }
    }

    #[rustfmt::skip]
    #[test]
    fn test_reverse128() {
        unsafe {
            #[allow(overflowing_literals)]
            let a = _mm_set_epi8(
                0x00, 0x11, 0x22, 0x33, 
                0x44, 0x55, 0x66, 0x77, 
                0x88, 0x99, 0xaa, 0xbb, 
                0xcc, 0xdd, 0xee, 0xff,
            );

            let b = m128i_reverse_bits(a);

            #[allow(overflowing_literals)]
            let expected_reverse = _mm_set_epi8(
                0xff, 0x77, 0xbb, 0x33,
                0xdd, 0x55, 0x99, 0x11, 
                0xee, 0x66, 0xaa, 0x22,
                0xCC, 0x44, 0x88, 0x00,
            );

            let c = std::slice::from_raw_parts(&b as *const __m128i as *const u64, 2);
            let d = std::slice::from_raw_parts(&expected_reverse as *const __m128i as *const u64, 2);

            assert_eq!(c, d);


        }
    }

    #[rustfmt::skip]
    #[test]
    fn test_byte_and_bit_order() {
        // This test is more of a sanity check for me. I got confused with how the bytes are
        // ordered, so this is test is an attempt at clearing up some of that confusion.
        unsafe {
            // The byte furthes away from the base address is the first function argument.
            let x = _mm_set_epi8(
                0x18, 0x17, 0x16, 0x15,
                0x14, 0x13, 0x12, 0x11,

                0x08, 0x07, 0x06, 0x05,
                0x04, 0x03, 0x02, 0x01,
            );

            // Interpret the 128-bit m128i value as a slice of 16 bytes.
            let xs = m128i_as_slice_u8(&x);
            
            // As you can see the first byte is `0x01`, which came last in 
            // the _mm_set_epi8 function.
            assert_eq!(xs[..8], [0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08]);
            assert_eq!(xs[8..], [0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18]);

            // Interpret the 128-bit m128i value as a slice of 2 u64s.
            let xs = m128i_as_slice_u64(&x);

            // This can be kind of confusing when you write it like this, because the order of
            // bytes is the opposite when comparing to the byte slice. This is correct.
            // This is a little endian system. The least significant bytes of multi-byte integers
            // have the lower address. Therefore, they come first when interpreting the data as a
            // byte slice, but they are at the end of multi-byte integer literals (like below),
            // since they are the less significant bytes. I understand this, but it can be
            // confusing sometimes.
            assert_eq!(xs[0], 0x08_07_06_05_04_03_02_01);
            assert_eq!(xs[1], 0x18_17_16_15_14_13_12_11);

            {
                // Shift `x` 2 bytes to the left.
                let slx = _mm_bslli_si128(x, 2);
                let xs = m128i_as_slice_u64(&slx);

                // Sanity check that the 2-byte left shift goes in the expected direction.
                assert_eq!(xs[0], 0x06_05_04_03_02_01_00_00);
                assert_eq!(xs[1], 0x16_15_14_13_12_11_08_07);
            }

            {
                // This causes the byte at index i to end up at index (15 - i). So the last byte
                // becomes the first and the first becomes the last.
                let reversed_bytes = reverse_byte_order_m128i(x);
                let xs = m128i_as_slice_u8(&reversed_bytes);

                assert_eq!(xs[..8], [0x18, 0x17, 0x16, 0x15, 0x14, 0x13, 0x12, 0x11]);
                assert_eq!(xs[8..], [0x08, 0x07, 0x06, 0x05, 0x04, 0x03, 0x02, 0x01]);

                // Reverse the bits of each byte of the previous byte reversal. These two
                // operations after each other reverses the bit order of the whole m128i.
                let reversed = m128i_reverse_epi8(reversed_bytes);
                let xs = m128i_as_slice_u8(&reversed);

                assert_eq!(xs[..8], [0x18, 0xe8, 0x68, 0xa8, 0x28, 0xc8, 0x48, 0x88]);
                assert_eq!(xs[8..], [0x10, 0xe0, 0x60, 0xa0, 0x20, 0xc0, 0x40, 0x80]);

                assert_eq!(m128i_as_slice_u8(&reversed), m128i_as_slice_u8(&m128i_reverse_bits(x)));
            }

            {
                #[allow(overflowing_literals)]
                let x = _mm_set_epi8(
                    0x00, 0x00, 0x00, 0x00, 
                    0x00, 0x00, 0x00, 0x00, 
                    0x00, 0x00, 0x00, 0x00, 
                    0x00, 0x00, 0x0f, 0xff, 
                );

                let xs = m128i_as_slice_u8(&x);
                assert_eq!(xs[0], 0xff);

                let x = _mm_bslli_si128(x, 14);
                let xs = m128i_as_slice_u8(&x);
                assert_eq!(xs[14..], [0xff, 0x0f]);

                let x = _mm_slli_epi16(x, 4);
                let xs = m128i_as_slice_u8(&x);
                assert_eq!(xs[15], 0xff);
                assert_eq!(xs[14], 0xf0);
            }
        }
    }
}
