use nanoserde::{DeJson, SerJson};

#[derive(Debug, Clone, SerJson, DeJson)]
pub struct LobbyGameState {
    pub game_mode_name: String,
    pub game_mode_state: String
}