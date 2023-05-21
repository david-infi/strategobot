
use crate::game::{
    logic::{all_possible_moves}, Piece, State, Action, Position, Rank, STARTING_RANKS};
use crate::reservoir_sample::{reservoir_sample, reservoir_sample_one};

use rand::{Rng, SeedableRng};
use rand_xoshiro::Xoshiro256StarStar;

pub struct BotOrienter {
    player_id: usize,
    bot: Box<dyn Bot>,
}

impl BotOrienter {
    pub fn new(bot: Box<dyn Bot>, player_id: usize) -> Self {
        BotOrienter { player_id, bot }
    }
}

impl Bot for BotOrienter {
    fn get_initial_placements(&mut self) -> Vec<(Rank, Position)> {
        let placements = self.bot.get_initial_placements();

        if self.player_id == 1 {
            return placements
                .into_iter()
                .map(|(rank, pos)| (rank, pos.reversed()))
                .collect();
        }

        placements
    }

    fn get_action(&mut self, state: State) -> Action {
        let state = if self.player_id == 1 {
            state.reversed()
        } else {
            state
        };

        let action = self.bot.get_action(state);

        if self.player_id == 1 {
            action.reversed()
        } else {
            action
        }
    }
}

pub trait Bot {
    fn get_initial_placements(&mut self) -> Vec<(Rank, Position)>;
    fn get_action(&mut self, state: State) -> Action;
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

fn random_action<R: Rng>(rng: &mut R, state: State, action_buffer: &mut Vec<Action>) -> Action {
    action_buffer.clear();

    all_possible_moves(
        &state.pieces[0],
        &state.bitmaps[0],
        &state.bitmaps[1],
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
    fn get_initial_placements(&mut self) -> Vec<(Rank, Position)> {
        random_placement(&mut self.rng, &STARTING_RANKS)
    }

    fn get_action(&mut self, state: State) -> Action {
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
    fn get_initial_placements(&mut self) -> Vec<(Rank, Position)> {
        random_placement(&mut self.rng, &STARTING_RANKS)
    }

    fn get_action(&mut self, state: State) -> Action {
        self.action_buffer.clear();
        self.score_buffer.clear();

        all_possible_moves(
            &state.pieces[0],
            &state.bitmaps[0],
            &state.bitmaps[1],
            &mut self.action_buffer,
        );

        let action_scores = self.action_buffer.iter().map(|action| {
            // If this action results in attacking an enemy, then this score is zero.
            let score: u8 = state.pieces[1]
                .iter()
                .map(|Piece { pos, .. }| pos.manhattan_distance(&action.to) as u8)
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
