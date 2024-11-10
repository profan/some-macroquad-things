use hecs::World;
use lockstep::lobby::Lobby;
use lockstep_client::command::GenericCommand;
use lockstep_client::game::{GameContext, GameLobbyContext};
use lockstep_client::{game::Game, step::LockstepClient};
use lockstep_client::step::PeerID;
use macroquad::math::vec2;
use macroquad::prelude::rand;
use nanoserde::{DeJson, SerJson};
use puffin_egui::egui;
use utility::{DebugText, TextPosition};

use crate::commands::{CommandsExt, GameCommand};
use crate::PlayerID;
use crate::measure_scope;
use crate::model::{build_commander_ship, create_asteroid, create_player_entity, set_default_energy_pool_size, set_default_metal_pool_size, set_player_team_allegiance, spawn_commander_ship, Commander, Controller, GameMessage, Health, Player, RymdGameModel};
use crate::view::RymdGameView;

#[derive(Debug, Clone)]
pub struct RymdGamePlayer {
    pub id: PlayerID
}

#[derive(Debug, Clone)]
pub struct RymdGameParameters {
    pub players: Vec<RymdGamePlayer>
}

impl RymdGameParameters {
    pub fn new() -> RymdGameParameters {
        RymdGameParameters { players: Vec::new() }
    }
}

pub struct RymdGame {
    pub stats: RymdGameFrameStats,
    setup: RymdGameSetup,
    chat: RymdGameChat,
    model: RymdGameModel,
    view: RymdGameView,
    is_started: bool,
    is_running: bool,
    is_paused: bool
}

pub struct RymdGameFrameStats {
    pub main_time_ms: f32,
    pub update_time_ms: f32,
    pub tick_view_time_ms: f32,
    pub draw_time_ms: f32
}

#[derive(Clone, Debug, SerJson, DeJson)]
pub struct RymdGameTeam {
    pub id: i32,
    pub players: Vec<PlayerID>
}

impl RymdGameTeam {
    pub fn new(id: i32) -> RymdGameTeam {
        RymdGameTeam { id, players: Vec::new() }
    }
}

#[derive(Clone, Debug, SerJson, DeJson)]
pub struct RymdGameModeConquestData {
    pub teams: Vec<RymdGameTeam>,
    pub starting_metal: i32,
    pub starting_energy: i32
}

