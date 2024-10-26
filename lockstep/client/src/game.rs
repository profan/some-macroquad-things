use egui_macroquad::egui;
use utility::DebugText;
use crate::step::{LockstepClient, PeerID};

pub trait Game where Self: Sized {

    async fn load_resources(&mut self);

    fn should_automatically_start(&self) -> bool {
        true
    }

    fn is_running(&self) -> bool;
    fn is_paused(&self) -> bool;

    fn start_game(&mut self, lockstep: &LockstepClient);
    fn stop_game(&mut self);

    fn resume_game(&mut self);
    fn pause_game(&mut self);

    // game
    fn handle_game_message(&mut self, peer_id: PeerID, message: &str);
    fn update(&mut self, debug: &mut DebugText, lockstep: &mut LockstepClient);
    fn draw(&mut self, debug: &mut DebugText, lockstep: &mut LockstepClient, dt: f32);
    fn draw_ui(&mut self, ctx: &egui::Context, debug: &mut DebugText, lockstep: &mut LockstepClient) {}
    fn reset(&mut self);

    // lobby
    fn handle_generic_message(&mut self, peer_id: PeerID, message: &str);
    fn draw_lobby_ui(&mut self, ui: &mut egui::Ui, debug: &mut DebugText, lockstep: &mut LockstepClient) {}
    
}