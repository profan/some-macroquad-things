use lockstep::lobby::{LobbyID, LobbyClientID, RelayMessage, Lobby};
use nanoserde::{DeJson, SerJson};

use crate::network::NetworkClient;

pub trait RelayMessageExt {
    fn send_relay_message(&mut self, message: RelayMessage);
}

impl RelayMessageExt for NetworkClient {
    fn send_relay_message(&mut self, message: RelayMessage) {
        self.send(ewebsock::WsMessage::Text(message.serialize_json()));
    }
}

pub trait RelayCommandsExt {

    fn start_lobby(&mut self);
    fn stop_lobby(&mut self);
    fn leave_lobby(&mut self);

    fn create_new_lobby(&mut self);
    fn join_lobby(&mut self, lobby_id: LobbyID);
    fn query_active_state(&mut self);

    fn ping(&mut self, from_client_id: LobbyClientID, to_client_id: Option<LobbyClientID>);
    fn pong(&mut self, from_client_id: Option<LobbyClientID>, to_client_id: LobbyClientID);

}

impl RelayCommandsExt for NetworkClient {

    fn create_new_lobby(&mut self) {
        self.send_relay_message(RelayMessage::CreateLobby("hello_world".to_string()));
        self.query_active_state();
    }

    fn start_lobby(&mut self) {
        self.send_relay_message(RelayMessage::StartLobby);
    }

    fn stop_lobby(&mut self) {
        self.send_relay_message(RelayMessage::StopLobby);
        self.query_active_state();
    }

    fn leave_lobby(&mut self) {
        self.send_relay_message(RelayMessage::LeaveLobby);
        self.query_active_state();
    }

    fn join_lobby(&mut self, lobby_id: LobbyID) {
        self.send_relay_message(RelayMessage::JoinLobby(lobby_id));
        self.query_active_state();
    }

    fn query_active_state(&mut self) {
        self.send_relay_message(RelayMessage::QueryActivePlayers);
        self.send_relay_message(RelayMessage::QueryActiveLobbies);
    }

    fn ping(&mut self, from_client_id: LobbyClientID, to_client_id: Option<LobbyClientID>) {
        self.send_relay_message(RelayMessage::Ping(from_client_id, to_client_id));
    }

    fn pong(&mut self, from_client_id: Option<LobbyClientID>, to_client_id: LobbyClientID) {
        self.send_relay_message(RelayMessage::Pong(from_client_id, to_client_id));
    }

}