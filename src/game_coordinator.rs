use crate::{
    bot::Bot,
    game::{
        battle_casualties, has_a_possible_move, scout_max_steps, Action, BoardBitmap, Piece,
        Position, Rank,
    },
};

pub struct State<'a> {
    pub starting_ranks: &'a [Rank],
    pub pieces: &'a [Piece],
    pub enemies: &'a [(Position, Option<Rank>)],
    pub piece_bitmap: &'a BoardBitmap,
    pub enemy_bitmap: &'a BoardBitmap,
}

pub struct GameCoordinator {
    bots: [Box<dyn Bot>; 2],
    pieces: [Vec<Piece>; 2],
    bitmaps: [BoardBitmap; 2],

    starting_ranks: Vec<Rank>,

    turn_count: usize,
    current_player: usize,
    max_turn_count: usize,
}

#[derive(Debug)]
pub enum Outcome {
    ReachedMaxTurnCount(usize),
    Win { winner: usize, turn_count: usize },
}

impl GameCoordinator {
    pub fn new(
        mut p0: Box<dyn Bot>,
        mut p1: Box<dyn Bot>,
        starting_ranks: &[Rank],
        max_turn_count: usize,
    ) -> GameCoordinator {
        let placements = [
            p0.get_initial_placement(starting_ranks),
            p1.get_initial_placement(starting_ranks),
        ];

        // From the bot's perspective they are both playing as red. So for both of them we need to
        // reverse the positions of their pieces before we provide them to the opponent.
        let starting_positions = placements.clone().map(|placements| {
            placements
                .iter()
                .map(|(_, pos)| pos.reverse())
                .collect::<Vec<_>>()
        });

        p0.notify_opponents_initial_placement(starting_ranks, &starting_positions[1]);
        p1.notify_opponents_initial_placement(starting_ranks, &starting_positions[0]);

        let bitmaps = placements.clone().map(|placements| {
            let mut bitmap = BoardBitmap::new();

            let indices = placements.iter().map(|(_, pos)| pos.to_bit_index());

            for idx in indices {
                bitmap.set(idx, true);
            }

            bitmap
        });

        let pieces = placements.map(|placements| {
            placements
                .into_iter()
                .map(|(rank, pos)| Piece {
                    rank,
                    pos,
                    is_revealed: false,
                })
                .collect()
        });

        GameCoordinator {
            bots: [p0, p1],
            pieces,
            bitmaps,
            starting_ranks: starting_ranks.to_vec(),
            turn_count: 0,
            current_player: 0,
            max_turn_count,
        }
    }

