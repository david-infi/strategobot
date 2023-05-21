use crate::{
    bot::{Bot, BotOrienter},
    game::{Rank, Battle, Piece, State, Turn},
    game::logic::{battle_casualties, has_a_possible_move},
};

pub struct GameCoordinator {
    bots: [Box<dyn Bot>; 2],
    max_turn_count: usize,
    state: State,
}

#[derive(Debug)]
pub enum Outcome {
    ReachedMaxTurnCount(usize),
    Win { winner: usize, turn_count: usize },
}

impl GameCoordinator {
    pub fn new(p0: Box<dyn Bot>, p1: Box<dyn Bot>, max_turn_count: usize) -> GameCoordinator {
        let mut p0 = Box::new(BotOrienter::new(p0, 0));
        let mut p1 = Box::new(BotOrienter::new(p1, 1));

        let placements = [p0.get_initial_placements(), p1.get_initial_placements()];

        GameCoordinator {
            bots: [p0, p1],
            max_turn_count,
            state: State::new_with_placements(&[&placements[0], &placements[1]]),
        }
    }

    pub fn play(&mut self) -> Outcome {
        while self.state.turn_count < self.max_turn_count {
            let current_player_id = self.state.current_player_id;
            let other_player_id = (current_player_id + 1) % 2;

            // If the current player has no possible moves, then they immediately lose.
            let player_can_move = has_a_possible_move(
                self.state.pieces[current_player_id].as_slice(),
                &self.state.bitmaps[current_player_id],
                &self.state.bitmaps[other_player_id],
            );

            if !player_can_move {
                return Outcome::Win {
                    winner: other_player_id,
                    turn_count: self.state.turn_count,
                };
            }

            let action = {
                // Make a copy of the current state where all the opponents ranks are hidden, unless
                // they have been previously revealed.
                let mut obscured_state = self.state;
                for Piece {
                    rank, is_revealed, ..
                } in obscured_state.pieces[other_player_id].iter_mut()
                {
                    if !*is_revealed {
                        *rank = Rank::Unknown;
                    }
                }

                self.bots[current_player_id].get_action(obscured_state)
            };

            debug_assert_eq!(self.state.find_action_error(&action), None);

            let has_enemy_at_destination =
                self.state.bitmaps[other_player_id].get(action.to.to_bit_index());

            let battle = if has_enemy_at_destination {
                // Find the piece that is on the destination square.
                let Piece {
                    rank: enemy_rank, ..
                } = self.state.pieces[other_player_id]
                    .iter()
                    .find(|Piece { pos, .. }| *pos == action.to)
                    .unwrap();

                // If the enemy piece is the flag, the game is over.
                if *enemy_rank == Rank::Flag {
                    return Outcome::Win {
                        winner: current_player_id,
                        turn_count: self.state.turn_count + 1,
                    };
                }

                // Find the piece we are trying to move to the destination square.
                let Piece {
                    rank: friend_rank, ..
                } = self.state.pieces[current_player_id]
                    .iter()
                    .find(|Piece { pos, .. }| *pos == action.from)
                    .unwrap();

                let (enemy_died, friend_died) = battle_casualties(enemy_rank, friend_rank);

                // The swap is a dirty trick to make sure that the rank of player 0 is at index 0.
                // If the current player is 0 then the swap is a noop of. If the current player is
                // 1 (meaning the order is incorrect), then the elements are swapped.
                let mut ranks = [*friend_rank, *enemy_rank];
                ranks.as_mut_slice().swap(0, current_player_id);

                let mut has_died = [friend_died, enemy_died];
                has_died.as_mut_slice().swap(0, current_player_id);

                Some(Battle { ranks, has_died })
            } else {
                None
            };

            self.state.update_with_turn(&Turn {
                player_id: current_player_id,
                action,
                battle,
            });
        }

        Outcome::ReachedMaxTurnCount(self.max_turn_count)
    }
}
