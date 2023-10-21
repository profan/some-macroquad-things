use macroquad::{prelude::*, ui::{widgets::Window, root_ui, hash}};
use lockstep::lobby::{DEFAULT_LOBBY_PORT, RelayMessage, LobbyClientID, LobbyState};
use nanoserde::SerJson;
use utility::DebugText;

use crate::{game::Game, relay::RelayClient, network::{NetworkClient, ConnectionState}, step::{LockstepClient, TurnState}, extensions::RelayCommandsExt};

#[derive(PartialEq)]
pub enum ApplicationMode {
    Frontend,
    Singleplayer,
    Multiplayer
}

pub struct ApplicationState {
    title: String,
    host_address: String,
    game: Box<dyn Game>,
    debug: DebugText,
    relay: RelayClient,
    lockstep: Option<LockstepClient>,
    net: NetworkClient,
    mode: ApplicationMode,
    current_frame: i64
}

impl ApplicationState {
    
    pub fn new(title: &str, game: Box<dyn Game>) -> ApplicationState {

        let target_host = format!("ws://localhost:{}", DEFAULT_LOBBY_PORT);

        ApplicationState {
            title: title.to_string(),
            host_address: target_host,
            game: game,
            debug: DebugText::new(),
            relay: RelayClient::new(),
            lockstep: None,
            net: NetworkClient::new(),
            mode: ApplicationMode::Frontend,
            current_frame: 0
        }

    }

    pub fn target_host(&self) -> &str {
        &self.host_address
    }

    pub fn set_target_host(&mut self, address: &str) {
        self.host_address = address.to_string();
    }

    fn is_in_running_game(&self) -> bool {
        let is_in_singleplayer = self.is_in_singleplayer();
        let is_in_running_multiplayer_game = self.is_in_multiplayer() && self.relay.is_in_currently_running_lobby();
        is_in_singleplayer || is_in_running_multiplayer_game
    }

    pub fn is_in_frontend(&self) -> bool {
        self.mode == ApplicationMode::Frontend
    }

    pub fn is_in_singleplayer(&self) -> bool {
        self.mode == ApplicationMode::Singleplayer
    }

    pub fn is_in_multiplayer(&self) -> bool {
        self.mode == ApplicationMode::Multiplayer
    }

    pub fn start_singleplayer_game(&mut self) {
        let local_peer_id = 0;
        let new_lockstep_client = LockstepClient::new(local_peer_id);
        self.mode = ApplicationMode::Singleplayer;
        self.lockstep = Some(new_lockstep_client);
        self.game.start_game();
    }

    pub fn start_multiplayer_game(&mut self) {
        self.mode = ApplicationMode::Multiplayer;
    }

    pub fn stop_game(&mut self) {
        if self.mode == ApplicationMode::Singleplayer {
            self.stop_local_game();
        } else if self.mode == ApplicationMode::Multiplayer {
            self.stop_multiplayer_game();
        }
    }

    fn stop_local_game(&mut self) {
        self.game.reset();
        self.game.stop_game();
        self.mode = ApplicationMode::Frontend;
        self.lockstep = None;
    }

    fn stop_multiplayer_game(&mut self) {
        self.game.stop_game();
        self.net.stop_lobby();
    }

    pub fn connect_to_server(&mut self) -> bool {
        self.net.connect(&self.host_address)
    }

    pub fn disconnect_from_server(&mut self) {
        self.lockstep = None;
        self.net.disconnect();
        self.relay.reset();
    }

    pub fn handle_messages(&mut self) {
        match self.mode {
            ApplicationMode::Frontend => self.handle_frontend(),
            ApplicationMode::Singleplayer => self.handle_singleplayer_game(),
            ApplicationMode::Multiplayer => self.handle_multiplayer_game(),
        }
    }

    fn handle_frontend(&mut self) {

    }

    fn handle_singleplayer_game(&mut self) {

    }
    
