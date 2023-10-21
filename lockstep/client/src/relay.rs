use lockstep::lobby::Lobby;
use lockstep::lobby::LobbyClient;
use lockstep::lobby::LobbyClientID;
use lockstep::lobby::LobbyID;
use lockstep::lobby::LobbyState;
use lockstep::lobby::RelayMessage;
use nanoserde::DeJson;

const IS_DEBUGGING: bool = false;

pub struct RelayClient {
    client_id: Option<LobbyClientID>,
    current_lobby_id: Option<LobbyID>,
    clients: Vec<LobbyClient>,
    lobbies: Vec<Lobby>
}

impl RelayClient {

    pub fn new() -> RelayClient {
        RelayClient {
            client_id: None,
            current_lobby_id: None,
            clients: Vec::new(),
            lobbies: Vec::new()
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
        println!("[RelayClient] got assigned a client id: {}", client_id);
        self.client_id = Some(client_id);
    }

    pub fn active_lobbies(&mut self, lobbies: &Vec<Lobby>) {
        println!("[RelayClient] got active lobbies response!");
        self.lobbies = lobbies.clone();
    }

    pub fn active_players(&mut self, players: &Vec<LobbyClient>) {
        println!("[RelayClient] got active player response!");
        self.clients = players.clone();
    }

    pub fn successfully_joined_lobby(&mut self, lobby_id: LobbyID) {
        println!("[RelayClient] successfully joined the lobby: {}", lobby_id);
        self.current_lobby_id = Some(lobby_id);
    }

    pub fn failed_to_join_lobby(&mut self, lobby_id: LobbyID, reason: &String) {
        println!("[RelayClient] failed to join the lobby: {} because: {}", lobby_id, reason);
    }

    pub fn joined_lobby(&mut self, client_id: LobbyClientID) {
        println!("[RelayClient] client with id: {} joined the lobby: {:?}", client_id, self.current_lobby_id);
        let current_lobby = self.lobbies.iter_mut().find(|lobby| lobby.id == self.current_lobby_id.unwrap()).unwrap();
        current_lobby.clients.push(client_id);
    }

    pub fn updated_lobby(&mut self, lobby: &Lobby) {
        println!("[RelayClient] got an update for the lobby: {}", lobby.id);
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
