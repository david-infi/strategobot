use crate::boardbitmap::BoardBitmap;
use crate::game::logic::scout_max_steps_with_stepper;
use crate::game::{Action, Position, Rank};
use crate::json_runner::{BattleResultJson, GameStateJson, TileJson};
use std::fmt;
use std::fmt::Display;
use thiserror::Error;
use tinyvec::ArrayVec;

#[derive(Clone, Copy)]
pub struct Piece {
    pub pos: Position,
    pub rank: Rank,
    pub has_moved: bool,
    pub is_revealed: bool,
}

impl Default for Piece {
    fn default() -> Self {
        Piece {
            pos: Position { x: 0, y: 0 },
            rank: Rank::Unknown,
            has_moved: false,
            is_revealed: false,
        }
    }
}

#[derive(Error, Debug, PartialEq, Eq)]
pub enum ActionError {
    NoFriendlyPieceOnFromPosition,
    FriendlyPieceIsNotMoveable,
    ToPositionOccpuiedByFriend,
    ToPositionIsAnInvalidMapPosition,
    MovementIsNotStraight,
    InvalidMovementDistance,
}

impl Display for ActionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub struct Battle {
    pub ranks: [Rank; 2],
    pub has_died: [bool; 2],
}

impl From<BattleResultJson> for Battle {
    fn from(battle: BattleResultJson) -> Battle {
        let mut ranks = [battle.attacker.rank, battle.defender.rank];
        ranks.as_mut_slice().swap(0, battle.attacker.player);

        let has_died = match battle.winner {
            None => [true, true],
            Some(0) => [false, true],
            Some(1) => [true, false],
            _ => unreachable!(),
        };

        Battle { ranks, has_died }
    }
}

pub struct Turn {
    pub player_id: usize,
    pub action: Action,
    pub battle: Option<Battle>,
}

impl From<GameStateJson> for Turn {
    fn from(state: GameStateJson) -> Turn {
        let Some(last_move) = state.last_move else { panic!() };

        let last_player_id = (state.active_player + 1) % 2;

        Turn {
            player_id: last_player_id,
            action: last_move.into(),
            battle: state.battle_result.map(|x| x.into()),
        }
    }
}

#[derive(Clone, Copy)]
pub struct State {
    pub current_player_id: usize,
    pub turn_count: usize,

    pub pieces: [ArrayVec<[Piece; 8]>; 2],
    pub bitmaps: [BoardBitmap; 2],
}

impl State {
    pub fn new_with_placements(placements: &[&[(Rank, Position)]; 2]) -> State {
        let pieces: [ArrayVec<_>; 2] = placements.map(|ps| {
            ps.iter()
                .map(|&(rank, pos)| Piece {
                    pos,
                    rank,
                    has_moved: false,
                    is_revealed: false,
                })
                .collect()
        });

        let mut bitmaps = [BoardBitmap::new(); 2];

        for (i, pieces) in pieces.iter().enumerate() {
            for Piece { pos, .. } in pieces.iter() {
                bitmaps[i].set(pos.to_bit_index(), true);
            }
        }

        State {
            current_player_id: 0,
            turn_count: 0,
            pieces,
            bitmaps,
        }
    }

    pub fn new_from_json_state(state: &GameStateJson) -> State {
        let mut res = State {
            current_player_id: 0,
            turn_count: 0,
            pieces: [ArrayVec::new(), ArrayVec::new()],
            bitmaps: [BoardBitmap::new(); 2],
        };

        for TileJson {
            rank,
            owner,
            coordinate,
            ..
        } in &state.board
        {
            if let Some(id) = owner {
                let rank = rank
                    .as_ref()
                    .and_then(|s| Rank::try_from(s.as_str()).ok())
                    .unwrap_or(Rank::Unknown);

                res.pieces[*id].push(Piece {
                    pos: (*coordinate).into(),
                    rank,
                    has_moved: false,
                    is_revealed: rank != Rank::Unknown,
                });
            }
        }

        for (i, pieces) in res.pieces.iter().enumerate() {
            for Piece { pos, .. } in pieces.iter() {
                res.bitmaps[i].set(pos.to_bit_index(), true);
            }
        }

        res
    }

    pub fn reversed(&self) -> State {
        State {
            current_player_id: self.current_player_id,
            turn_count: self.turn_count,
            pieces: [1, 0].map(|id| {
                self.pieces[id]
                    .into_iter()
                    .map(|piece @ Piece { pos, .. }| Piece {
                        pos: pos.reversed(),
                        ..piece
                    })
                    .collect()
            }),
            bitmaps: [1, 0].map(|id| self.bitmaps[id].reversed()),
        }
    }

    pub fn update_with_turn(&mut self, turn: &Turn) {
        let id = turn.player_id;

        self.bitmaps[id].set(turn.action.from.to_bit_index(), false);
        self.bitmaps[id].set(turn.action.to.to_bit_index(), true);

        let mut piece = self.pieces[id]
            .iter_mut()
            .find(|Piece { pos, .. }| *pos == turn.action.from)
            .unwrap();

        piece.pos = turn.action.to;
        piece.has_moved = true;

        if let Some(Battle { has_died, .. }) = turn.battle {
            for id in [0, 1] {
                let idx = self.pieces[id]
                    .iter()
                    .position(|Piece { pos, .. }| *pos == turn.action.to)
                    .unwrap();

                self.pieces[id][idx].is_revealed = true;

                self.bitmaps[id].set(turn.action.to.to_bit_index(), !has_died[id]);

                if has_died[id] {
                    self.pieces[id].swap_remove(idx);
                }
            }
        }

        self.turn_count += 1;
        self.current_player_id = (turn.player_id + 1) % 2
    }
}

pub fn validate_action(state: &State, action: &Action) -> Result<(), ActionError> {
    use ActionError::*;

    let friend = state.pieces[state.current_player_id]
        .iter()
        .find(|p| p.pos == action.from);

    let Some(friend) = friend else { return Err(NoFriendlyPieceOnFromPosition); };

    if !friend.rank.is_moveable() {
        return Err(FriendlyPieceIsNotMoveable);
    }

    let is_destination_occupied_by_friend = !state.pieces[state.current_player_id]
        .iter()
        .any(|p| p.pos == action.to);
    if !is_destination_occupied_by_friend {
        return Err(ToPositionOccpuiedByFriend);
    }

    if !action.to.is_valid_map_position() {
        return Err(ToPositionIsAnInvalidMapPosition);
    }

    if !(action.from.x == action.to.x || action.from.y == action.to.y) {
        return Err(MovementIsNotStraight);
    }

    let is_valid_movement_distance = if friend.rank == Rank::Scout {
        action.distance()
            <= scout_max_steps_with_stepper(
                action.direction().to_stepper(),
                &action.from,
                &state.bitmaps[state.current_player_id],
                &state.bitmaps[(state.current_player_id + 1) % 2],
            )
    } else {
        action.distance() == 1
    };
    if !is_valid_movement_distance {
        return Err(InvalidMovementDistance);
    }

    Ok(())
}
