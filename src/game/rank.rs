use serde::{Deserialize, Serialize};

#[derive(PartialEq, Eq, Clone, Copy, Deserialize, Serialize)]
pub enum Rank {
    Spy,
    Scout,
    Miner,
    Sergeant,
    Lieutenant,
    Captain,
    Major,
    Colonel,
    General,
    Marshal,

    Bomb,
    Flag,

    Unknown, // Not very idiomatic, but it simplifies some other stuff
}

pub const STARTING_RANKS: [Rank; 8] = [
    Rank::Spy,
    Rank::Scout,
    Rank::Scout,
    Rank::Miner,
    Rank::General,
    Rank::Marshal,
    Rank::Bomb,
    Rank::Flag,
];

impl Rank {
    pub fn is_moveable(&self) -> bool {
        !matches!(&self, Rank::Flag | Rank::Bomb)
    }

    pub fn from_str(s: &str) -> Option<Rank> {
        let rank = match s {
            "Spy" => Rank::Spy,
            "Scout" => Rank::Scout,
            "Miner" => Rank::Miner,
            "General" => Rank::General,
            "Marshal" => Rank::Marshal,
            "Bomb" => Rank::Bomb,
            "Flag" => Rank::Flag,
            _ => return None,
        };

        Some(rank)
    }

    pub fn to_str(self) -> &'static str {
        use Rank::*;
        match &self {
            Spy => "Spy",
            Scout => "Scout",
            Miner => "Miner",
            General => "General",
            Marshal => "Marshal",
            Bomb => "Bomb",
            Flag => "Flag",
            _ => unreachable!(),
        }
    }
}

impl TryFrom<&str> for Rank {
    type Error = &'static str;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        Rank::from_str(s).ok_or("String is not a Rank")
    }
}
