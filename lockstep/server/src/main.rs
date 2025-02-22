use std::cell::RefCell;
use std::collections::BTreeMap;

use std::rc::Rc;


use lockstep::lobby::DEFAULT_LOBBY_PORT;
use lockstep::lobby::Lobby;
use lockstep::lobby::LobbyClient;
use lockstep::lobby::LobbyClientID;
use lockstep::lobby::LobbyID;
use lockstep::lobby::LobbyState;
use lockstep::lobby::RelayMessage;

use nanoserde::{DeJson, SerJson};
struct Router {
    sender: ws::Sender,
    inner: Box<dyn ws::Handler>,
    server: Rc<RefCell<RelayServer>>,
}

impl ws::Handler for Router {
    fn on_request(&mut self, req: &ws::Request) -> ws::Result<ws::Response> {

        // Clone the sender so that we can move it into the child handler
        let out = self.sender.clone();
        
        // Allocate a client on the server and associate it with our session
        let client_id = self.server.borrow_mut().create_client(self.sender.clone());

        match req.resource() {
            "/" => self.inner = Box::new(Session { ws: out, id: client_id, server: self.server.clone() }),
            _ => (),
        }

        // Delegate to the child handler
        self.inner.on_request(req)
    }

    // Pass through any other methods that should be delegated to the child.

    fn on_shutdown(&mut self) {
        self.inner.on_shutdown()
    }

    fn on_open(&mut self, shake: ws::Handshake) -> ws::Result<()> {
        self.inner.on_open(shake)
    }

    fn on_message(&mut self, msg: ws::Message) -> ws::Result<()> {
        self.inner.on_message(msg)
    }

    fn on_close(&mut self, code: ws::CloseCode, reason: &str) {
        self.inner.on_close(code, reason)
    }

    fn on_error(&mut self, err: ws::Error) {
        self.inner.on_error(err);
    }

}

// This handler returns a 404 response to all handshake requests
struct NotFound;

impl ws::Handler for NotFound {
    fn on_request(&mut self, req: &ws::Request) -> ws::Result<ws::Response> {
        let mut res = ws::Response::from_request(req)?;
        res.set_status(404);
        res.set_reason("Not Found");
        Ok(res)
    }
}

struct Session {
    ws: ws::Sender,
    id: LobbyClientID,
    server: Rc<RefCell<RelayServer>>
}

impl ws::Handler for Session {

    fn on_open(&mut self, _shake: ws::Handshake) -> ws::Result<()> {
        println!("[id: {}] connected!", self.id);
        Ok(())
    }

    fn on_message(&mut self, msg: ws::Message) -> ws::Result<()> {

        let Ok(text) = msg.as_text() else { return Ok(()); };

        self.server.borrow_mut().handle_relay_message(self.id, text);
        
        Ok(())

    }

    fn on_close(&mut self, _code: ws::CloseCode, reason: &str) {
        self.server.borrow_mut().remove_client(self.id);
        if reason.is_empty() == false {
            println!("[id: {}] disconnected with reason: {}!", self.ws.connection_id(), reason);
        } else {
            println!("[id: {}] disconnected!", self.ws.connection_id());
        }
    }

}

struct RelayServer {

    current_lobby_id: LobbyID,
    current_client_id: LobbyClientID,
    senders: BTreeMap<LobbyClientID, ws::Sender>,
    clients: BTreeMap<LobbyClientID, LobbyClient>,
    lobbies: BTreeMap<LobbyID, Lobby>,
    port: u16

}

impl RelayServer {

    fn new() -> RelayServer {
        RelayServer {
            current_lobby_id: 0,
            current_client_id: 0,
            senders: BTreeMap::new(),
            clients: BTreeMap::new(), 
            lobbies: BTreeMap::new(),
            port: DEFAULT_LOBBY_PORT
        }
    }

