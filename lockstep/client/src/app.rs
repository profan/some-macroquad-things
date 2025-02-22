use egui_macroquad::egui::{self, Align2};
use macroquad::prelude::*;
use lockstep::lobby::{LobbyClientID, LobbyState, RelayMessage, DEFAULT_LOBBY_PORT};
use nanoserde::{DeJson, SerJson};
use utility::{screen_dimensions, DebugText};

use crate::{command::ApplicationCommand, extensions::RelayCommandsExt, game::{Game, GameContext, GameLobbyContext}, network::{ConnectionState, NetworkClient, NetworkClientSwitch, NetworkClientWebSocket}, relay::RelayClient, step::{LockstepClient, TickResult, TurnCommand, TurnState}};

#[derive(PartialEq)]
pub enum ApplicationMode {
    Frontend,
    Singleplayer,
    Multiplayer
}

pub struct ApplicationState<GameType> where GameType: Game {
    title: String,
    host_address: String,
    game: GameType,
    debug: DebugText,
    relay: RelayClient,
    lockstep: Option<LockstepClient>,
    net: NetworkClientSwitch,
    mode: ApplicationMode,
    debug_text_colour: Color,
    current_frame: i64,
    current_tick: i64,
    quit_requested: bool
}

impl<GameType> ApplicationState<GameType> where GameType: Game {
    
    pub fn new(title: &str, specific_game: GameType) -> ApplicationState<GameType> {

        let target_host = format!("ws://localhost:{}", DEFAULT_LOBBY_PORT);

        ApplicationState {
            title: title.to_string(),
            host_address: target_host,
            game: specific_game,
            debug: DebugText::new(),
            relay: RelayClient::new(),
            lockstep: None,
            net: NetworkClientSwitch::new(),
            mode: ApplicationMode::Frontend,
            debug_text_colour: WHITE,
            current_frame: 0,
            current_tick: 0,
            quit_requested: false
        }

    }

    pub fn was_quit_requested(&self) -> bool {
        self.quit_requested
    }
    
    pub fn set_quit_requested(&mut self, value: bool) {
        self.quit_requested = value;
    }

    pub fn get_game(&mut self) -> &mut GameType {
        &mut self.game
    }

    pub fn get_debug_text_colour(&self) -> Color {
        self.debug_text_colour
    }

    pub fn set_debug_text_colour(&mut self, color: Color) {
        self.debug_text_colour = color;
    }
     
    pub async fn load_resources(&mut self) {
        self.game.load_resources().await;
    }

    pub fn target_host(&self) -> &str {
        &self.host_address
    }

    pub fn set_target_host(&mut self, address: &str) {
        self.host_address = format!("ws://{}:{}", address.to_string(), DEFAULT_LOBBY_PORT);
    }

    fn is_in_running_game(&self) -> bool {
        let is_in_running_singleplayer_game = self.is_in_singleplayer() && self.game.is_running();
        let is_in_running_multiplayer_game = self.is_in_multiplayer() && self.relay.is_in_currently_running_lobby();
        is_in_running_singleplayer_game || is_in_running_multiplayer_game
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

        self.net.start_singleplayer();

        let local_peer_id = 0;
        let new_lockstep_client = LockstepClient::new(local_peer_id, true);
        self.mode = ApplicationMode::Singleplayer;

        if self.game.should_automatically_start() {
            self.game.start_game(&new_lockstep_client);
        }

        self.lockstep = Some(new_lockstep_client);
        self.current_tick = 0;

        self.game.on_enter_lobby();

    }

    pub fn start_multiplayer_game(&mut self) {
        self.mode = ApplicationMode::Multiplayer;
        self.current_tick = 0;
    }

    pub fn stop_game(&mut self) {

        if self.mode == ApplicationMode::Singleplayer {
            self.stop_singleplayer_game();
        } else if self.mode == ApplicationMode::Multiplayer {
            self.stop_multiplayer_game();
        }

    }

    fn stop_singleplayer_game(&mut self) {
        self.game.on_leave_lobby();
        self.game.reset();
        self.game.stop_game();
        self.mode = ApplicationMode::Frontend;
        self.lockstep = None;
        self.net.stop();
    }

    fn stop_multiplayer_game(&mut self) {
        self.game.stop_game();
        self.relay.stop_lobby();
    }

    pub fn connect_to_server(&mut self) -> bool {
        self.net.start_multiplayer();
        self.net.connect(&self.host_address)
    }

