use lockstep_client::game::{GameContext, GameLobbyContext};
use lockstep_client::{game::Game, step::LockstepClient};
use lockstep_client::step::PeerID;
use nanoserde::{DeJson, SerJson};
use puffin_egui::egui;
use utility::{DebugText, TextPosition};

use crate::commands::{CommandsExt, GameCommand};
use crate::gamemodes::chickens::RymdGameModeChickens;
use crate::gamemodes::conquest::RymdGameModeConquest;
use crate::gamemodes::gamemode::RymdGameMode;
use crate::PlayerID;
use crate::measure_scope;
use crate::model::{GameMessage, RymdGameModel};
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
            if self.game_modes[i].name() == game_mode_name {
                found_game_mode_id = Some(i);
                break;
            }
        }

        if let Some(game_mode_id) = found_game_mode_id {

            let last_game_mode = self.game_mode.take();
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
        let GameCommand::Message { text } = game_command;
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
            }
        }
        
    }

    fn handle_game_message(&mut self, peer_id: PeerID, message: &str) {

        // println!("[RymdGame] got message: {} from: {} on tick: {}", message, peer_id, self.model.current_tick);

        match GameMessage::deserialize_json(message) {
            Ok(ref message) => self.model.handle_message(message),
            Err(err) => {
                println!("[RymdGame] failed to parse game message: {}!", message);
            }
        }

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

        if let Some(game_mode) = &mut self.setup.game_mode {
            game_mode.on_client_joined_lobby(lockstep, peer_id);
        }

    }

    fn on_client_left_lobby(&mut self, peer_id: PeerID, lockstep: &mut LockstepClient) {

        self.chat.on_client_left_lobby(peer_id);

        if let Some(game_mode) = &mut self.setup.game_mode {
            game_mode.on_client_left_lobby(lockstep, peer_id);
        }

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