    fn handle_relay_message(&mut self, sender_id: LobbyClientID, message_text: &str) {

        match RelayMessage::deserialize_json(message_text) {
            Ok(msg) => match msg {

                // client management on the relay server
                RelayMessage::Register(name) => { self.register_client_nickname(sender_id, name); },

                // messaages for lobby management for the clients
                RelayMessage::CreateLobby(name) => { self.create_lobby(sender_id, name); },
                RelayMessage::StartLobby => { self.start_lobby_with_client_id(sender_id); },
                RelayMessage::StopLobby => { self.stop_lobby_with_client_id(sender_id); },
                RelayMessage::CloseLobby => { self.close_lobby_with_client_id(sender_id); },
                RelayMessage::LeaveLobby => { self.leave_lobby(sender_id); },
                RelayMessage::JoinLobby(lobby_id) => { self.join_lobby(lobby_id, sender_id); },

                // ping/pong messages between clients
                RelayMessage::Ping(from_client_id, to_client_id) => { self.ping(from_client_id, to_client_id); },
                RelayMessage::Pong(from_client_id, to_client_id) => { self.pong(from_client_id, to_client_id); },

                // lobby state update message for clients
                RelayMessage::PushLobbyData(data) => { self.send_update_data_to_clients_lobby(sender_id, data); },

                // messages for passing game data, external to the relay server (to be forwarded to all in the same lobby)
                RelayMessage::Message(peer_id, text) => { self.send_message_to_clients_lobby(sender_id, RelayMessage::Message(peer_id, text)); },

                // messages for querying relay server/lobby state
                RelayMessage::QueryActiveLobbies => { self.query_active_lobbies(sender_id); },
                RelayMessage::QueryActivePlayers => { self.query_active_players(sender_id); },

                // messages the server may send, so we don't care here :>
                RelayMessage::ClientID(_) => (),
                RelayMessage::LeftLobby(_) => (),
                RelayMessage::JoinedLobby(_) => (),
                RelayMessage::UpdatedLobby(_) => (),
                RelayMessage::StartedLobby => (),
                RelayMessage::StoppedLobby => (),
                RelayMessage::SuccessfullyJoinedLobby(_) => (),
                RelayMessage::FailedToJoinLobby(_, _) => (),
                RelayMessage::ActiveLobbies(_) => (),
                RelayMessage::ActivePlayers(_) => (),
                RelayMessage::Boss(_) => (),

            },
            Err(err) => {
                println!("[id: {}]: sent invalid message, with error: {}", sender_id, err);
            }
        };

    }

    pub fn start() {

        let new_relay_server = Rc::new(RefCell::new(RelayServer::new()));

        // Listen on an address and call the closure for each connection
        if let Err(error) = ws::listen(format!("0.0.0.0:{}", new_relay_server.borrow().port), |out| {
            Router {
                sender: out,
                inner: Box::new(NotFound),
                server: new_relay_server.clone()
            }
        }) {
            // Inform the user of failure
            println!("[lockstep] failed to create WebSocket due to {:?}", error);
        }

    }

    pub fn send_client_id(&self, client_id: LobbyClientID) {
        self.send_message_to_client(client_id, RelayMessage::ClientID(client_id));
    }

    pub fn get_client_lobby(&self, client_id: LobbyClientID) -> Option<LobbyID> {
        self.lobbies.iter()
            .find(|(_, lobby)| lobby.clients.contains(&client_id))
            .and_then(|(lobby_id, _)| Some(*lobby_id))
    }

    pub fn is_client_in_lobby(&self, client_id: LobbyClientID) -> bool {
        self.get_client_lobby(client_id).is_some()
    }

    pub fn register_client_nickname(&mut self, client_id: LobbyClientID, nick: String) {
        if let Some(client) = self.clients.get_mut(&client_id) {
            client.name = nick;
        }
    }

    pub fn create_new_unique_client_name(&mut self) -> String {

        let initial_unique_name = "rts_fan";

        let mut current_unique_name = initial_unique_name.to_owned();
        let mut current_unique_name_id = 1;

        for (_, client) in &self.clients {
            if client.name == current_unique_name {
                current_unique_name = format!("{}{}", initial_unique_name, current_unique_name_id);
                current_unique_name_id += 1;
            }
        }

        return current_unique_name;

    }

    pub fn create_client(&mut self, sender: ws::Sender) -> LobbyClientID {
        let created_client_id = self.current_client_id;
        let created_client_name = self.create_new_unique_client_name();
        self.clients.insert(created_client_id, LobbyClient { id: created_client_id, name: created_client_name });
        self.senders.insert(created_client_id, sender);
        self.send_client_id(created_client_id);
        self.current_client_id += 1;
        created_client_id
    }

    pub fn close_lobby_if_empty(&mut self, lobby_id: LobbyID) -> bool {
        if let Some(lobby) = self.lobbies.get(&lobby_id) {
            let lobby_player_count = lobby.clients.len();
            if lobby_player_count == 0 {
                self.close_lobby(lobby_id);
            }
            true
        } else {
            false
        }
    }
 
    pub fn remove_client(&mut self, client_id: LobbyClientID) {
        if let Some(client_lobby_id) = self.get_client_lobby(client_id) {
            self.client_left_lobby(client_lobby_id, client_id);
        }
        self.clients.remove(&client_id);
        self.senders.remove(&client_id);
        println!("cleaned up client: {}", client_id);
    }