    fn handle_multiplayer_game(&mut self) {

        let query_server_interval = 100; // every 100 frames? :D
    
        if self.is_in_running_game() == false && self.net.is_connected() && self.current_frame % query_server_interval == 0 {
            self.net.query_active_state();
        }

        self.current_frame += 1;
    
        match self.net.try_recv() {
            Some(msg) => match msg {
                ewebsock::WsEvent::Message(ewebsock::WsMessage::Text(text)) => {
    
                    fn handle_lockstep_message(client_id: LobbyClientID, lockstep_client: &mut Option<LockstepClient>, message: &str) {
                        // handle messages we get, but do not handle messages sent to ourselves!... probably? :D
                        if let Some(lockstep) = lockstep_client && lockstep.peer_id() != client_id {
                            lockstep.handle_message(client_id, message);
                        }
                    }
                    
                    if let Some(event) = self.relay.handle_message(text, |client_id, msg| handle_lockstep_message(client_id, &mut self.lockstep, msg)) {
                        match event {
                            RelayMessage::SuccessfullyJoinedLobby(_) => {
                                if let Some(client_id) = self.relay.get_client_id() {
                                    let new_lockstep_client = LockstepClient::new(client_id);
                                    self.lockstep = Some(new_lockstep_client);
                                } else {
                                    panic!("client didn't have client id for some reason when receiving successfully joined lobby message, should be impossible!");
                                }
                            },
                            RelayMessage::UpdatedLobby(lobby) => {       
                                if let Some(lockstep) = &mut self.lockstep {             
                                    if let Some(our_lobby) = self.relay.get_current_lobby() && our_lobby.id == lobby.id {
                                        lockstep.update_peers(&our_lobby.clients.as_slice());
                                    }
                                }
                            },
                            RelayMessage::LeftLobby(client_id) => {
                                if let Some(lockstep) = &mut self.lockstep && lockstep.peer_id() == client_id {
                                    self.lockstep = None;
                                }
                                self.game.reset();
                            },
                            RelayMessage::StartedLobby => {
                                self.game.start_game();
                            },
                            RelayMessage::StoppedLobby => {
                                if let Some(lockstep) = &mut self.lockstep {
                                    lockstep.reset();
                                }
                                self.game.reset();
                            },
                            _ => ()
                        }
                    }
                },
                ewebsock::WsEvent::Error(_) | ewebsock::WsEvent::Closed => {
                    self.lockstep = None;
                    self.relay.reset();
                    self.game.reset();
                },
                _ => ()
            },
            None => (),
        };
    
    }

    pub fn update(&mut self) {

        if self.mode == ApplicationMode::Singleplayer || self.mode == ApplicationMode::Multiplayer {
            if is_key_pressed(KeyCode::Escape) && self.is_in_running_game() {
                self.stop_game();
            }
        }
    
        if let Some(lockstep) = &mut self.lockstep {
    
            if self.mode == ApplicationMode::Singleplayer {
                lockstep.tick_with(|peer_id, msg| self.game.handle_message(peer_id, msg), |_ ,_| ());
            } else if self.mode == ApplicationMode::Multiplayer && self.relay.is_in_currently_running_lobby() {
                lockstep.tick_with(|peer_id, msg| self.game.handle_message(peer_id, msg), |peer_id, msg| self.net.send_text(RelayMessage::Message(peer_id, msg).serialize_json()));
            }
    
            if lockstep.turn_state() == TurnState::Running {
    
                self.game.start_game();
                self.game.update(&mut self.debug, lockstep);
    
            } else if lockstep.turn_state() == TurnState::Waiting {
    
                self.game.pause_game();

            }
    
        }

    }

    pub fn draw(&mut self) {

        if self.game.is_running() {
            self.game.draw(&mut self.debug);
        }

        draw_debug_text(self);
        draw_ui(self);

    }

}