    pub fn play(&mut self) -> Outcome {
        let mut enemies: Vec<(Position, Option<Rank>)> = Vec::new();
        enemies.reserve(self.starting_ranks.len());

        while self.turn_count < self.max_turn_count {
            let current_player = self.current_player;
            let other_player = (current_player + 1) % 2;

            enemies.clear();
            enemies.extend(
                self.pieces[other_player]
                    .iter()
                    .map(|piece| (piece.pos.reverse(), piece.is_revealed.then_some(piece.rank))),
            );

            let enemy_bitmap = self.bitmaps[other_player].reversed();

            let state = State {
                starting_ranks: &self.starting_ranks,
                pieces: &self.pieces[current_player],
                enemies: &enemies,
                piece_bitmap: &self.bitmaps[current_player],
                enemy_bitmap: &enemy_bitmap,
            };

            // Make sure the bitmap positions and the positions of the pieces are in sync
            /*
            if cfg!(debug_assertions) {
                let mut bitmap_positions = self.bitmaps[current_player]
                    .as_bitslice()
                    .iter_ones()
                    .map(|idx| Position::from_bit_index(idx))
                    .collect::<Vec<_>>();
                bitmap_positions.sort();

                let mut piece_positions = self.pieces[current_player]
                    .clone()
                    .into_iter()
                    .map(|piece| piece.pos)
                    .collect::<Vec<_>>();
                piece_positions.sort();

                assert_eq!(bitmap_positions, piece_positions);

                let mut bitmap_positions = self.bitmaps[other_player]
                    .as_bitslice()
                    .iter_ones()
                    .map(|idx| Position::from_bit_index(idx))
                    .collect::<Vec<_>>();
                bitmap_positions.sort();

                let mut piece_positions = self.pieces[other_player]
                    .clone()
                    .into_iter()
                    .map(|piece| piece.pos)
                    .collect::<Vec<_>>();
                piece_positions.sort();

                assert_eq!(bitmap_positions, piece_positions);
            }
            */

            let player_can_move = has_a_possible_move(
                &self.pieces[current_player],
                &self.bitmaps[current_player],
                &self.bitmaps[other_player],
            );

            if !player_can_move {
                return Outcome::Win {
                    winner: other_player,
                    turn_count: self.turn_count,
                };
            }

            let action = self.bots[current_player].get_action(&state);

            if cfg!(debug_assertions) {
                self.assert_is_valid_action(&action);
            }

            let piece_idx = self.pieces[current_player]
                .iter()
                .position(|piece| piece.pos == action.from)
                .unwrap();

            // If the destination square has an enemy on it, we have to resolve the battle.
            let piece_died = if enemy_bitmap.get(action.to.to_bit_index()) {
                let enemy_idx = self.pieces[other_player]
                    .iter()
                    .position(|piece| piece.pos.reverse() == action.to)
                    .unwrap();

                // Find the piece that is on the destination square.
                let Piece {
                    rank: enemy_rank, ..
                } = self.pieces[other_player][enemy_idx];

                // If the enemy piece is the flag, the game is over.
                if enemy_rank == Rank::Flag {
                    return Outcome::Win {
                        winner: current_player,
                        turn_count: self.turn_count,
                    };
                }

                // Find the piece we are trying to move to the destination square.
                let Piece { rank: own_rank, .. } = self.pieces[current_player][piece_idx];

                let (piece_dies, enemy_dies) = battle_casualties(&enemy_rank, &own_rank);

                // Regardless, of whether the piece dies or not, it is now known to the other
                // player.
                self.pieces[current_player][piece_idx].is_revealed = true;
                self.pieces[other_player][enemy_idx].is_revealed = true;

                if piece_dies {
                    self.pieces[current_player].swap_remove(piece_idx);
                    // Updating the bitmaps and position of this piece happens later, because this
                    // always needs to happen even if there is no battle.
                }

                if enemy_dies {
                    self.bitmaps[other_player].set(action.to.reverse().to_bit_index(), false);
                    self.pieces[other_player].swap_remove(enemy_idx);
                }

                piece_dies
            } else {
                false
            };

            // Update the position of the piece we are moving in the `self.pieces` Vec, and the
            // bitmap.

            // Whether the piece died or not, it will no longer be in the same place.
            self.bitmaps[current_player].set(action.from.to_bit_index(), false);

            if !piece_died {
                self.pieces[current_player][piece_idx].pos = action.to;
                self.bitmaps[current_player].set(action.to.to_bit_index(), true);
            }

            // Notify the enemy bot of the move.
            enemies.clear();
            enemies.extend(
                self.pieces[current_player]
                    .iter()
                    .map(|piece| (piece.pos.reverse(), piece.is_revealed.then_some(piece.rank))),
            );

            let enemy_bitmap = self.bitmaps[current_player].reversed();

            let state = State {
                starting_ranks: &self.starting_ranks,
                pieces: &self.pieces[other_player],
                enemies: &enemies,
                piece_bitmap: &self.bitmaps[other_player],
                enemy_bitmap: &enemy_bitmap,
            };

            self.bots[other_player].notify_opponents_action(&state, action.reverse());

            self.current_player = other_player;
            self.turn_count += 1;
        }

        Outcome::ReachedMaxTurnCount(self.max_turn_count)
    }

    fn assert_is_valid_action(&self, action: &Action) {
        let piece = self.pieces[self.current_player]
            .iter()
            .find(|p| p.pos == action.from);

        assert!(piece.is_some());

        let Some(piece) = piece else { unreachable!() };

        let is_piece_moveable = piece.rank.is_moveable();
        let is_destination_free_from_allies = !self.pieces[self.current_player]
            .iter()
            .any(|p| p.pos == action.to);
        let is_destination_a_valid_map_position = action.to.is_valid_map_position();
        let is_valid_piece_movement = if piece.rank == Rank::Scout {
            let bitmap = self.bitmaps[(self.current_player + 1) % 2].reversed();

            action.distance()
                <= scout_max_steps(
                    &action.from,
                    &action.direction(),
                    &self.bitmaps[self.current_player],
                    &bitmap,
                )
        } else {
            action.distance() == 1
        };

        assert!(is_piece_moveable);
        assert!(is_destination_free_from_allies);
        assert!(is_destination_a_valid_map_position);
        assert!(is_valid_piece_movement);
    }
}
