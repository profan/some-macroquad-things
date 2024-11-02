use lockstep_client::step::LockstepClient;
use nanoserde::{DeJson, SerJson};

use crate::game::{RymdGameModeChickensData, RymdGameModeConquestData};

#[derive(Debug, SerJson, DeJson)]
pub enum GameCommand {
    UpdateChickenGameLobby { data: RymdGameModeChickensData },
    UpdateConquestGameLobby { data: RymdGameModeConquestData },
    Message { text: String }
}

pub trait CommandsExt {
    fn send_chickens_lobby_data(&mut self, data: &RymdGameModeChickensData);
    fn send_conquest_lobby_data(&mut self, data: &RymdGameModeConquestData);
    fn send_chat_message(&mut self, message: String);
}

impl CommandsExt for LockstepClient {

    fn send_chickens_lobby_data(&mut self, data: &RymdGameModeChickensData) {
        let game_command = GameCommand::UpdateChickenGameLobby { data: data.clone() };
        self.send_generic_message(&game_command.serialize_json());
    }

    fn send_conquest_lobby_data(&mut self, data: &RymdGameModeConquestData) {
        let game_command = GameCommand::UpdateConquestGameLobby { data: data.clone() };
        self.send_generic_message(&game_command.serialize_json());
    }

    fn send_chat_message(&mut self, message: String) {
        let game_command = GameCommand::Message { text: message };
        self.send_generic_message(&game_command.serialize_json());
    }
    
}