use lockstep_client::command::GenericCommand;
use lockstep_client::{game::Game, step::LockstepClient};
use lockstep_client::step::PeerID;
use nanoserde::{DeJson, SerJson};
use puffin_egui::egui;
use utility::{DebugText, TextPosition};

use crate::PlayerID;
use crate::measure_scope;
use crate::model::{GameCommand, GameMessage, RymdGameModel};
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
            Ok(ref message) => match message {
                GameCommand::Message { text } => self.chat.current_messsage_buffer += text,
            },
            Err(err) => {
                println!("[RymdGame] failed to parse generic message: {}!", message);
                return;      
            }
        }
        
    }

    fn handle_game_message(&mut self, peer_id: PeerID, message: &str) {    

        match GameMessage::deserialize_json(message) {
            Ok(ref message) => self.model.handle_message(message),
            Err(err) => {
                println!("[RymdGame] failed to parse game message: {}!", message);
                return;
            }
        };

    }

    #[profiling::function]
    fn update(&mut self, debug: &mut DebugText, lockstep: &mut LockstepClient) {
        measure_scope!(self.stats.update_time_ms);
        self.model.tick();
        self.view.update(&mut self.model);
    }

    #[profiling::function]
    fn draw(&mut self, debug: &mut DebugText, lockstep: &mut LockstepClient, dt: f32) {

        {
            measure_scope!(self.stats.tick_view_time_ms);
            self.view.tick(&mut self.model, lockstep, dt);
        }
        {
            measure_scope!(self.stats.draw_time_ms);
            self.view.draw(&mut self.model, debug, lockstep, dt);
        }

        self.draw_frame_stats(debug);

    }

    fn draw_ui(&mut self, ctx: &egui::Context, debug: &mut DebugText, lockstep: &mut LockstepClient) {

        self.view.draw_ui(ctx, &mut self.model, debug, lockstep);
        
        if crate::INGAME_PROFILER_ENABLED {
            puffin_egui::profiler_window(ctx);
        }

    }

    fn draw_lobby_ui(&mut self, ui: &mut egui::Ui, debug: &mut DebugText, lockstep: &mut LockstepClient) {

        ui.label(&self.chat.current_messsage_buffer);
        ui.text_edit_singleline(&mut self.chat.current_message);

        if ui.button("send message").clicked() && self.chat.current_message.is_empty() == false {
            self.chat.current_message = format!("[peer {}] - {}\n", lockstep.peer_id(), self.chat.current_message);
            let game_command = GameCommand::Message { text: self.chat.current_message.clone() };
            lockstep.send_generic_message(&game_command.serialize_json());
            self.chat.current_message.clear();
        }
        
    }

    fn reset(&mut self) {
        self.stop_game();
    }

    async fn load_resources(&mut self) {
        self.view.load_resources().await;
    }

    fn on_enter_lobby(&mut self) {
        self.chat.reset();
    }

    fn on_leave_lobby(&mut self) {
        self.chat.reset();
    }

    fn on_client_joined_lobby(&mut self, peer_id: PeerID) {
        self.chat.current_messsage_buffer += &format!("[peer {}] joined!\n", peer_id);
    }

    fn on_client_left_lobby(&mut self, peer_id: PeerID) {
        self.chat.current_messsage_buffer += &format!("[peer {}] left!\n", peer_id);
    }

}

impl RymdGame {
    
    pub fn new() -> RymdGame {
        RymdGame {
            chat: RymdGameChat::new(),
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