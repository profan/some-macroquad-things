use lockstep_client::game::GameLobbyContext;
use puffin_egui::egui;

use crate::{commands::GameCommand, game::RymdGameParameters, model::RymdGameModel, PlayerID};

#[derive(PartialEq)]
pub enum RymdGameModeResult {
    Start,
    Continue,
    End
}

pub trait RymdGameMode {

    fn name(&self) -> &str;

    fn on_start(&self, model: &mut RymdGameModel, parameters: &RymdGameParameters);
    fn tick(&self, model: &mut RymdGameModel) -> RymdGameModeResult;

    fn on_client_joined_lobby(&mut self, client_id: PlayerID, ctx: &mut GameLobbyContext);
    fn on_client_left_lobby(&mut self, client_id: PlayerID, ctx: &mut GameLobbyContext);

    fn on_lobby_command(&mut self, client_id: PlayerID, game_command: &GameCommand);
    fn on_lobby_update(&mut self, new_lobby_data: String);
    
    fn handle_lobby_tick(&mut self, ctx: &mut GameLobbyContext);
    fn draw_lobby_ui(&mut self, ui: &mut egui::Ui, ctx: &mut GameLobbyContext);

}