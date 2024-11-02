use std::collections::BTreeMap;

use lockstep::lobby::Lobby;
use lockstep::lobby::LobbyClient;
use lockstep::lobby::LobbyClientID;
use lockstep::lobby::LobbyID;
use lockstep::lobby::LobbyState;
use lockstep::lobby::RelayMessage;
use macroquad::time::get_time;
use nanoserde::DeJson;

const IS_DEBUGGING: bool = false;

struct RelayPingStats {
    send_time: i32,
    receive_time: i32,
    last_time: i32
}

impl RelayPingStats {
    fn new() -> RelayPingStats {
        RelayPingStats { send_time: 0, receive_time: 0, last_time: 999 }
    }

    fn ping(&self) -> i32 {

        if self.receive_time != self.send_time {
            self.receive_time - self.send_time
        } else {
            self.last_time
        }

    }
}

pub struct RelayClient {
    client_id: Option<LobbyClientID>,
    current_lobby_id: Option<LobbyID>,
    client_stats: BTreeMap<LobbyClientID, RelayPingStats>, // milliseconds latency
    server_stats: RelayPingStats,
    clients: Vec<LobbyClient>,
    lobbies: Vec<Lobby>,
    is_debug: bool
}

impl RelayClient {

    pub fn new() -> RelayClient {
        RelayClient {
            client_id: None,
            current_lobby_id: None,
            client_stats: BTreeMap::new(),
            server_stats: RelayPingStats::new(),
            clients: Vec::new(),
            lobbies: Vec::new(),
            is_debug: false
        }
    }

    pub fn is_in_lobby(&self) -> bool {
        self.get_current_lobby().is_some()
    }

    pub fn is_in_currently_running_lobby(&self) -> bool {
        if let Some(lobby) = self.get_current_lobby() && lobby.state == LobbyState::Running {
            true
        } else {
            false
        }
    }

    pub fn get_current_lobby(&self) -> Option<&Lobby> {
        if let Some(lobby_id) = self.current_lobby_id && let Some(lobby) = self.lobby_with_id(lobby_id) {
            Some(lobby)
        } else {
            None
        }
    }

    pub fn get_client_ping(&self, client_id: LobbyClientID) -> i32 {
        if let Some(client_stats) = self.client_stats.get(&client_id) {
            client_stats.ping()
        } else {
            999
        }
    }

    pub fn get_client_id(&self) -> Option<LobbyClientID> {
        self.client_id
    }

    pub fn get_lobbies(&self) -> &Vec<Lobby> {
        &self.lobbies
    }

    pub fn get_clients(&self) -> &Vec<LobbyClient> {
        &self.clients
    }

    pub fn client_id(&mut self, client_id: LobbyClientID) {

        if self.is_debug {
            println!("[RelayClient] got assigned a client id: {}", client_id);
        }

        self.client_id = Some(client_id);

    }

    pub fn active_lobbies(&mut self, lobbies: &Vec<Lobby>) {

        if self.is_debug {
            println!("[RelayClient] got active lobbies response!");
        }

        self.lobbies = lobbies.clone();

    }

    pub fn active_players(&mut self, players: &Vec<LobbyClient>) {

        if self.is_debug {
            println!("[RelayClient] got active player response!");
        }

        self.clients = players.clone();

    }

    pub fn successfully_joined_lobby(&mut self, lobby_id: LobbyID) {

        if self.is_debug {
            println!("[RelayClient] successfully joined the lobby: {}", lobby_id);
        }

        self.current_lobby_id = Some(lobby_id);

    }

    pub fn failed_to_join_lobby(&mut self, lobby_id: LobbyID, reason: &String) {

        if self.is_debug {
            println!("[RelayClient] failed to join the lobby: {} because: {}", lobby_id, reason);
        }

    }

    pub fn joined_lobby(&mut self, client_id: LobbyClientID) {

        if self.is_debug {
            println!("[RelayClient] client with id: {} joined the lobby: {:?}", client_id, self.current_lobby_id);
        }

        let current_lobby = self.lobbies.iter_mut().find(|lobby| lobby.id == self.current_lobby_id.unwrap()).unwrap();
        current_lobby.clients.push(client_id);

    }

    pub fn updated_lobby(&mut self, lobby: &Lobby) {

        if self.is_debug {
            println!("[RelayClient] got an update for the lobby: {}", lobby.id);
        }

        self.add_or_update_lobby(lobby);

    }

    pub fn left_lobby(&mut self, client_id: LobbyClientID) {
        
        println!("[RelayClient] client with id: {} left the lobby: {:?}", client_id, self.current_lobby_id);
        let current_lobby = self.lobbies.iter_mut().find(|lobby| lobby.id == self.current_lobby_id.unwrap()).unwrap();

        // if we're leaving our own lobby, set our current lobby to none
        if let Some(current_client_id) = self.client_id && client_id == current_client_id {
            self.current_lobby_id = None;
        }

        current_lobby.clients.retain(|id| *id != client_id);

    }