    pub fn create_lobby(&mut self, client_id: LobbyClientID, name: String) -> LobbyID {
        let created_lobby_id = self.current_lobby_id;
        self.lobbies.insert(created_lobby_id, Lobby::new(created_lobby_id, client_id, name));

        // #FIXME: this is terrible :D
        self.update_lobby(created_lobby_id, client_id);
        self.join_lobby(created_lobby_id, client_id);
        self.update_lobby(created_lobby_id, client_id);
        
        self.current_lobby_id += 1;
        created_lobby_id
    }

    pub fn start_lobby(&mut self, lobby_id: LobbyID) {
        if let Some(lobby) = self.lobbies.get_mut(&lobby_id) {
            lobby.state = LobbyState::Running;
        } else {
            // tried to start a lobby which does not exist!
        }
    }

    pub fn start_lobby_with_client_id(&mut self, client_id: LobbyClientID) {
        if let Some(lobby_id) = self.get_client_lobby(client_id) {
            self.start_lobby(lobby_id);
            self.update_lobby_for_all(lobby_id);
            self.send_message_to_lobby(lobby_id, RelayMessage::StartedLobby);
            self.update_lobby_for_all(lobby_id);
        } else {
            // can't start nonexistent lobby!
        }
    }

    pub fn stop_lobby_with_client_id(&mut self, client_id: LobbyClientID) {
        if let Some(lobby_id) = self.get_client_lobby(client_id) {
            self.stop_lobby(lobby_id);
            self.update_lobby_for_all(lobby_id);
            self.send_message_to_lobby(lobby_id, RelayMessage::StoppedLobby);
            self.update_lobby_for_all(lobby_id);
        } else {
            // can't stop nonexistent lobby!
        }
    }

    pub fn close_lobby_with_client_id(&mut self, client_id: LobbyClientID) {
        if let Some(lobby_id) = self.get_client_lobby(client_id) {
            self.close_lobby(lobby_id);
        } else {
            // can't stop nonexistent lobby!
        }
    }

    pub fn stop_lobby(&mut self, lobby_id: LobbyID) {
        if let Some(lobby) = self.lobbies.get_mut(&lobby_id) {
            lobby.state = LobbyState::Open;
        } else {
            // can't stop nonexistent lobby!
        }
    }

    pub fn close_lobby(&mut self, lobby_id: LobbyID) {
        self.lobbies.remove(&lobby_id);
    }

    pub fn leave_lobby(&mut self, client_id: LobbyClientID) {
        if let Some(lobby_id) = self.get_client_lobby(client_id) {

            self.client_left_lobby(lobby_id, client_id);

            let leaving_message = RelayMessage::LeftLobby(client_id);
            self.send_message_to_client(client_id, leaving_message.clone());

        } else {
            // can't leave a lobby you aren't in!
        }
    }

    pub fn update_lobby(&mut self, lobby_id: LobbyID, client_id: LobbyClientID) {
        let lobby = &self.lobbies[&lobby_id];
        self.send_message_to_client(client_id, RelayMessage::UpdatedLobby(lobby.clone()));
    }

    pub fn update_lobby_for_all(&mut self, lobby_id: LobbyID) {
        let lobby = &self.lobbies[&lobby_id];
        self.send_message_to_lobby(lobby_id, RelayMessage::UpdatedLobby(lobby.clone()));
    }

    fn client_joined_lobby(&mut self, lobby_id: LobbyID, client_id: LobbyClientID) {

        let lobby = self.lobbies.get_mut(&lobby_id).unwrap();
        let lobby_id = lobby.id;

        lobby.clients.push(client_id);
        let boss_changed = lobby.figure_out_lobby_boss();
        let new_lobby_boss = lobby.boss;
        if boss_changed {
            println!("the boss of lobby: {} changed to client: {}", lobby_id, lobby.boss);
        }

        let cloned_lobby = lobby.clone();
        self.send_message_to_client(client_id, RelayMessage::SuccessfullyJoinedLobby(lobby_id));
        self.send_message_to_clients_lobby(client_id, RelayMessage::UpdatedLobby(cloned_lobby));
        
        let joining_message = RelayMessage::JoinedLobby(client_id);
        self.send_message_to_clients_lobby(client_id, joining_message.clone());

        if boss_changed {
            self.send_message_to_clients_lobby(client_id, RelayMessage::Boss(new_lobby_boss));
        }

    }

