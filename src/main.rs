mod bot;
mod game;
mod game_coordinator;

use bot::{Bot, AgressoBot, RandoBot};
use game::{Position, Rank, STARTING_RANKS};
use game_coordinator::{GameCoordinator, Outcome};
use rand::{Rng, RngCore, SeedableRng};
use rand_xoshiro::SplitMix64;
use rayon::prelude::*;
use std::time::Instant;
use serde::Deserialize;
use serde_json::{json, Map, Value};

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

#[allow(non_snake_case)]
#[derive(Deserialize)]
struct PartialGameInitJson {
    You: usize,
}

#[allow(non_snake_case)]
#[derive(Deserialize)]
struct PositionJson {
    X: u8,
    Y: u8,
}

#[allow(non_snake_case)]
#[derive(Deserialize)]
struct MoveJson {
    From: PositionJson,
    To: PositionJson,
}

#[allow(non_snake_case)]
#[derive(Deserialize)]
struct AttackerJson {
    Player: usize,
    Rank: String,
}

#[allow(non_snake_case)]
#[derive(Deserialize)]
struct BattleResultJson {
    Attacker: AttackerJson,
    Defender: AttackerJson,
}

#[allow(non_snake_case)]
#[derive(Deserialize)]
struct PartialGameStateJson {
    ActivePlayer: usize,
    TurnNumber: usize,
    LastMove: Option<MoveJson>,
    BattleResult: Option<BattleResultJson>,
}

struct Communicator {
    bot: Box<dyn Bot>,
}

impl Communicator {
    fn new(bot: Box<dyn Bot>) -> Communicator {
        Communicator {
            bot,
        }
    }

    fn run(&mut self) {
        println!("bot-start");

        let mut line_buffer = String::new();
        let stdin = std::io::stdin();

        stdin.read_line(&mut line_buffer).expect("");

        let game_init: PartialGameInitJson = serde_json::from_str(&line_buffer).unwrap();

        let player_id = game_init.You;

        let placement = self.bot.get_initial_placement(&STARTING_RANKS);

        fn pos_to_json(pos: &Position) -> Value {
            json!({ "X": pos.x, "Y": pos.y })
        }

        let setup_pieces = placement.iter().map(|(rank, pos)| {
            json!({
                "Rank": rank.to_str().to_string(),
                "Position": pos_to_json(pos),
            })
        }).collect::<Vec<Value>>(); 

        let setup_board_command = json!({ "Pieces": setup_pieces });

        println!("{}", serde_json::to_string(&setup_board_command).expect(""));

        loop {
            line_buffer.clear();
            stdin.read_line(&mut line_buffer).unwrap();
            let game_state: PartialGameStateJson = serde_json::from_str(&line_buffer).unwrap();

            if let Some(battle_result) = game_state.BattleResult {
                todo!();
            }

            if game_state.ActivePlayer == player_id {
                // Make your move...
            } else {
                // Inform bot of opponents move.
            }
        }
    }
}

fn main() {
    let mut seeder = SplitMix64::seed_from_u64(5498709864);
    let bot = Box::new(AgressoBot::new(seeder.next_u64()));
    let mut communicator = Communicator::new(bot);
    communicator.run();
}

fn main2() {
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