fn draw_debug_text(state: &mut ApplicationState) {

    state.debug.new_frame();

    if state.net.connection_state() != ConnectionState::Disconnected {
        state.debug.draw_text(format!("connected to host: {}", state.net.connected_host()), utility::TextPosition::TopLeft, BLACK);
    }

    state.debug.draw_text(format!("connection state: {:?}", state.net.connection_state()), utility::TextPosition::TopLeft, BLACK);

    if let Some(client_id) = state.relay.get_client_id() {
        state.debug.draw_text(format!("client id: {}", client_id), utility::TextPosition::TopLeft, BLACK);
    }
        
    if state.net.connection_state() != ConnectionState::Disconnected {

        state.debug.draw_text("all clients", utility::TextPosition::TopRight, BLACK);
        for c in state.relay.get_clients() {
            state.debug.draw_text(format!("{} ({})", c.name.as_str(), c.id), utility::TextPosition::TopRight, BLACK);
        }

        for l in state.relay.get_lobbies() {
            state.debug.draw_text(format!("{} ({})", l.name, l.id), utility::TextPosition::BottomRight, BLACK);
        }
        state.debug.draw_text("all lobbies", utility::TextPosition::BottomRight, BLACK);

    }

    if let Some(lobby) = state.relay.get_current_lobby() {
        state.debug.skip_line(utility::TextPosition::TopLeft);
        state.debug.draw_text(format!("lobby: {} ({})", lobby.name, lobby.id), utility::TextPosition::TopLeft, BLACK);
        let clients_string = lobby.clients.iter().fold(String::new(), |acc, c| acc + " " + &state.relay.client_with_id(*c).unwrap().name);
        state.debug.draw_text(format!("- clients: {}", clients_string.trim()), utility::TextPosition::TopLeft, BLACK);
    }

    if let Some(lockstep) = &state.lockstep {

        state.debug.draw_text(format!("turn part: {}", lockstep.turn_part()), utility::TextPosition::BottomLeft, BLACK);
        state.debug.draw_text(format!("turn number: {}", lockstep.turn_number()), utility::TextPosition::BottomLeft, BLACK);
        state.debug.draw_text(format!("turn length: {}", lockstep.turn_length()), utility::TextPosition::BottomLeft, BLACK);
        state.debug.draw_text(format!("turn delay: {}", lockstep.turn_delay()), utility::TextPosition::BottomLeft, BLACK);
        state.debug.draw_text(format!("turn state: {:?}", lockstep.turn_state()), utility::TextPosition::BottomLeft, BLACK);
        state.debug.draw_text(format!("peers: {}", lockstep.peers().len()), utility::TextPosition::BottomLeft, BLACK);

    }

}

fn draw_lobby_ui(state: &mut ApplicationState) {

    if state.relay.is_in_currently_running_lobby() {
        return;   
    };

    if is_key_pressed(KeyCode::Escape) {
        if state.relay.is_in_lobby() {
            state.net.leave_lobby();
        } else {
            state.disconnect_from_server();
            state.mode = ApplicationMode::Frontend;
        }
    }

    let center_of_screen = vec2(screen_width() / 2.0, screen_height() / 2.0);
    let size_of_window = vec2(400.0, 400.0);

    let lobbies_title = format!("{} - lobbies", state.title);
    let lobby_title = format!("{} - lobby", state.title);

    Window::new(hash!(), center_of_screen - size_of_window / 2.0, size_of_window)
        .label(if state.relay.is_in_lobby() { &lobbies_title } else { &lobby_title })
        .titlebar(true)
        .ui(&mut *root_ui(), |ui| {

        let mut current_y_position = 0.0;

        if state.net.is_connected() {

            if let Some(lobby) = state.relay.get_current_lobby() {

                if lobby.state == LobbyState::Open {
                
                    let label_text = format!("lobby: {}", lobby.name);
                    let label_size_half = ui.calc_size(&label_text) / 2.0;
                    ui.label(vec2(size_of_window.x / 2.0, 0.0) - vec2(label_size_half.x, 0.0) + vec2(0.0, current_y_position), &label_text);
                    current_y_position += label_size_half.y * 2.0;

                    for client_id in &lobby.clients {

                        let c = state.relay.client_with_id(*client_id).unwrap();

                        let label_text = format!("{} ({})", c.name, c.id);
                        let label_size_half = ui.calc_size(&label_text) / 2.0;
                        ui.label(vec2(size_of_window.x / 2.0, 0.0) - vec2(label_size_half.x, 0.0) + vec2(0.0, current_y_position), &label_text);
                        current_y_position += label_size_half.y * 2.0;

                    }

                    let label_text = "start";
                    let label_size_half = ui.calc_size(label_text) / 2.0;
                    if ui.button(vec2(size_of_window.x / 2.0, 0.0) - vec2(label_size_half.x, 0.0) + vec2(0.0, current_y_position), label_text) {
                        state.net.start_lobby();
                    }
                    current_y_position += label_size_half.y * 2.0;

                    let label_text = "leave";
                    let label_size_half = ui.calc_size(label_text) / 2.0;
                    if ui.button(vec2(size_of_window.x / 2.0, 0.0) - vec2(label_size_half.x, 0.0) + vec2(0.0, current_y_position), label_text) {
                        state.net.leave_lobby();
                    }
                    current_y_position += label_size_half.y * 2.0;

                } else {

                    // running?

                }
    
            } else {

                if state.relay.get_lobbies().is_empty() == false {
                    for l in state.relay.get_lobbies() {
    
                        if let Some(client_id) = state.relay.get_client_id() && state.relay.lobby_of_client(client_id).is_some_and(|lobby| lobby.id == l.id) {
                            continue;
                        }
        
                        let lobby_text = format!("{} ({})", l.name, l.id);
                        let lobby_size_half = ui.calc_size(&lobby_text) / 2.0;
                        ui.label(vec2(size_of_window.x / 2.0 - lobby_size_half.x, 0.0), &lobby_text);
                        current_y_position += lobby_size_half.y * 2.0;
        
                        let button_size_half = ui.calc_size("join");
                        if ui.button(vec2(size_of_window.x / 2.0 - lobby_size_half.x, 0.0) + vec2(0.0, lobby_size_half.y * 2.0) + vec2(button_size_half.x, 0.0), "join") {
                            state.net.join_lobby(l.id);
                        }
                        current_y_position += button_size_half.y * 2.0;
            
                    }
                } else {
        
                    let label_text = "there appears to be no lobbies!";
                    let label_size_half = ui.calc_size(label_text) / 2.0;
                    ui.label(vec2(size_of_window.x / 2.0 - label_size_half.x, 0.0), label_text);
                    current_y_position += label_size_half.y * 2.0;
        
                };

                let label_text = "create new lobby";
                let label_size_half = ui.calc_size(label_text) / 2.0;
                if ui.button(vec2(size_of_window.x / 2.0, 0.0) - vec2(label_size_half.x, 0.0) + vec2(0.0, current_y_position), label_text) {
                    state.net.query_active_state();
                    state.net.create_new_lobby();
                }
                current_y_position += label_size_half.y * 2.0;

                if false {
                    let label_text = "refresh";
                    let label_size_half = ui.calc_size(label_text) / 2.0;
                    if ui.button(vec2(size_of_window.x / 2.0, 0.0) - vec2(label_size_half.x, 0.0) + vec2(0.0, current_y_position), label_text) {
                        // refresh the current lobbies and players?
                    }
                    current_y_position += label_size_half.y * 2.0;
                }

            }

        } else if state.net.is_connecting() {

            let label_text = "connecting...";
            let center_of_window = size_of_window / 2.0 - ui.calc_size(label_text) / 2.0;
            ui.label(center_of_window, label_text);

        }

        if state.net.is_connected() == false && state.net.is_connecting() == false {

            let label_text = "connect to server";
            let label_size_half = ui.calc_size(label_text) / 2.0;
            if ui.button(vec2(size_of_window.x / 2.0, size_of_window.y - label_size_half.y * 4.0) - vec2(label_size_half.x, 0.0), label_text) {
                state.connect_to_server();
            }

        }

        if state.net.is_connected() {

            let label_text = "disconnect from server";
            let label_size_half = ui.calc_size(label_text) / 2.0;
            if ui.button(vec2(size_of_window.x / 2.0, size_of_window.y - label_size_half.y * 4.0) - vec2(label_size_half.x, 0.0), label_text) {
                state.disconnect_from_server();
            }

        }

    });

}