impl RymdGameModeConquestData {
    pub fn new() -> RymdGameModeConquestData {
        RymdGameModeConquestData {
            teams: vec![RymdGameTeam::new(0), RymdGameTeam::new(1)],
            starting_metal: 1000,
            starting_energy: 1000
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

    fn get_number_of_commanders_of_player(world: &mut World, player_id: PlayerID) -> i32 {
        let mut number_of_commanders = 0;
        for (e, (commander, controller)) in world.query_mut::<(&Commander, &Controller)>() {         
            if controller.id == player_id {
                number_of_commanders += 1;
            }
        }
        number_of_commanders
    }

    fn is_commander_dead_for_player(model: &mut RymdGameModel, player_id: PlayerID) -> bool {
        for (e, player) in model.world.query_mut::<&Player>() {
            if player.id == player_id {
                return Self::get_number_of_commanders_of_player(&mut model.world, player_id) <= 0;
            }
        }
        return false;
    }

    fn is_any_commander_still_alive_in_team(model: &mut RymdGameModel, team: &RymdGameTeam) -> bool {
        let mut has_alive_commander = false;
        for &player_id in &team.players {
            if Self::is_commander_dead_for_player(model, player_id) == false {
                has_alive_commander = true;
            }
        }
        has_alive_commander
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

        create_player_commander_ships(model, parameters);
        create_asteroid_clumps(model, number_of_asteroid_clumps, number_of_asteroids);

        set_default_metal_pool_size(&mut model.world, self.data.starting_metal, self.data.starting_metal);
        set_default_energy_pool_size(&mut model.world, self.data.starting_energy, self.data.starting_energy);

        for team in &self.data.teams {
            for &player_id in &team.players {
                let current_team_mask: u64 = 1 << team.id;
                set_player_team_allegiance(&mut model.world, player_id, current_team_mask);
            }
        }

    }

    fn on_lobby_update(&mut self, new_lobby_data: String) {
        
        if let Ok(rymd_game_mode_conquest_data) = RymdGameModeConquestData::deserialize_json(&new_lobby_data) {
            self.data = rymd_game_mode_conquest_data;
        }

    }

    fn tick(&self, model: &mut RymdGameModel) -> RymdGameModeResult {

        for team in &self.data.teams {
            if Self::is_any_commander_still_alive_in_team(model, team) == false {
                // evaporate all the units of this team?
            }
        }

        RymdGameModeResult::Continue
        
    }
    
    fn draw_lobby_ui(&mut self, ui: &mut egui::Ui, ctx: &mut GameLobbyContext) {

        let mut anything_changed = false;

        ui.vertical_centered(|ui| {

            ui.heading("settings");

            ui.horizontal(|ui| {
                ui.label("starting metal");
                let e = ui.add(egui::Slider::new(&mut self.data.starting_metal, 1000..=50000));
                anything_changed = anything_changed || e.changed();
            });

            ui.horizontal(|ui| {
                ui.label("starting energy");
                let e = ui.add(egui::Slider::new(&mut self.data.starting_energy, 1000..=50000));
                anything_changed = anything_changed || e.changed();
            });

            ui.heading("teams");

            for team in &mut self.data.teams.clone() {

                ui.separator();
                ui.heading(format!("team {}", team.id));
                for &player_id in &team.players {
                    ui.label(format!("{} ({})", ctx.get_lobby_client_name(player_id), player_id.to_string()));
                }

                if team.players.contains(&ctx.lockstep().peer_id()) == false && ui.button("join").clicked() {
                    self.data.move_player_to_team(ctx.lockstep().peer_id(), team.id);
                    anything_changed = true;
                }

            }

        });

        if anything_changed {
            let conquest_lobby_data = self.data.serialize_json();
            ctx.push_new_lobby_data(conquest_lobby_data);
        }

    }

}

fn create_asteroid_clumps(model: &mut RymdGameModel, number_of_asteroid_clumps: i32, number_of_asteroids: i32) {

    for i in 0..number_of_asteroid_clumps {

        let asteroid_clump_random_x = model.random.gen_range(-4000, 4000);
        let asteroid_clump_random_y = model.random.gen_range(-4000, 4000);

        for i in 0..number_of_asteroids {

            let random_x = model.random.gen_range(asteroid_clump_random_x - 400, asteroid_clump_random_x + 400);
            let random_y = model.random.gen_range(asteroid_clump_random_y - 400, asteroid_clump_random_y + 400);

            let new_asteroid = create_asteroid(&mut model.world, vec2(random_x as f32, random_y as f32), 0.0);

        }

    }

}

fn create_player_commander_ships(model: &mut RymdGameModel, parameters: &RymdGameParameters) {

    for player in &parameters.players {

        create_player_entity(&mut model.world, player.id);

        let start_random_x = model.random.gen_range(-400, 400);
        let start_random_y = model.random.gen_range(-400, 400);

        let commander_ship = spawn_commander_ship(&mut model.world, player.id, vec2(start_random_x as f32, start_random_y as f32));
    
    }

}

#[derive(Clone, Debug, SerJson, DeJson)]
pub struct RymdGameModeChickensData {
    pub number_of_waves: i32,
    pub difficulty_multiplier: f32
}

impl RymdGameModeChickensData {
    pub fn new() -> RymdGameModeChickensData {
        RymdGameModeChickensData {
            number_of_waves: 3,
            difficulty_multiplier: 1.0
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

    fn on_lobby_update(&mut self, new_lobby_data: String) {

        if let Ok(rymd_game_mode_chickens_data) = RymdGameModeChickensData::deserialize_json(&new_lobby_data) {
            self.data = rymd_game_mode_chickens_data;
        }
        
    }

    fn draw_lobby_ui(&mut self, ui: &mut egui::Ui, ctx: &mut GameLobbyContext) {
        
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

        if any_element_changed {
            let chickens_lobby_data = self.data.serialize_json();
            ctx.push_new_lobby_data(chickens_lobby_data);
        }

    }

}

pub enum RymdGameModeResult {
    Start,
    Continue,
    End
}

pub trait RymdGameMode {

    fn name(&self) -> &str;

    fn on_start(&self, model: &mut RymdGameModel, parameters: &RymdGameParameters);
    fn tick(&self, model: &mut RymdGameModel) -> RymdGameModeResult;

    fn on_lobby_update(&mut self, new_lobby_data: String);
    fn draw_lobby_ui(&mut self, ui: &mut egui::Ui, ctx: &mut GameLobbyContext);

}

pub struct RymdGameSetup {
    game_modes: Vec<Box<dyn RymdGameMode>>,
    game_mode: Option<Box<dyn RymdGameMode>>,
    selected_game_mode: String
}

impl RymdGameSetup {
    pub fn new() -> RymdGameSetup {
        RymdGameSetup {
            game_modes: vec![Box::new(RymdGameModeConquest::new()), Box::new(RymdGameModeChickens::new())],
            game_mode: None,
            selected_game_mode: String::new()
        }
    }

    pub fn set_game_mode(&mut self, game_mode_name: String) {
        let mut found_game_mode_id = None;

        for i in 0..self.game_modes.len() {
            if self.game_modes[i].name() == &game_mode_name {
                found_game_mode_id = Some(i);
                break;
            }
        }

        if let Some(game_mode_id) = found_game_mode_id {

            let last_game_mode = std::mem::replace(&mut self.game_mode, None);
            let new_game_mode = self.game_modes.remove(game_mode_id);

            if let Some(last_game_mode) = last_game_mode {
                self.game_modes.push(last_game_mode);
            }

            self.game_mode = Some(new_game_mode);

        }
    }
}

pub struct RymdGameChat {
    pub current_messsage_buffer: String,
    pub current_message: String,
}

impl RymdGameChat {
    pub fn new() -> RymdGameChat {
        RymdGameChat {
            current_messsage_buffer: String::new(),
            current_message: String::new()
        }
    }

    pub fn on_game_command(&mut self, game_command: &GameCommand) {
        let GameCommand::Message { text } = game_command else { return; };
        self.current_messsage_buffer += text;
    }

    pub fn on_client_joined_lobby(&mut self, peer_id: PeerID) {
        self.current_messsage_buffer += &format!("[peer {}] joined!\n", peer_id);
    }

    pub fn on_client_left_lobby(&mut self, peer_id: PeerID) {
        self.current_messsage_buffer += &format!("[peer {}] left!\n", peer_id);
    }

    pub fn reset(&mut self) {
        self.current_messsage_buffer.clear();
        self.current_message.clear();
    }
}

impl RymdGameFrameStats {
    fn new() -> RymdGameFrameStats {
        RymdGameFrameStats { main_time_ms: 0.0, update_time_ms: 0.0, tick_view_time_ms: 0.0, draw_time_ms: 0.0 }
    }
}

impl Game for RymdGame {

    fn should_automatically_start(&self) -> bool {
        false
    }

    fn is_running(&self) -> bool {
        self.is_running
    }

    fn is_paused(&self) -> bool {
        self.is_paused
    }

    fn start_game(&mut self, lockstep: &LockstepClient) {

        if self.is_started {

            self.is_running = true;
            self.is_paused = false;

        } else {

            let game_parameters = if lockstep.is_singleplayer() {
                let local_game_players = vec![RymdGamePlayer { id: lockstep.peer_id() }, RymdGamePlayer { id: 1 }];
                RymdGameParameters { players: local_game_players }
            } else {
                let game_players = lockstep.peers().iter().map(|client| RymdGamePlayer { id: client.id } ).collect();
                RymdGameParameters { players: game_players }
            };
            
            if let Some(game_mode) = &mut self.setup.game_mode {
                game_mode.on_start(&mut self.model, &game_parameters);
            }

            self.model.start(game_parameters.clone());
            self.view.start(game_parameters.clone(), lockstep.peer_id());
            
            self.is_running = true;
            self.is_started = true;
            self.is_paused = false;

        }

    }

    fn resume_game(&mut self) {
        self.is_paused = false;
    }

    fn pause_game(&mut self) {
        self.is_paused = true;
    }

    fn stop_game(&mut self) {
        self.is_running = false;
        self.is_started = false;
        self.is_paused = false;
        self.model.stop()
    }

    fn handle_generic_message(&mut self, peer_id: PeerID, message: &str) {

        match GameCommand::deserialize_json(message) {
            Ok(ref game_command) => {
                self.chat.on_game_command(game_command);
            },
            Err(err) => {
                println!("[RymdGame] failed to parse generic message: {}!", message);
                return;      
            }
        }
        
    }

    fn handle_game_message(&mut self, peer_id: PeerID, message: &str) {

        // println!("[RymdGame] got message: {} from: {} on tick: {}", message, peer_id, self.model.current_tick);

        match GameMessage::deserialize_json(message) {
            Ok(ref message) => self.model.handle_message(message),
            Err(err) => {
                println!("[RymdGame] failed to parse game message: {}!", message);
                return;
            }
        };

    }

    #[profiling::function]
    fn update(&mut self, ctx: &mut GameContext) {

        if self.is_started == false {
            return;
        }

        measure_scope!(self.stats.update_time_ms);
        self.model.tick();

        if let Some(game_mode) = &mut self.setup.game_mode {
            game_mode.tick(&mut self.model);
        }
        
        self.view.update(&mut self.model);

    }

    #[profiling::function]
    fn draw(&mut self, ctx: &mut GameContext, dt: f32) {

        if self.is_started == false {
            return;
        }

        {
            measure_scope!(self.stats.tick_view_time_ms);
            self.view.tick(&mut self.model, ctx, dt);
        }
        {
            measure_scope!(self.stats.draw_time_ms);
            self.view.draw(&mut self.model, ctx, dt);
            // if let Some(game_mode) = &self.setup.game_mode {
            //     game_mode.draw(&self.model, &mut self.view);
            // }
        }

        self.draw_frame_stats(ctx.debug_text());

    }

    fn draw_ui(&mut self, ui_ctx: &egui::Context, ctx: &mut GameContext) {

        if self.is_started == false {
            return;
        }

        self.view.draw_ui(ui_ctx, &mut self.model, ctx);

        // if let Some(game_mode) = &self.setup.game_mode {
        //     game_mode.draw_ui(&self.model, &mut self.view);
        // }
        
        if crate::INGAME_PROFILER_ENABLED {
            puffin_egui::profiler_window(ui_ctx);
        }

    }

    fn draw_lobby_ui(&mut self, ui: &mut egui::Ui, ctx: &mut GameLobbyContext) {

        // let gamemode_result = egui::ComboBox::from_label("Current Game Mode")
        //     .selected_text(format!("{:?}", self.setup.game_mode.as_ref().and_then(|g| Some(g.name())).unwrap_or("None")))
        //     .show_ui(ui, |ui| {
        //         for game_mode in &mut self.setup.game_modes {
        //             ui.selectable_value(&mut self.setup.selected_game_mode, game_mode.name().to_string(), game_mode.name().to_string());
        //         }
        //     }
        // );

        // self.setup.set_game_mode(self.setup.selected_game_mode.clone());

        if let Some(game_mode) = &mut self.setup.game_mode {
            game_mode.draw_lobby_ui(ui, ctx);
        }

        if ctx.lockstep_mut().is_singleplayer() == false {

            ui.separator();
            ui.label("chat");

            ui.label(&self.chat.current_messsage_buffer);
            ui.text_edit_singleline(&mut self.chat.current_message);
    
            if ui.button("send message").clicked() && self.chat.current_message.is_empty() == false {
                let chat_message_to_send = format!("[peer {}] {}\n", ctx.lockstep_mut().peer_id(), self.chat.current_message);
                ctx.lockstep_mut().send_chat_message(chat_message_to_send.to_string());
                self.chat.current_message.clear();
            }

        }
        
    }

    fn reset(&mut self) {
        self.stop_game();
    }

    async fn load_resources(&mut self) {
        self.view.load_resources().await;
    }

    fn on_enter_lobby(&mut self) {
        self.setup.game_mode = Some(Box::new(RymdGameModeConquest::new()));
        self.chat.reset();
    }

    fn on_leave_lobby(&mut self) {
        self.setup.game_mode = None;
        self.chat.reset();
    }

    fn handle_lobby_update(&mut self, new_lobby_data: String) {
        if let Some(game_mode) = &mut self.setup.game_mode && self.is_started == false {
            game_mode.on_lobby_update(new_lobby_data);
        }
    }

    fn on_client_joined_lobby(&mut self, peer_id: PeerID, lockstep: &mut LockstepClient) {
        self.chat.on_client_joined_lobby(peer_id);
    }

    fn on_client_left_lobby(&mut self, peer_id: PeerID, lockstep: &mut LockstepClient) {
        self.chat.on_client_left_lobby(peer_id);
    }

}

impl RymdGame {
    
    pub fn new() -> RymdGame {
        RymdGame {
            chat: RymdGameChat::new(),
            setup: RymdGameSetup::new(),
            stats: RymdGameFrameStats::new(),
            model: RymdGameModel::new(),
            view: RymdGameView::new(),
            is_running: false,
            is_started: false,
            is_paused: false
        }
    }

    fn draw_frame_stats(&self, debug: &mut DebugText) {
        let fps = 1000.0 / self.stats.main_time_ms;
        debug.draw_text("game update", TextPosition::TopRight, macroquad::color::WHITE);
        debug.draw_text(format!("frame time: {:.2} ms ({:.0} fps)", self.stats.main_time_ms, fps), TextPosition::TopRight, macroquad::color::WHITE);
        debug.draw_text(format!("update tick time: {:.2} ms", self.stats.update_time_ms), TextPosition::TopRight, macroquad::color::WHITE);
        debug.draw_text(format!("tick view time: {:.2} ms", self.stats.tick_view_time_ms), TextPosition::TopRight, macroquad::color::WHITE);
        debug.draw_text(format!("draw time: {:.2} ms", self.stats.draw_time_ms), TextPosition::TopRight, macroquad::color::WHITE);
    }

}