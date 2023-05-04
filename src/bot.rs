use crate::game::{all_possible_moves, Action, Position, Rank};
use crate::game_coordinator::State;
use crate::{reservoir_sample, reservoir_sample_one};
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

fn random_placement<R: Rng>(rng: &mut R, ranks: &[Rank]) -> Vec<(Rank, Position)> {
    let all_positions = itertools::iproduct!((0..10), (0..4)).map(|(x, y)| Position { x, y });

    let chosen_positions = reservoir_sample(rng, all_positions, ranks.len());

    chosen_positions
        .into_iter()
        .zip(ranks)
        .map(|(pos, rank)| (*rank, pos))
        .collect::<Vec<_>>()
}

fn random_action<R: Rng>(rng: &mut R, state: &State, action_buffer: &mut Vec<Action>) -> Action {
    action_buffer.clear();

    all_possible_moves(
        state.pieces,
        state.piece_bitmap,
        state.enemy_bitmap,
        action_buffer,
    );

    let idx = rng.gen_range(0..action_buffer.len());

    action_buffer[idx]
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
    fn get_initial_placement(&mut self, starting_ranks: &[Rank]) -> Vec<(Rank, Position)> {
        random_placement(&mut self.rng, starting_ranks)
    }

    fn get_action(&mut self, state: &State) -> Action {
        // There is guaranteed to be at least one possible move, because the game coordinator
        // checks for the existence of one before it requests an action.

        random_action(&mut self.rng, state, &mut self.move_buffer)
    }
}

pub struct AgressoBot {
    rng: Xoshiro256StarStar,
    action_buffer: Vec<Action>,
    score_buffer: Vec<u8>,
}

impl AgressoBot {
    pub fn new(seed: u64) -> AgressoBot {
        AgressoBot {
            rng: Xoshiro256StarStar::seed_from_u64(seed),
            action_buffer: Vec::new(),
            score_buffer: Vec::new(),
        }
    }
}

impl Bot for AgressoBot {
    fn get_initial_placement(&mut self, starting_ranks: &[Rank]) -> Vec<(Rank, Position)> {
        random_placement(&mut self.rng, starting_ranks)
    }

    fn get_action(&mut self, state: &State) -> Action {
        self.action_buffer.clear();
        self.score_buffer.clear();

        all_possible_moves(
            state.pieces,
            state.piece_bitmap,
            state.enemy_bitmap,
            &mut self.action_buffer,
        );

        let action_scores = self.action_buffer.iter().map(|action| {
            // If this action results in attacking an enemy, then this score is zero.
            let score: u8 = state
                .enemies
                .iter()
                .map(|(p, _)| p.manhattan_distance(&action.to) as u8)
                .min()
                .unwrap();
            score
        });

        self.score_buffer.extend(action_scores);

        let best_score = self.score_buffer.iter().min().unwrap();

        let best_scoring_actions = self
            .action_buffer
            .iter()
            .zip(&self.score_buffer)
            .filter_map(|(a, s)| if s == best_score { Some(a) } else { None });

        *reservoir_sample_one(&mut self.rng, best_scoring_actions).unwrap()
    }
}