    pub fn disconnect_from_server(&mut self) {
        self.lockstep = None;
        self.net.disconnect();
        self.relay.reset();
        self.net.stop()
    }

    pub fn ping_clients(&mut self) {

        let Some(current_client_id) = self.relay.get_client_id() else { return; };

        for c in self.relay.get_clients().clone() {
            self.relay.ping(current_client_id, Some(c.id));
            self.relay.ping(current_client_id, Some(c.id));
        }
        
    }

    pub fn handle_messages(&mut self) {

        match self.mode {
            ApplicationMode::Frontend => self.handle_frontend(),
            ApplicationMode::Singleplayer => self.handle_singleplayer_game(),
            ApplicationMode::Multiplayer => self.handle_multiplayer_game(),
        }

        self.relay.handle_queued_messages(|m| self.net.send_text(m.to_string()));

    }

    fn handle_frontend(&mut self) {

    }

    fn handle_singleplayer_game(&mut self) {

        self.handle_singleplayer_or_multiplayer_game();

    }
    
    fn handle_multiplayer_game(&mut self) {

        self.handle_singleplayer_or_multiplayer_game();
    
    }

    fn handle_singleplayer_or_multiplayer_game(&mut self) {

        let query_server_interval = 100;
        // every 100 frames? :D
    
        // #FIXME: previously also queried if not in a game, but maybe just always update?
        if self.net.is_connected() && self.current_frame % query_server_interval == 0 {
            // query for lobby/ping/etc state
            self.relay.query_active_state();
            self.ping_clients();
        }
    
        self.current_frame += 1;
    
        match self.net.try_recv() {
            Some(msg) => match msg {
                ewebsock::WsEvent::Message(ewebsock::WsMessage::Text(text)) => {
    
                    fn handle_lockstep_message(client_id: LobbyClientID, lockstep_client: &mut Option<LockstepClient>, message: &str) {
    
                        // handle messages we get, but do not handle messages sent to ourselves!... probably? :D
                        if let Some(lockstep) = lockstep_client && lockstep.peer_id() != client_id {
    
                            match ApplicationCommand::deserialize_json(&message) {
                                Ok(cmd) => match cmd {
                                    ApplicationCommand::GenericCommand(generic_command) => lockstep.handle_generic_message(client_id, generic_command),
                                    ApplicationCommand::TurnCommand(turn_command) => lockstep.handle_message(client_id, turn_command),
                                },
                                Err(err) => {
                                    println!("[LockstepClient] got error: {} when processing message: {}", err, message);
                                    return;
                                },
                            };
                    
                        }
    
                    }
            
                    if let Some(event) = self.relay.handle_message(text, |client_id, msg| handle_lockstep_message(client_id, &mut self.lockstep, msg)) {
                        match event {
                            RelayMessage::SuccessfullyJoinedLobby(_) => {
                                if let Some(client_id) = self.relay.get_client_id() {
                                    let is_singleplayer = false;
                                    let new_lockstep_client = LockstepClient::new(client_id, is_singleplayer);
                                    self.lockstep = Some(new_lockstep_client);
                                    self.game.on_enter_lobby();
                                } else {
                                    panic!("[LockstepClient] client didn't have client id for some reason when receiving successfully joined lobby message, should be impossible!");
                                }
                            },
                            RelayMessage::UpdatedLobby(lobby) => {       
                                if let Some(lockstep) = &mut self.lockstep {             
                                    if let Some(our_lobby) = self.relay.get_current_lobby() && our_lobby.id == lobby.id {
                                        lockstep.update_peers(&our_lobby.clients.as_slice());
                                        self.game.handle_lobby_update(lobby.data.clone());
                                    }
                                }
                            },
                            RelayMessage::JoinedLobby(client_id) => {
                                if let Some(lockstep) = &mut self.lockstep {
                                    if lockstep.peer_id() == client_id {
    
                                    } else {
                                        self.game.on_client_joined_lobby(client_id, lockstep);
                                    }
                                }
                            },
                            RelayMessage::LeftLobby(client_id) => {
                                if let Some(lockstep) = &mut self.lockstep {
                                    if lockstep.peer_id() == client_id {
                                        self.lockstep = None;
                                        self.game.on_leave_lobby();
                                    } else {
                                        self.game.on_client_left_lobby(client_id, lockstep);
                                    }
                                }
                                self.game.reset();
                            },
                            RelayMessage::StartedLobby => {
                                if let Some(lockstep) = &mut self.lockstep {
                                    lockstep.reset();
                                }
                                if let Some(lockstep) = &self.lockstep {
                                    self.game.start_game(lockstep);
                                } else {
                                    println!("[LockstepClient] could not start the game as no active lockstep client, definitely an error!");
                                }
                        
                            },
                            RelayMessage::StoppedLobby => {
                                if let Some(lockstep) = &mut self.lockstep {
                                    lockstep.reset();
                                }
                                self.game.reset();
                            },
                            RelayMessage::Ping(from_client_id, to_client_id) => {
                                if let Some(client_id) = self.relay.get_client_id() && to_client_id == Some(client_id) {
                                    self.relay.pong(Some(client_id), from_client_id);
                                }
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

        self.handle_messages();

        if self.mode == ApplicationMode::Singleplayer || self.mode == ApplicationMode::Multiplayer {

            if is_key_pressed(KeyCode::Escape) && self.is_in_running_game() {
                self.stop_game();
            }

            if is_key_pressed(KeyCode::Escape) && self.is_in_singleplayer() && self.is_in_running_game() == false {
                self.stop_singleplayer_game();
            }

            if is_key_pressed(KeyCode::Escape) && self.is_in_multiplayer() && self.is_in_running_game() == false {
                self.stop_multiplayer_game();
            }

        }

        if let Some(lockstep) = &mut self.lockstep {

            if self.mode == ApplicationMode::Singleplayer {
                lockstep.handle_generic_messages_with(
                    |peer_id, msg| self.game.handle_generic_message(peer_id, msg),
                    |_, _| ()
                );
            } else if self.mode == ApplicationMode::Multiplayer {
                lockstep.handle_generic_messages_with(
                    |peer_id, msg| self.game.handle_generic_message(peer_id, msg),
                    |peer_id, msg| self.net.send_text(RelayMessage::Message(peer_id, msg).serialize_json())
                );
            }

        }
    
        if let Some(lockstep) = &mut self.lockstep && self.game.is_running() {
    
            if self.mode == ApplicationMode::Singleplayer {

                lockstep.tick_with(
                    |peer_id, msg| self.game.handle_game_message(peer_id, msg),
                    |_ ,_| ()
                );

            } else if self.mode == ApplicationMode::Multiplayer && self.relay.is_in_currently_running_lobby() {

                let tick_result = lockstep.tick_with(
                    |peer_id, msg| self.game.handle_game_message(peer_id, msg),
                    |peer_id, msg| self.net.send_text(RelayMessage::Message(peer_id, msg).serialize_json())
                );

                if tick_result == TickResult::RunningNewTurn {
                    // println!("tick: {}, turn: {}", self.current_tick, lockstep.turn_number());
                }

            }
    
            if lockstep.turn_state() == TurnState::Running {

                self.game.resume_game();

                // only tick when actually running
                let mut game_context = GameContext { debug_text: &mut self.debug, relay_client: &self.relay, lockstep };
                self.game.update(&mut game_context);
                self.current_tick += 1;

            } else {

                self.game.pause_game();

            }
    
        }

    }

    fn draw_debug_text(&mut self) {

        self.debug.new_frame();
    
        if self.net.connection_state() != ConnectionState::Disconnected {
            self.debug.draw_text(format!("connected to host: {}", self.net.connected_host()), utility::TextPosition::TopLeft, self.debug_text_colour);
        }
    
        self.debug.draw_text(format!("connection state: {:?}", self.net.connection_state()), utility::TextPosition::TopLeft, self.debug_text_colour);
    
        if let Some(client_id) = self.relay.get_client_id() {
            self.debug.draw_text(format!("client id: {}", client_id), utility::TextPosition::TopLeft, self.debug_text_colour);
        }

        self.debug.draw_text(format!("client tick: {}", self.current_tick), utility::TextPosition::TopLeft, self.debug_text_colour);
            
        if self.net.connection_state() != ConnectionState::Disconnected {
    
            self.debug.draw_text("all clients", utility::TextPosition::TopRight, self.debug_text_colour);
            for c in self.relay.get_clients() {
                let client_rtt_to_us_in_ms = self.relay.get_client_ping(c.id);
                self.debug.draw_text(format!("{} ({}) - {} ms", c.name.as_str(), c.id, client_rtt_to_us_in_ms), utility::TextPosition::TopRight, self.debug_text_colour);
            }
    
            for l in self.relay.get_lobbies() {
                 self.debug.draw_text(format!("{} ({}) - {:?}", l.name, l.id, l.state), utility::TextPosition::BottomRight, self.debug_text_colour);
            }
            self.debug.draw_text("all lobbies", utility::TextPosition::BottomRight, self.debug_text_colour);
    
        }
    
        if let Some(lobby) = self.relay.get_current_lobby() {
            self.debug.skip_line(utility::TextPosition::TopLeft);
            self.debug.draw_text(format!("lobby: {} ({})", lobby.name, lobby.id), utility::TextPosition::TopLeft, self.debug_text_colour);
            let clients_string = lobby.clients.iter().fold(String::new(), |acc, c| acc + " " + &self.relay.client_with_id(*c).map_or("INVALID_USER", |c| &c.name));
            self.debug.draw_text(format!("- clients: {}", clients_string.trim()), utility::TextPosition::TopLeft, self.debug_text_colour);
        }
    
        if let Some(lockstep) = &self.lockstep {
    
            self.debug.draw_text(format!("turn part: {}", lockstep.turn_part()), utility::TextPosition::BottomLeft, self.debug_text_colour);
            self.debug.draw_text(format!("turn number: {}", lockstep.turn_number()), utility::TextPosition::BottomLeft, self.debug_text_colour);
            self.debug.draw_text(format!("turn length: {}", lockstep.turn_length()), utility::TextPosition::BottomLeft, self.debug_text_colour);
            self.debug.draw_text(format!("turn delay: {}", lockstep.turn_delay()), utility::TextPosition::BottomLeft, self.debug_text_colour);
            self.debug.draw_text(format!("turn state: {:?}", lockstep.turn_state()), utility::TextPosition::BottomLeft, self.debug_text_colour);
            self.debug.draw_text(format!("peers: {}", lockstep.peers().len()), utility::TextPosition::BottomLeft, self.debug_text_colour);
    
        }
    
    }
    
    fn draw_lobby_ui(&mut self, ui: &mut egui::Ui) {

        let lobby = self.relay.get_current_lobby().expect("called draw_lobby_ui without there being a current lobby!").clone();

        ui.vertical_centered_justified(|ui| {

            ui.label("clients");
            for client_id in &lobby.clients {
                let c = self.relay.client_with_id(*client_id).unwrap();
                let client_rtt_to_us_in_ms = self.relay.get_client_ping(c.id);
                ui.label(format!("{} (id: {}) - {} ms", c.name, client_id, client_rtt_to_us_in_ms));
            }

            ui.separator();

            let mut lobby_context = GameLobbyContext {
                debug_text: &mut self.debug,
                relay_client: &mut self.relay,
                lockstep: self.lockstep.as_mut().expect("lockstep client instance must be valid here!"),
                new_lobby_data_to_push: None
            };

            self.game.draw_lobby_ui(ui, &mut lobby_context);

        });

        ui.horizontal(|ui| {

            if ui.button("start").clicked() {
                self.relay.start_lobby();
            }

            if ui.button("leave").clicked() {
                self.relay.leave_lobby();
            }

        });

    }

    fn draw_lobby_list_ui(&mut self, ui: &mut egui::Ui) {

        if self.relay.get_lobbies().is_empty() == false {

            for lobby in self.relay.get_lobbies() {

                let lobby_text = format!("{} ({}) - {:?}", lobby.name, lobby.id, lobby.state);

                ui.vertical_centered_justified(|ui| {
                    ui.horizontal(|ui| {
                        ui.label(lobby_text);
                        if lobby.state == LobbyState::Open {
                            if ui.button("join").clicked() {
                                self.relay.join_lobby(lobby.id);
                            }
                        }
                    });
                });

            }

        } else {
            ui.label("there appears to be no lobbies!");
        }

        if ui.button("create new lobby").clicked() {
            self.relay.query_active_state();
            self.relay.create_new_lobby();
        }

    }

    fn leave_lobby_or_disconnect_from_server(&mut self) {
        if self.relay.is_in_lobby() {
            self.relay.leave_lobby();
        } else {
            self.disconnect_from_server();
            self.mode = ApplicationMode::Frontend;
        }
    }

    fn draw_single_player_lobby_ui(&mut self, ctx: &egui::Context) {

        draw_centered_menu_window(ctx, "singleplayer", |ui| {

            let mut lobby_context = GameLobbyContext {
                debug_text: &mut self.debug,
                relay_client: &mut self.relay,
                lockstep: self.lockstep.as_mut().expect("lockstep client instance must be valid here!"),
                new_lobby_data_to_push: None
            };

            self.game.draw_lobby_ui(ui, &mut lobby_context);

            if let Some(new_lobby_data) = lobby_context.new_lobby_data_to_push {
                self.game.handle_lobby_update(new_lobby_data);
            }

        });

    }

    fn draw_multiplayer_lobby_ui(&mut self, ctx: &egui::Context) {
    
        if self.relay.is_in_currently_running_lobby() {
            return;   
        };
    
        if is_key_pressed(KeyCode::Escape) {
            self.leave_lobby_or_disconnect_from_server();
        }

        let current_lobby_name = self.relay.get_current_lobby().and_then(|l| Some(l.name.clone()));
        let lobby_title = format!("{} - lobby - {}", self.title, current_lobby_name.unwrap_or("None".to_string()));
        let lobbies_title = format!("{} - lobbies", self.title);

        let menu_window_title = if self.relay.is_in_lobby() { lobby_title } else { lobbies_title };

        draw_centered_menu_window(ctx, &menu_window_title, |ui| {

            if self.net.is_connected() {
                if self.relay.is_in_lobby() {
                    self.draw_lobby_ui(ui);
                } else {
                    self.draw_lobby_list_ui(ui);
                }
            }

            if self.relay.is_in_lobby() == false {
                self.draw_lobbies_view(ui);
            }

        });
    
    }
    
    fn draw_lobbies_view(&mut self, ui: &mut egui::Ui) {

        if self.net.is_connecting() {
            ui.label("connecting...");
        }
    
        if self.net.is_connected() == false {
    
            if ui.button("connect to relay server").clicked() {
                self.connect_to_server();
            }
    
            if ui.button("back to menu").clicked() {
                self.leave_lobby_or_disconnect_from_server();
            }
    
        } else {

            if ui.button("disconnect from server").clicked() {
                self.disconnect_from_server();
            }

            ui.label(format!("connected to relay server: {}", self.target_host()));
            
        }

    }
    
    fn draw_main_menu_ui(&mut self, ctx: &egui::Context) {
    
        let lockstep_main_menu_title = format!("{}", self.title);

        draw_centered_menu_window(ctx, &lockstep_main_menu_title, |ui| {

            if ui.button("singleplayer").clicked() {
                self.start_singleplayer_game();
            }

            if ui.button("multiplayer").clicked() {
                self.start_multiplayer_game();
            }

            if ui.button("quit").clicked() {
                self.quit_requested = true;
            }

        });
    
    }
    
    fn draw_ui(&mut self, ctx: &egui::Context) {
    
        if self.mode == ApplicationMode::Frontend {
            self.draw_main_menu_ui(ctx);
        }
    
        if self.mode == ApplicationMode::Singleplayer && self.is_in_running_game() == false {
            self.draw_single_player_lobby_ui(ctx);
        }
    
        if self.mode == ApplicationMode::Multiplayer && self.is_in_running_game() == false {
            self.draw_multiplayer_lobby_ui(ctx);
        }
    
    }

    pub fn draw(&mut self, dt: f32) {

        if self.game.is_running() && let Some(lockstep) = &mut self.lockstep {
            let mut game_context = GameContext { debug_text: &mut self.debug, relay_client: &self.relay, lockstep };
            self.game.draw(&mut game_context, dt);
        }

        self.draw_debug_text();

        egui_macroquad::ui(|ctx| {
            if let Some(lockstep) = &mut self.lockstep {
                let mut game_context = GameContext { debug_text: &mut self.debug, relay_client: &self.relay, lockstep };
                self.game.draw_ui(ctx, &mut game_context);
            }
            self.draw_ui(ctx);
        });

        egui_macroquad::draw();

    }

}

pub fn draw_centered_menu_window<F>(ctx: &egui::Context, title: &str, mut contents: F)
    where F: FnMut(&mut egui::Ui) -> ()
{
    let screen_center = screen_dimensions() / 2.0;
    egui::Window::new(title)
        .pivot(Align2::CENTER_CENTER)
        .fixed_pos((screen_center.x, screen_center.y))
        .collapsible(false)
        .resizable(false)
        .frame(egui::Frame::default().inner_margin(egui::Margin::same(16.0)))
        .show(ctx, |ui| {
            ui.vertical_centered_justified(|ui| {
                contents(ui);
            });
        });
}