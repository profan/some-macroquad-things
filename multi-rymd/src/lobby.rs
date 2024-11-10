use nanoserde::{DeJson, SerJson};

use crate::game::{RymdGameModeChickensData, RymdGameModeConquestData};

#[derive(Debug, Clone, SerJson, DeJson)]
pub enum LobbyGameState {
    Conquest(RymdGameModeConquestData),
    Chickens(RymdGameModeChickensData)
}