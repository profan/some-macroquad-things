use nanoserde::{DeJson, SerJson};

use crate::gamemodes::{chickens::RymdGameModeChickensData, conquest::RymdGameModeConquestData};

#[derive(Debug, Clone, SerJson, DeJson)]
pub enum LobbyGameState {
    Conquest(RymdGameModeConquestData),
    Chickens(RymdGameModeChickensData)
}