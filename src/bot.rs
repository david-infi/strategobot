use crate::game::{all_possible_moves, Action, Position, Rank};
use crate::game_coordinator::State;
use crate::reservoir_sample;
use rand::{Rng, SeedableRng};
use rand_xoshiro::Xoshiro256StarStar;

pub trait Bot {
    fn get_initial_placement(&mut self, starting_ranks: &[Rank]) -> Vec<(Rank, Position)>;
    fn get_action(&mut self, state: &State) -> Action;

    fn notify_opponents_initial_placement(
        &mut self,
        _starting_ranks: &[Rank],
        _enemies: &[Position],
    ) {
    }

    fn notify_opponents_action(&mut self, _state_after_action: &State, _action: Action) {}
}

pub struct RandoBot {
    rng: Xoshiro256StarStar,
    move_buffer: Vec<Action>,
}

impl RandoBot {
    pub fn new(seed: u64) -> RandoBot {
        RandoBot {
            rng: Xoshiro256StarStar::seed_from_u64(seed),
            move_buffer: Vec::new(),
        }
    }
}

impl Bot for RandoBot {
    fn get_initial_placement(&mut self, ranks: &[Rank]) -> Vec<(Rank, Position)> {
        let all_positions = itertools::iproduct!((0..10), (0..4)).map(|(x, y)| Position { x, y });

        let chosen_positions = reservoir_sample(&mut self.rng, all_positions, ranks.len());

        chosen_positions
            .into_iter()
            .zip(ranks)
            .map(|(pos, rank)| (*rank, pos))
            .collect::<Vec<_>>()
    }

    fn get_action(&mut self, state: &State) -> Action {
        // There is guaranteed at least one move, because the game coordinator checks for this
        // before it requests a move.
        self.move_buffer.clear();
        all_possible_moves(
            state.pieces,
            state.piece_bitmap,
            state.enemy_bitmap,
            &mut self.move_buffer,
        );
        let idx = self.rng.gen_range(0..self.move_buffer.len());
        self.move_buffer[idx]
    }
}