fn draw_main_menu_ui(state: &mut ApplicationState) {

    let center_of_screen = vec2(screen_width() / 2.0, screen_height() / 2.0);
    let size_of_window = vec2(400.0, 400.0);

    let lockstep_main_menu_title = format!("{} - main menu", state.title);
    Window::new(hash!(), center_of_screen - size_of_window / 2.0, size_of_window)
        .label(&lockstep_main_menu_title)
        .titlebar(true)
        .ui(&mut *root_ui(), |ui| {

        let mut current_y_position = 0.0;

        let label_text = "singleplayer";
        let label_size_half = ui.calc_size(label_text) / 2.0;
        if ui.button(vec2(size_of_window.x / 2.0, 0.0) - vec2(label_size_half.x, 0.0) + vec2(0.0, current_y_position), label_text) {
            state.start_singleplayer_game();
        }
        current_y_position += label_size_half.y * 2.0;

        let label_text = "multiplayer";
        let label_size_half = ui.calc_size(label_text) / 2.0;
        if ui.button(vec2(size_of_window.x / 2.0, 0.0) - vec2(label_size_half.x, 0.0) + vec2(0.0, current_y_position), label_text) {
            state.start_multiplayer_game();
        }
        current_y_position += label_size_half.y * 2.0;

    });

}

fn draw_ui(state: &mut ApplicationState) {

    if state.mode == ApplicationMode::Frontend {
        draw_main_menu_ui(state);
    }

    if state.mode == ApplicationMode::Singleplayer {
        // ... do we need anything?
    }

    if state.mode == ApplicationMode::Multiplayer {
        draw_lobby_ui(state);
    }

}