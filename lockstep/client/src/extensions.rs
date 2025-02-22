use lockstep::lobby::{LobbyID, LobbyClientID, RelayMessage};
use nanoserde::SerJson;

use crate::{network::NetworkClient, relay::RelayClient};

pub trait RelayMessageExt {
    fn send_relay_message(&mut self, message: RelayMessage);
}

impl RelayMessageExt for dyn NetworkClient {
    fn send_relay_message(&mut self, message: RelayMessage) {
        self.send(ewebsock::WsMessage::Text(message.serialize_json()));
    }
}

pub trait RelayCommandsExt {

    fn start_lobby(&self);
    fn stop_lobby(&self);
    fn leave_lobby(&self);

    fn create_new_lobby(&self);
    fn join_lobby(&self, lobby_id: LobbyID);
    fn query_active_state(&self);

    fn ping(&self, from_client_id: LobbyClientID, to_client_id: Option<LobbyClientID>);
    fn pong(&self, from_client_id: Option<LobbyClientID>, to_client_id: LobbyClientID);

    fn send_lobby_data(&self, lobby_data: String);

}

impl RelayCommandsExt for RelayClient {

    fn create_new_lobby(&self) {
        self.send_relay_message(RelayMessage::CreateLobby("hello_world".to_string()));
        self.query_active_state();
    }

    fn start_lobby(&self) {
        self.send_relay_message(RelayMessage::StartLobby);
    }

    fn stop_lobby(&self) {
        self.send_relay_message(RelayMessage::StopLobby);
        self.query_active_state();
    }

    fn leave_lobby(&self) {
        self.send_relay_message(RelayMessage::LeaveLobby);
        self.query_active_state();
    }

    fn join_lobby(&self, lobby_id: LobbyID) {
        self.send_relay_message(RelayMessage::JoinLobby(lobby_id));
        self.query_active_state();
    }

    fn query_active_state(&self) {
        self.send_relay_message(RelayMessage::QueryActivePlayers);
        self.send_relay_message(RelayMessage::QueryActiveLobbies);
    }

    fn ping(&self, from_client_id: LobbyClientID, to_client_id: Option<LobbyClientID>) {
        self.send_relay_message(RelayMessage::Ping(from_client_id, to_client_id));
    }

    fn pong(&self, from_client_id: Option<LobbyClientID>, to_client_id: LobbyClientID) {
        self.send_relay_message(RelayMessage::Pong(from_client_id, to_client_id));
    }

    fn send_lobby_data(&self, lobby_data: String) {
        self.send_relay_message(RelayMessage::PushLobbyData(lobby_data));
    }

}