    pub fn ping(&mut self, from_client_id: LobbyClientID, to_client_id: Option<LobbyClientID>) {

        if let Some(current_client_id) = self.client_id && from_client_id == current_client_id {

            if let Some(to_client_id) = to_client_id {

                if self.is_debug {
                    println!("[RelayClient] sent ping message to: {}!", to_client_id);
                }

                let current_time_in_ms = (get_time() * 1000.0) as i32;

                if let Some(stats) = self.client_stats.get_mut(&to_client_id) {
                    stats.receive_time = current_time_in_ms;
                    stats.send_time = current_time_in_ms;
                } else {
                    self.client_stats.insert(to_client_id, RelayPingStats { send_time: current_time_in_ms, receive_time: current_time_in_ms, last_time: 999 });
                }

            } else {
                if self.is_debug {
                    println!("[RelayClient] sent ping message to server!");
                }
            }

        }

    }

    pub fn pong(&mut self, from_client_id: Option<LobbyClientID>, to_client_id: LobbyClientID) {

        if let Some(client_id) = from_client_id && to_client_id == self.client_id.expect("client id should never be empty here") {

            if self.is_debug {
                println!("[RelayClient] got pong message from: {}, updating ping!", client_id);
            }

            let current_time_in_ms = (get_time() * 1000.0) as i32;
            if let Some(stats) = self.client_stats.get_mut(&client_id) {
                stats.receive_time = current_time_in_ms;
                stats.last_time = stats.ping();
            }

        } else {
            if self.is_debug {
                println!("[RelayClient] got pong message from server!");
            }
        }

    }

    pub fn client_with_id(&self, client_id: LobbyClientID) -> Option<&LobbyClient> {
        for c in &self.clients {
            if c.id == client_id {
                return Some(c);
            }
        }
        return None;
    }

    pub fn lobby_of_client(&self, client_id: LobbyClientID) -> Option<&Lobby> {
        for l in &self.lobbies {
            if l.clients.contains(&client_id) {
                return Some(l);
            }
        }
        return None;  
    }

    pub fn lobby_with_id(&self, lobby_id: LobbyID) -> Option<&Lobby> {
        for l in &self.lobbies {
            if l.id == lobby_id {
                return Some(l);
            }
        }
        return None;  
    }

    fn add_or_update_lobby(&mut self, lobby: &Lobby) {
        if let Some(existing_lobby) = self.lobbies.iter_mut().find(|l| l.id == lobby.id) {
            let _ = std::mem::replace(existing_lobby, lobby.clone());
        } else {
            self.lobbies.push(lobby.clone());
        }
    }

    pub fn reset(&mut self) {
        self.client_id = None;
        self.current_lobby_id = None;
        self.lobbies.clear();
        self.clients.clear();
    }

    pub fn handle_message<F>(&mut self, text: String, handle_message: F) -> Option<RelayMessage>
        where F: FnOnce(LobbyClientID, &str) -> ()
    {

        let msg = match RelayMessage::deserialize_json(&text) {
            Ok(msg) => msg,
            Err(err) => {
                println!("client got error: {} when trying to deserialize message!", err);
                return None
            }
        };

        match msg {

            RelayMessage::ClientID(client_id) => { self.client_id(client_id); },

            RelayMessage::ActiveLobbies(ref lobbies) => { self.active_lobbies(&lobbies); },
            RelayMessage::ActivePlayers(ref players) => { self.active_players(&players); },

            RelayMessage::SuccessfullyJoinedLobby(lobby_id) => { self.successfully_joined_lobby(lobby_id); },
            RelayMessage::FailedToJoinLobby(lobby_id, ref reason) => { self.failed_to_join_lobby(lobby_id, reason); },

            RelayMessage::JoinedLobby(client_id) => { self.joined_lobby(client_id); },
            RelayMessage::UpdatedLobby(ref lobby) => { self.updated_lobby(lobby); },
            RelayMessage::LeftLobby(client_id) => { self.left_lobby(client_id); },

            RelayMessage::StartedLobby => {},
            RelayMessage::StoppedLobby => {},

            RelayMessage::Ping(from_client_id, to_client_id) => { self.ping(from_client_id, to_client_id); },
            RelayMessage::Pong(from_client_id, to_client_id) => { self.pong(from_client_id, to_client_id); },

            // when the game/lobby is running, these relevant
            RelayMessage::Message(client_id, ref text) => handle_message(client_id, text),

            // server only messages
            RelayMessage::QueryActiveLobbies => (),
            RelayMessage::QueryActivePlayers => (),
            RelayMessage::Register(_) => (),
            RelayMessage::CreateLobby(_) => (),
            RelayMessage::StartLobby => (),
            RelayMessage::StopLobby => (),
            RelayMessage::CloseLobby => (),
            RelayMessage::JoinLobby(_) => (),
            RelayMessage::LeaveLobby => ()

        };

        Some(msg)
        
    }

}
