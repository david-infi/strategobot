use strategobot::{
    bot::RandoBot,
    game_coordinator::{GameCoordinator, Outcome},
    json_runner::run_bot,
};

use rand::{RngCore, SeedableRng};
use rand_xoshiro::SplitMix64;
use std::time::Instant;

fn main() {
    let mut seeder = SplitMix64::seed_from_u64(123456789);
    let bot = Box::new(RandoBot::new(seeder.next_u64()));
    run_bot(bot).expect("Communication should work");
}

fn _game_runner_test() {
    let start_time = Instant::now();

    let mut seeder = SplitMix64::seed_from_u64(54989864);

    let mut outcomes: Vec<Outcome> = Vec::new();

    let round_count = 100_000;

    for _ in 0..round_count {
        let mut game_coordinator = GameCoordinator::new(
            Box::new(RandoBot::new(seeder.next_u64())),
            Box::new(RandoBot::new(seeder.next_u64())),
            5000,
        );

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
}
