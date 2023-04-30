mod bot;
mod game;
mod game_coordinator;

use bot::RandoBot;
use game::Rank;
use game_coordinator::GameCoordinator;
use rand::{Rng, RngCore, SeedableRng};
use rand_xoshiro::SplitMix64;
use std::time::Instant;

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

    for _ in 0..1_000 {
        let mut game_coordinator = GameCoordinator::new(
            Box::new(RandoBot::new(seeder.next_u64())),
            Box::new(RandoBot::new(seeder.next_u64())),
            &starting_ranks,
            100_000,
        );

        game_coordinator.play();
    }

    let run_time = start_time.elapsed();

    println!("{} seconds", run_time.as_secs_f32());
}
