mod bot;
mod game;
mod game_coordinator;

use bot::{AgressoBot, RandoBot};
use game::Rank;
use game_coordinator::{GameCoordinator, Outcome};
use rand::{Rng, RngCore, SeedableRng};
use rand_xoshiro::SplitMix64;
use rayon::prelude::*;
use std::time::Instant;

fn reservoir_sample_one<T, I: Iterator<Item = T>, R: Rng>(rng: &mut R, mut source: I) -> Option<T> {
    let mut chosen = source.next()?;

    let mut i = 2;
    for sample in source {
        let j = rng.gen_range(0..i);
        if j == 0 {
            chosen = sample;
        }
        i += 1;
    }

    Some(chosen)
}

fn reservoir_sample<T, I: Iterator<Item = T>, R: Rng>(
    rng: &mut R,
    mut source: I,
    k: usize,
) -> Vec<T> {
    let mut samples = Vec::new();

    for _ in 0..k {
        if let Some(x) = source.next() {
            samples.push(x);
        } else {
            break;
        }
    }

    let mut i = k + 1;
    for sample in source {
        let j = rng.gen_range(0..i);
        if j < k {
            samples[j] = sample;
        }
        i += 1;
    }

    samples
}

fn main() {
    let starting_ranks = vec![
        Rank::Marshal,
        Rank::General,
        Rank::Miner,
        Rank::Scout,
        Rank::Scout,
        Rank::Spy,
        Rank::Bomb,
        Rank::Flag,
    ];

    let start_time = Instant::now();

    let mut seeder = SplitMix64::seed_from_u64(5498709864);

    /*
    let mut seeds = Vec::new();
    for _ in 0..16 {
        seeds.push(seeder.next_u64());
    }

    let num_games_per_thread = 5_000;
    seeds.par_iter().for_each(|seed| {
        let mut rng = SplitMix64::seed_from_u64(*seed);

        for _ in 0..num_games_per_thread {
            let mut game_coordinator = GameCoordinator::new(
                Box::new(RandoBot::new(rng.next_u64())),
                Box::new(AgressoBot::new(rng.next_u64())),
                &starting_ranks,
                1000,
            );

            game_coordinator.play();
        }
    });
    */

    let mut outcomes: Vec<Outcome> = Vec::new();

    for _ in 0..100_000 {
        let mut game_coordinator = GameCoordinator::new(
            Box::new(AgressoBot::new(seeder.next_u64())),
            Box::new(AgressoBot::new(seeder.next_u64())),
            &starting_ranks,
            100,
        );

        let outcome = game_coordinator.play();
        outcomes.push(outcome);
    }

    let mut timeouts: usize = 0;
    let mut wins = [0, 0];
    let mut total_turns = 0usize;

    for outcome in outcomes.iter() {
        match outcome {
            Outcome::Win { winner, turn_count } => {
                wins[*winner] += 1;
                total_turns += turn_count;
            }
            Outcome::ReachedMaxTurnCount(_) => timeouts += 1,
        }
    }

    println!(
        "[Total games: {}] [Timeouts: {timeouts}] [Wins: {} | {}] [Average turns: {}]",
        outcomes.len(),
        wins[0],
        wins[1],
        total_turns as f64 / 100_000.0
    );

    let run_time = start_time.elapsed();

    println!("{} seconds", run_time.as_secs_f32());
}