    fn client_left_lobby(&mut self, lobby_id: LobbyID, client_id: LobbyClientID) {

        let lobby = self.lobbies.get_mut(&lobby_id).unwrap();
        let lobby_id = lobby.id;

        lobby.clients.retain(|id| client_id != *id);
        let boss_changed = lobby.figure_out_lobby_boss();
        let new_lobby_boss = lobby.boss;
        if boss_changed {
            println!("the boss of lobby: {} changed to client: {}", lobby_id, lobby.boss);
            self.send_message_to_clients_lobby(client_id, RelayMessage::Boss(new_lobby_boss));
        }

        let leaving_message = RelayMessage::LeftLobby(client_id);
        self.send_message_to_lobby(lobby_id, leaving_message.clone());
        self.close_lobby_if_empty(lobby_id);

    }

    pub fn join_lobby(&mut self, lobby_id: LobbyID, client_id: LobbyClientID) {
        if let Some(lobby) = self.lobbies.get(&lobby_id) {

            if lobby.state == LobbyState::Running {
                self.send_message_to_client(client_id, RelayMessage::FailedToJoinLobby(lobby_id, "lobby is already running! currently you cannot join running lobbies, sorry!".to_string()));
                return;
            }

            self.client_joined_lobby(lobby_id, client_id);
 
        } else {
            self.send_message_to_client(client_id, RelayMessage::FailedToJoinLobby(lobby_id, "lobby does not exist!".to_string()));
        }
    }

    pub fn ping(&mut self, from_client_id: LobbyClientID, to_client_id: Option<LobbyClientID>) {

        if let Some(to_client_id) = to_client_id {

            if self.senders.contains_key(&to_client_id) == false {
                return;
            }

            // asking another client :)
            self.send_message_to_client(to_client_id, RelayMessage::Ping(from_client_id, Some(to_client_id)));

        } else {
            
            // asking the server!
            self.pong(None, from_client_id);

        }

    }

    pub fn pong(&mut self, from_client_id: Option<LobbyClientID>, to_client_id: LobbyClientID) {

        if self.senders.contains_key(&to_client_id) == false {
            return;
        }

        if let Some(from_client_id) = from_client_id {

            // forward client pong to actual target client
            self.send_message_to_client(to_client_id, RelayMessage::Pong(Some(from_client_id), to_client_id));

        } else {

            // respond to client with pong :)
            self.send_message_to_client(to_client_id, RelayMessage::Pong(None, to_client_id));

        }

    }

    pub fn send_message_to_client(&self, client_id: LobbyClientID, message: RelayMessage) {
        if self.senders.contains_key(&client_id) == false {
            println!("attempted to send message to client: {} which seems to have disconnected, ignoring message!", client_id);
            return;
        }

        let client_sender = &self.senders[&client_id];
        let _ = client_sender.send(message.serialize_json());
    }

    pub fn send_message_to_lobby(&self, lobby_id: LobbyID, message: RelayMessage) {
        let msg = ws::Message::Text(message.serialize_json());
        if let Some(lobby) = self.lobbies.get(&lobby_id) {      
            for client_id in &lobby.clients {
                let client_sender = &self.senders[&client_id];
                let _ = client_sender.send(msg.clone()); // #FIXME: should we handle this potential error at all?
            }
        } else {
            // tried to send a message without being in a lobby, probably an error?
        }
    }

    pub fn send_update_data_to_lobby(&mut self, lobby_id: LobbyID, state: String) {

        if let Some(lobby) = &mut self.lobbies.get_mut(&lobby_id) {
            lobby.data = state;
            self.update_lobby_for_all(lobby_id);
        } else {
            // tried to update data for a non-existent lobby, probably an error?
        }

    }

    pub fn send_message_to_clients_lobby(&self, client_id: LobbyClientID, message: RelayMessage) {
        if let Some(lobby_id) = self.get_client_lobby(client_id) {      
            self.send_message_to_lobby(lobby_id, message);
        } else {
            // tried to send a message without being in a lobby, probably an error?
        }
    }

    pub fn send_update_data_to_clients_lobby(&mut self, client_id: LobbyClientID, data: String) {
        if let Some(lobby_id) = self.get_client_lobby(client_id) {      
            self.send_update_data_to_lobby(lobby_id, data);
        } else {
            // tried to send a message without being in a lobby, probably an error?
        }
    }

    pub fn query_active_lobbies(&self, client_id: LobbyClientID) {
        let active_lobbies = self.lobbies.iter()
            .map(|(_lobby_id, lobby)| lobby.clone())
            .collect();

        self.send_message_to_client(client_id, RelayMessage::ActiveLobbies(active_lobbies));
    }

    pub fn query_active_players(&self, client_id: LobbyClientID) {
        let active_clients = self.clients.iter()
            .map(|(_client_id, client)| client.clone())
            .collect();

        self.send_message_to_client(client_id, RelayMessage::ActivePlayers(active_clients));
    }

}

fn main() {
    RelayServer::start();
}