use egui_macroquad::egui;
use lockstep::lobby::Lobby;
use lockstep::lobby::LobbyClient;
use lockstep::lobby::LobbyClientID;
use utility::DebugText;
use crate::extensions::RelayCommandsExt;
use crate::{relay::RelayClient, step::{LockstepClient, PeerID}};

pub struct GameContext<'a> {
    pub(crate) debug_text: &'a mut DebugText,
    pub(crate) relay_client: &'a RelayClient,
    pub(crate) lockstep: &'a mut LockstepClient
}

impl<'a> GameContext<'a> {

    pub fn debug_text(&mut self) -> &mut DebugText {
        &mut self.debug_text
    }
    
    pub fn current_lobby(&self) -> Option<&Lobby> {
        self.relay_client.get_current_lobby()
    }

    pub fn get_lobby_client(&self, client_id: LobbyClientID) -> &LobbyClient {
        &self.relay_client.get_client(client_id).unwrap()
    }

    pub fn get_lobby_client_name(&self, client_id: LobbyClientID) -> &str {
        &self.relay_client.get_client(client_id).unwrap().name
    }

    pub fn lockstep_mut(&mut self) -> &mut LockstepClient {
        self.lockstep
    }

    pub fn lockstep(&self) -> &LockstepClient {
        self.lockstep
    }

}

pub struct GameLobbyContext<'a> {
    pub(crate) debug_text: &'a mut DebugText,
    pub(crate) relay_client: &'a mut RelayClient,
    pub(crate) lockstep: &'a mut LockstepClient,
    pub(crate) new_lobby_data_to_push: Option<String>
}

impl<'a> GameLobbyContext<'a> {

    pub fn debug_text(&mut self) -> &mut DebugText {
        &mut self.debug_text
    }

    pub fn current_lobby(&self) -> Option<&Lobby> {
        self.relay_client.get_current_lobby()
    }

    pub fn get_lobby_client(&self, client_id: LobbyClientID) -> Option<&LobbyClient> {
        self.relay_client.get_client(client_id)
    }

    pub fn get_lobby_client_name(&self, client_id: LobbyClientID) -> &str {
        if let Some(client) = self.relay_client.get_client(client_id) {
            &client.name
        } else {
            "INVALID_CLIENT"
        }
    }

    pub fn push_new_lobby_data(&mut self, new_lobby_data: String) {
        self.relay_client.send_lobby_data(new_lobby_data);
    }

    pub fn get_new_lobby_data(&self) -> Option<&String> {
        self.new_lobby_data_to_push.as_ref()
    }

    pub fn lockstep_mut(&mut self) -> &mut LockstepClient {
        self.lockstep
    }

    pub fn lockstep(&self) -> &LockstepClient {
        self.lockstep
    }

    pub fn is_player_boss(&self) -> bool {
        if let Some(lobby) = self.current_lobby() {
            lobby.boss == self.lockstep.peer_id()
        } else {
            false
        }
    }

    pub fn get_lobby_boss_id(&self) -> Option<LobbyClientID> {
        if let Some(lobby) = self.current_lobby() {
            Some(lobby.boss)
        } else {
            None
        }
    }

}

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
    fn update(&mut self, ctx: &mut GameContext);
    fn draw(&mut self, ctx: &mut GameContext, dt: f32);
    fn draw_ui(&mut self, _ui_ctx: &egui::Context, _ctx: &mut GameContext) {}
    fn reset(&mut self);

    // lobby
    fn on_enter_lobby(&mut self) {}
    fn on_leave_lobby(&mut self) {}

    fn on_client_joined_lobby(&mut self, _peer_id: PeerID, _ctx: &mut GameLobbyContext) {}
    fn on_client_left_lobby(&mut self, _peer_id: PeerID, _ctx: &mut GameLobbyContext) {}

    fn handle_lobby_update(&mut self, _new_lobby_data: String) {}
    fn handle_generic_message(&mut self, peer_id: PeerID, message: &str);

    fn handle_lobby_tick(&mut self, _ctx: &mut GameLobbyContext) {}
    fn draw_lobby_ui(&mut self, _ui: &mut egui::Ui, _ctx: &mut GameLobbyContext) {}
    
} 