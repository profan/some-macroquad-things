use lockstep_client::game::GameLobbyContext;
use nanoserde::{DeJson, SerJson};
use puffin_egui::egui;

use crate::{commands::GameCommand, game::RymdGameParameters, lobby::LobbyGameState, model::RymdGameModel, utils::helpers::{create_asteroid_clumps, create_player_commander_ships}, PlayerID};

use super::gamemode::{RymdGameMode, RymdGameModeResult};

#[derive(Clone, Debug, SerJson, DeJson)]
pub struct RymdGameModeChickensData {
    pub number_of_waves: i32,
    pub difficulty_multiplier: f32,
    pub changed: bool
}

impl RymdGameModeChickensData {
    pub fn new() -> RymdGameModeChickensData {
        RymdGameModeChickensData {
            number_of_waves: 3,
            difficulty_multiplier: 1.0,
            changed: false
        }
    }
}

#[derive(Clone)]
pub struct RymdGameModeChickens {
    pub data: RymdGameModeChickensData
}

impl RymdGameModeChickens {
    pub fn new() -> RymdGameModeChickens {
        RymdGameModeChickens {
            data: RymdGameModeChickensData::new()
        }
    }
}

impl RymdGameMode for RymdGameModeChickens {

    fn name(&self) -> &str {
        "Chickens"
    }

    fn on_start(&self, model: &mut RymdGameModel, parameters: &RymdGameParameters) {

        model.random.srand(42);

        let number_of_asteroid_clumps = 10;
        let number_of_asteroids = 10;

        create_player_commander_ships(model, parameters);
        create_asteroid_clumps(model, number_of_asteroid_clumps, number_of_asteroids);

    }

    fn tick(&self, model: &mut RymdGameModel) -> RymdGameModeResult {
        RymdGameModeResult::Continue
    }

    fn on_client_joined_lobby(&mut self, client_id: PlayerID, ctx: &mut GameLobbyContext) {
        
    }

    fn on_client_left_lobby(&mut self, client_id: PlayerID, ctx: &mut GameLobbyContext) {
        
    }

    fn on_lobby_update(&mut self, new_lobby_data: String) {

        if let Ok(rymd_game_mode_chickens_data) = RymdGameModeChickensData::deserialize_json(&new_lobby_data) {
            self.data = rymd_game_mode_chickens_data;
        }
        
    }

    fn draw_lobby_ui(&mut self, ui: &mut egui::Ui, ctx: &mut GameLobbyContext) {
        
        let old_data = self.data.clone();
        let mut any_element_changed = false;

        ui.vertical_centered(|ui| {

            ui.horizontal(|ui| {
                ui.label("number of waves");
                let e = ui.add(egui::Slider::new(&mut self.data.number_of_waves, 5..=30));
                any_element_changed = any_element_changed || e.changed();
            });
    
            ui.horizontal(|ui| {
                ui.label("difficulty multiplier");
                let e = ui.add(egui::Slider::new(&mut self.data.difficulty_multiplier, 0.0..=10.0));
                any_element_changed = any_element_changed || e.changed();
            });

        });

        self.data.changed = self.data.changed || any_element_changed;

        if ctx.is_player_boss() == false {
            self.data = old_data;
        }

    }
    
    fn on_lobby_command(&mut self, client_id: PlayerID, game_command: &GameCommand) {
        
    }
    
    fn handle_lobby_tick(&mut self, ctx: &mut GameLobbyContext) {

        if ctx.is_player_boss() && self.data.changed {
            
            self.data.changed = false;
            
            let chicken_lobby_data = self.data.serialize_json();
            let lobby_game_state = LobbyGameState {
                game_mode_name: self.name().to_owned(),
                game_mode_state: chicken_lobby_data
            };
            
            ctx.push_new_lobby_data(lobby_game_state.serialize_json());

        }

    }

    fn force_lobby_update(&mut self, ctx: &mut GameLobbyContext) {
        self.data.changed = true;
        self.handle_lobby_tick(ctx);
    }

}