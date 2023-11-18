use macroquad::{prelude::*, ui::{widgets::Window, root_ui, hash}};
use lockstep::lobby::{DEFAULT_LOBBY_PORT, RelayMessage, LobbyClientID, LobbyState, Lobby};
use nanoserde::SerJson;
use utility::DebugText;
use yakui::{Direction, Response, widgets::{ButtonWidget, Pad}};

use crate::{game::Game, relay::RelayClient, network::{NetworkClient, ConnectionState}, step::{LockstepClient, TurnState}, extensions::RelayCommandsExt};

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
    net: NetworkClient,
    mode: ApplicationMode,
    debug_text_colour: Color,
    current_frame: i64
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
            net: NetworkClient::new(),
            mode: ApplicationMode::Frontend,
            debug_text_colour: WHITE,
            current_frame: 0
        }

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
        let new_lockstep_client = LockstepClient::new(local_peer_id, true);
        self.mode = ApplicationMode::Singleplayer;
        self.game.start_game(&new_lockstep_client);
        self.lockstep = Some(new_lockstep_client);
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

    pub fn ping_clients(&mut self) {

        let Some(current_client_id) = self.relay.get_client_id() else { return; };

        for c in self.relay.get_clients().clone() {
            self.net.ping(current_client_id, Some(c.id));
            self.relay.ping(current_client_id, Some(c.id));
        }
        
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
    
        // #FIXME: previously also queried if not in a game, but maybe just always update?
        if self.net.is_connected() && self.current_frame % query_server_interval == 0 {
            // query for lobby/ping/etc state
            self.net.query_active_state();
            self.ping_clients()
            // #TODO: ping server too?
            // self.net.ping(self.relay.get_client_id().expect("could not get current client id?"), None);
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
                                    let is_singleplayer = false;
                                    let new_lockstep_client = LockstepClient::new(client_id, is_singleplayer);
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
                                if let Some(lockstep) = &self.lockstep {
                                    self.game.start_game(lockstep);
                                } else {
                                    println!("could not start the game as no active lockstep client, definitely an error!");
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
                                    self.net.pong(Some(client_id), from_client_id);
                                }
                            }
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
    
                self.game.resume_game();
                self.game.update(&mut self.debug, lockstep);
    
            } else if lockstep.turn_state() == TurnState::Waiting {
    
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
            let clients_string = lobby.clients.iter().fold(String::new(), |acc, c| acc + " " + &self.relay.client_with_id(*c).unwrap().name);
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
    
    fn draw_lobby_ui(&mut self) {

        let lobby = self.relay.get_current_lobby().expect("called draw_lobby_ui without there being a current lobby!");

        yakui::label(format!("lobby: {}", lobby.name.clone()));

        yakui::column(|| {

            for client_id in &lobby.clients {
                let c = self.relay.client_with_id(*client_id).unwrap();
                yakui::label(format!("{} ({})", c.name, client_id));
            }

        });

        yakui_min_row(|| {
            if yakui::button("start").clicked {
                self.net.start_lobby();
            }

            if yakui::button("leave").clicked {
                self.net.leave_lobby();
            }
        });

    }

    fn draw_multiplayer_lobby_ui(&mut self) {
    
        if self.relay.is_in_currently_running_lobby() {
            return;   
        };
    
        if is_key_pressed(KeyCode::Escape) {
            if self.relay.is_in_lobby() {
                self.net.leave_lobby();
            } else {
                self.disconnect_from_server();
                self.mode = ApplicationMode::Frontend;
            }
        }
    
        let lobbies_title = format!("{} - lobbies", self.title);
        let lobby_title = format!("{} - lobby", self.title);

        draw_menu_window_with_column(|| {

            yakui::label(if self.relay.is_in_lobby() { lobby_title } else { lobbies_title });

            if self.net.is_connected() {

                if self.relay.is_in_lobby() {
                    self.draw_lobby_ui();
                } else {
                    if self.relay.get_lobbies().is_empty() == false {

                        for lobby in self.relay.get_lobbies() {

                            let lobby_text = format!("{} ({}) - {:?}", lobby.name, lobby.id, lobby.state);
                            yakui_min_row(|| {
                                yakui::label(lobby_text);
                                if lobby.state == LobbyState::Open && yakui::button("join").clicked {
                                    self.net.join_lobby(lobby.id);
                                }
                            });

                        }

                    } else {
                        yakui::label("there appears to be no lobbies!");
                    }

                    if yakui::button("create new lobby").clicked {
                        self.net.query_active_state();
                        self.net.create_new_lobby();
                    }

                }

            }

            if self.relay.is_in_lobby() == false {

                if self.net.is_connecting() {
                    yakui::label("connecting...");
                }

                if self.net.is_connected() == false {

                    // let mut host_textbox = yakui::widgets::TextBox::new("localhost");
                    // host_textbox.style.color = yakui::Color::WHITE;

                    // if let Some(address) = &host_textbox.show().text {
                    //     self.set_target_host(&address);
                    // }

                    yakui::label(self.target_host().to_string());

                    if yakui::button("connect to server").clicked {
                        self.connect_to_server();
                    }

                } else {
                    if yakui::button("disconnect from server").clicked {
                        self.disconnect_from_server();
                    }
                }

            }

        });
    
    }
    
    fn draw_main_menu_ui(&mut self) {
    
        let lockstep_main_menu_title = format!("{}", self.title);

        draw_menu_window_with_column(|| {

            yakui::label(lockstep_main_menu_title);
            
            if yakui::button("singleplayer").clicked {
                self.start_singleplayer_game();
            }

            if yakui::button("multiplayer").clicked {
                self.start_multiplayer_game();
            }

        });
    
    }
    
    fn draw_ui(&mut self) {
    
        if self.mode == ApplicationMode::Frontend {
            self.draw_main_menu_ui();
        }
    
        if self.mode == ApplicationMode::Singleplayer {
            // ... do we need anything?
        }
    
        if self.mode == ApplicationMode::Multiplayer {
            self.draw_multiplayer_lobby_ui();
        }
    
    }

    pub fn draw(&mut self) {

        yakui_macroquad::start();

        if self.game.is_running() {
            self.game.draw(&mut self.debug);
        }

        self.draw_debug_text();
        self.draw_ui();

        yakui_macroquad::finish();

        yakui_macroquad::draw();

    }

}

pub fn draw_menu_window<F>(children: F)
    where F: FnOnce() -> ()
{
    yakui::center(|| {
        yakui::pad(Pad::all(32.0), || {
            yakui::colored_box_container(yakui::Color::WHITE.adjust(0.1), ||{
                yakui::pad(Pad::all(32.0), || {
                    children();               
                });
            });
        });
    });
}

pub fn draw_menu_window_with_column<F>(children: F)
    where F: FnOnce() -> ()
{
    draw_menu_window(|| {
        let mut list = yakui::widgets::List::new(Direction::Down);
        list.cross_axis_alignment = yakui::CrossAxisAlignment::Center;
        list.item_spacing = 16.0;
        
        list.show(|| {
            children();
        });
    });
}

pub fn yakui_min_column<F>(children: F)
    where F: FnOnce() -> ()
{
    let mut column = yakui::widgets::List::column();
    column.main_axis_size = yakui::MainAxisSize::Min;
    column.show(children);
}

pub fn yakui_min_row<F>(children: F)
    where F: FnOnce() -> ()
{
    let mut row = yakui::widgets::List::row();
    row.main_axis_size = yakui::MainAxisSize::Min;
    row.show(children);
}