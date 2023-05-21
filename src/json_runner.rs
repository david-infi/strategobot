use anyhow::Result;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::io::Write;

use crate::bot::{Bot, BotOrienter};
use crate::game::{Action, Position, Rank, State, Turn};

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct PartialGameInitJson {
    pub you: usize,
}

#[derive(Serialize, Deserialize, Copy, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct PositionJson {
    pub x: u8,
    pub y: u8,
}

impl From<Position> for PositionJson {
    fn from(pos: Position) -> Self {
        Self { x: pos.x, y: pos.y }
    }
}

#[derive(Serialize, Deserialize, Clone, Copy)]
#[serde(rename_all = "PascalCase")]
pub struct MoveCommandJson {
    pub from: PositionJson,
    pub to: PositionJson,
}

impl From<Action> for MoveCommandJson {
    fn from(action: Action) -> Self {
        MoveCommandJson {
            from: action.from.into(),
            to: action.to.into(),
        }
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct AttackerJson {
    pub player: usize,
    pub rank: Rank,
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct BattleResultJson {
    pub winner: Option<usize>,
    pub attacker: AttackerJson,
    pub defender: AttackerJson,
    pub position: PositionJson,
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct TileJson {
    pub rank: Option<Rank>,
    pub owner: Option<usize>,
    pub is_water: bool,
    pub coordinate: PositionJson,
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct GameStateJson {
    pub active_player: usize,
    pub turn_number: usize,
    pub board: Vec<TileJson>,
    pub last_move: Option<MoveCommandJson>,
    pub battle_result: Option<BattleResultJson>,
}

#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct PlacementJson {
    pub rank: Rank,
    pub position: PositionJson,
}

#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct SetupBoardCommandJson {
    pub pieces: Vec<PlacementJson>,
}

impl From<Vec<(Rank, Position)>> for SetupBoardCommandJson {
    fn from(placement: Vec<(Rank, Position)>) -> Self {
        Self {
            pieces: placement
                .into_iter()
                .map(|(rank, pos)| PlacementJson {
                    rank,
                    position: pos.into(),
                })
                .collect(),
        }
    }
}

fn write_json<T: Serialize>(obj: T) -> Result<()> {
    let mut stdout = std::io::stdout();
    serde_json::to_writer(&mut stdout, &obj)?;
    writeln!(&mut stdout)?;
    Ok(())
}

fn read_json<T: DeserializeOwned>() -> Result<T> {
    let stdin = std::io::stdin();
    let mut line_buffer = String::new();

    stdin.read_line(&mut line_buffer)?;

    let json: T = serde_json::from_reader(line_buffer.as_bytes())?;

    Ok(json)
}

pub fn run_bot(bot: Box<dyn Bot>) -> Result<()> {
    println!("bot-start");

    let PartialGameInitJson { you: player_id } = read_json()?;

    let mut bot = BotOrienter::new(bot, player_id);

    write_json(SetupBoardCommandJson::from(bot.get_initial_placements()))?;

    let mut state = State::new_from_json_state(&read_json()?);

    loop {
        if state.current_player_id == player_id {
            write_json(MoveCommandJson::from(bot.get_action(state)))?;
        } else {
            state.update_with_turn(&read_json::<GameStateJson>()?.into());
        }

        state.update_with_turn(&read_json::<GameStateJson>()?.into());
    }
}
