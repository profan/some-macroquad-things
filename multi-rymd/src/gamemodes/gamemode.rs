use lockstep_client::{game::GameLobbyContext, step::LockstepClient};
use puffin_egui::egui;

use crate::{game::RymdGameParameters, model::RymdGameModel, PlayerID};

pub enum RymdGameModeResult {
    Start,
    Continue,
    End
}

pub trait RymdGameMode {

    fn name(&self) -> &str;

    fn on_start(&self, model: &mut RymdGameModel, parameters: &RymdGameParameters);
    fn tick(&self, model: &mut RymdGameModel) -> RymdGameModeResult;

    fn on_client_joined_lobby(&mut self, lockstep: &LockstepClient, client_id: PlayerID);
    fn on_client_left_lobby(&mut self, lockstep: &LockstepClient, client_id: PlayerID);

    fn on_lobby_update(&mut self, new_lobby_data: String);
    
    fn draw_lobby_ui(&mut self, ui: &mut egui::Ui, ctx: &mut GameLobbyContext);

}