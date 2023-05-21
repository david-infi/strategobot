

use crate::boardbitmap::BoardBitmap;
use crate::game::{Piece, Rank, Direction, Position, Action, ALL_DIRECTIONS};


pub fn battle_casualties(defender: &Rank, attacker: &Rank) -> (bool, bool) {
    use Rank::*;

    match (*defender, *attacker) {
        (Flag, _) | (_, Bomb | Flag) => panic!(),
        (Marshal, Spy) | (Bomb, Miner) => (true, false),
        (Bomb, _) => (false, true),
        (defender, attacker) => {
            let def = defender as u8;
            let atk = attacker as u8;
            (atk >= def, def >= atk)
        }
    }
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
    friends: &[Piece],
    friend_bitmap: &BoardBitmap,
    enemy_bitmap: &BoardBitmap,
) -> bool {
    for Piece { rank, pos, .. } in friends {
        if !rank.is_moveable() {
            continue;
        }

        if *rank == Rank::Scout {
            for dir in ALL_DIRECTIONS {
                let steps = scout_max_steps(pos, &dir, friend_bitmap, enemy_bitmap);
                if steps > 0 {
                    return true;
                }
            }

            continue;
        }

        for neighbour in IntoIterator::into_iter(pos.neighbours()) {
            if !neighbour.is_valid_map_position() || friend_bitmap.get(neighbour.to_bit_index()) {
                continue;
            }

            return true;
        }
    }

    false
}

pub fn all_possible_moves(
    friends: &[Piece],
    friend_bitmap: &BoardBitmap,
    enemy_bitmap: &BoardBitmap,
    actions: &mut Vec<Action>,
) {
    for Piece { rank, pos, .. } in friends {
        if !rank.is_moveable() {
            continue;
        }

        if *rank == Rank::Scout {
            for dir in ALL_DIRECTIONS {
                use Direction::*;

                let steps = scout_max_steps(pos, &dir, friend_bitmap, enemy_bitmap);

                let f = match dir {
                    Up => Position::up,
                    Right => Position::right,
                    Down => Position::down,
                    Left => Position::left,
                };

                let mut to = *pos;
                for _ in 0..steps {
                    to = f(&to);
                    actions.push(Action { from: *pos, to });
                }
            }

            continue;
        }

        for neighbour in IntoIterator::into_iter(pos.neighbours()) {
            if !neighbour.is_valid_map_position() || friend_bitmap.get(neighbour.to_bit_index()) {
                continue;
            }

            actions.push(Action {
                from: *pos,
                to: neighbour,
            });
        }
    }
}
