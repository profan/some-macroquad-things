use lockstep_client::{game::GameLobbyContext, step::LockstepClient};
use nanoserde::{DeJson, SerJson};
use puffin_egui::egui;

use crate::{commands::{CommandsExt, GameCommand}, game::{RymdGameParameters, RymdGameTeam}, model::{set_default_energy_pool_size, set_default_metal_pool_size, set_player_team_allegiance, RymdGameModel}, utils::helpers::{create_asteroid_clumps, create_player_commander_ships, create_players, is_any_commander_still_alive_in_team}, PlayerID};

use super::gamemode::{RymdGameMode, RymdGameModeResult};

#[derive(Clone, Debug, SerJson, DeJson)]
pub struct RymdGameModeConquestData {
    pub teams: Vec<RymdGameTeam>,
    pub starting_metal: i32,
    pub starting_energy: i32,
    pub changed: bool
}

impl RymdGameModeConquestData {

    pub fn new() -> RymdGameModeConquestData {
        RymdGameModeConquestData {
            teams: vec![RymdGameTeam::new(0), RymdGameTeam::new(1)],
            starting_metal: 1000,
            starting_energy: 1000,
            changed: false
        }
    }

    pub fn move_player_to_team(&mut self, player_id: PlayerID, target_team_id: i32) {

        for team in &mut self.teams {
            team.players.retain(|&p| p != player_id);
        }

        for team in &mut self.teams {
            if team.id == target_team_id {
                team.players.push(player_id);
                break;
            }
        }

    }

    pub fn remove_player_from_teams(&mut self, player_id: PlayerID) {

        for team in &mut self.teams {
            team.players.retain(|&p| p != player_id);
        }

    }

}

#[derive(Clone)]
pub struct RymdGameModeConquest {
    pub data: RymdGameModeConquestData
}

impl RymdGameModeConquest {

    pub fn new() -> RymdGameModeConquest {
        RymdGameModeConquest {
            data: RymdGameModeConquestData::new()
        }
    }

}

impl RymdGameMode for RymdGameModeConquest {

    fn name(&self) -> &str {
        "Conquest"
    }

    fn on_start(&self, model: &mut RymdGameModel, parameters: &RymdGameParameters) {

        model.random.srand(42);

        let number_of_asteroid_clumps = 10;
        let number_of_asteroids = 10;

        create_players(model, parameters);

        for team in &self.data.teams {
            for &player_id in &team.players {
                let current_team_mask: u64 = 1 << team.id;
                set_player_team_allegiance(&mut model.world, player_id, current_team_mask);
            }
        }

        create_player_commander_ships(model, parameters);
        create_asteroid_clumps(model, number_of_asteroid_clumps, number_of_asteroids);

        set_default_metal_pool_size(&mut model.world, self.data.starting_metal, self.data.starting_metal);
        set_default_energy_pool_size(&mut model.world, self.data.starting_energy, self.data.starting_energy);

    }

    fn on_client_joined_lobby(&mut self, client_id: PlayerID, ctx: &mut GameLobbyContext) {
        
        if ctx.is_player_boss() {
            self.data.move_player_to_team(client_id, self.data.teams.first().unwrap().id);
            self.data.changed = true;
        }

    }

    fn on_client_left_lobby(&mut self, client_id: PlayerID, ctx: &mut GameLobbyContext) {

        if ctx.is_player_boss() {
            self.data.remove_player_from_teams(client_id);
            self.data.changed = true;
        }
        
    }

    fn on_lobby_update(&mut self, new_lobby_data: String) {
        
        if let Ok(rymd_game_mode_conquest_data) = RymdGameModeConquestData::deserialize_json(&new_lobby_data) {
            self.data = rymd_game_mode_conquest_data;
        }

    }

    fn tick(&self, model: &mut RymdGameModel) -> RymdGameModeResult {

        for team in &self.data.teams {
            if is_any_commander_still_alive_in_team(&mut model.world, team) == false {
                // evaporate all the units of this team?
            }
        }

        RymdGameModeResult::Continue
        
    }
    
    fn draw_lobby_ui(&mut self, ui: &mut egui::Ui, ctx: &mut GameLobbyContext) {

        let can_modify_settings_directly = ctx.is_player_boss();
        let mut anything_changed = false;

        ui.vertical_centered(|ui| {

            ui.heading("settings");

            ui.horizontal(|ui| {
                ui.label("starting metal");

                let old_metal_value = self.data.starting_metal;
                let e = ui.add(egui::Slider::new(&mut self.data.starting_metal, 1000..=50000));
                if ctx.is_player_boss() == false {
                    self.data.starting_metal = old_metal_value;
                }

                anything_changed = anything_changed || e.changed();
            });

            ui.horizontal(|ui| {
                ui.label("starting energy");

                let old_energy_value = self.data.starting_energy;
                let e = ui.add(egui::Slider::new(&mut self.data.starting_energy, 1000..=50000));
                if ctx.is_player_boss() == false {
                    self.data.starting_energy = old_energy_value;
                }
                
                anything_changed = anything_changed || e.changed();
            });

            ui.heading("teams");

            for team in &mut self.data.teams.clone() {

                ui.separator();
                ui.heading(format!("team {}", team.id));
                for &player_id in &team.players {
                    ui.label(format!("{} ({})", ctx.get_lobby_client_name(player_id), player_id));
                }

                if team.players.contains(&ctx.lockstep().peer_id()) == false && ui.button("join").clicked() {
                    ctx.lockstep_mut().send_join_team_message(team.id);
                }

            }

        });
        
        self.data.changed = self.data.changed || anything_changed;

    }
    
    fn on_lobby_command(&mut self, client_id: PlayerID, game_command: &GameCommand) {
        
        match game_command {
            GameCommand::Message { text } => (),
            GameCommand::JoinTeam { team_id } => {
                self.data.move_player_to_team(client_id, *team_id);
                self.data.changed = true;
            }
            GameCommand::LeaveTeam => {
                self.data.remove_player_from_teams(client_id);
                self.data.changed = true;
            }
        }

    }
    
    fn handle_lobby_tick(&mut self, ctx: &mut GameLobbyContext) {
        
        if ctx.is_player_boss() && self.data.changed {
            
            self.data.changed = false;
            let conquest_lobby_data = self.data.serialize_json();
            ctx.push_new_lobby_data(conquest_lobby_data);

        }

    }

}