mod boardbitmap;
mod bot;
mod game;
mod game_coordinator;
mod json_runner;
mod reservoir_sample;

use bot::{AgressoBot, RandoBot};

use game_coordinator::{GameCoordinator, Outcome};
use json_runner::run_bot;

use rand::{RngCore, SeedableRng};
use rand_xoshiro::SplitMix64;
use std::time::Instant;

fn main() {
    let start_time = Instant::now();

    let mut seeder = SplitMix64::seed_from_u64(54989864);

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

    let round_count = 10_000;

    for _ in 0..round_count {
        let mut game_coordinator = if seeder.next_u64() & 1 == 0 {
            GameCoordinator::new(
                Box::new(RandoBot::new(seeder.next_u64())),
                Box::new(AgressoBot::new(seeder.next_u64())),
                5000,
            )
        } else {
            GameCoordinator::new(
                Box::new(AgressoBot::new(seeder.next_u64())),
                Box::new(RandoBot::new(seeder.next_u64())),
                5000,
            )
        };

        let outcome = game_coordinator.play().expect("");
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
        total_turns as f64 / round_count as f64
    );

    let run_time = start_time.elapsed();

    println!("{} seconds", run_time.as_secs_f32());

    /*
    let mut seeder = SplitMix64::seed_from_u64(154989864);
    let bot = Box::new(AgressoBot::new(seeder.next_u64()));
    run_bot(bot).expect("");
    */
}
