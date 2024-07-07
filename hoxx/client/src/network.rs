use ewebsock::Options;
use macroquad::logging::error;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum ConnectionState {
    Connected,
    Connecting,
    Disconnected
}

pub struct NetworkClient {
    sender: Option<ewebsock::WsSender>,
    receiver: Option<ewebsock::WsReceiver>,
    state: ConnectionState,
    host: String
}

impl NetworkClient {

    pub fn new() -> NetworkClient {
        NetworkClient {
            sender: None,
            receiver: None,
            state: ConnectionState::Disconnected,
            host: String::new()
        }
    }

    pub fn is_connected(&self) -> bool {
        self.state == ConnectionState::Connected
    }

    pub fn is_connecting(&self) -> bool {
        self.state == ConnectionState::Connecting
    }

    pub fn is_disconnected(&self) -> bool {
        self.state == ConnectionState::Disconnected
    }

    pub fn connected_host(&self) -> &str {
        self.host.as_str()
    }

    pub fn connection_state(&self) -> ConnectionState {
        self.state
    }
    
    pub fn connect(&mut self, address: &str) -> bool {
        match ewebsock::connect(address, Options::default()) {
            Ok((sender, receiver)) => {
                self.sender = Some(sender);
                self.receiver = Some(receiver);
                self.host = address.to_string();
                self.state = ConnectionState::Connecting;
                true
            },
            Err(_) => false
        }
    }

    pub fn try_recv(&mut self) -> Option<ewebsock::WsEvent> {

        if let Some(receiver) = &mut self.receiver {
            if let Some(message) = receiver.try_recv() {
                match &message {
                    ewebsock::WsEvent::Opened => {
                        self.state = ConnectionState::Connected
                    },
                    ewebsock::WsEvent::Error(error) => {
                        // NOTE: we currently just disconnect on any error, this should probably log later?
                        error!("[hoxx-client] got disconnected with error: {}", error);
                        self.state = ConnectionState::Disconnected;
                        self.disconnect();
                    },
                    ewebsock::WsEvent::Closed => {
                        error!("[hoxx-client] got disconnected by connection closing");
                        self.state = ConnectionState::Disconnected;
                        self.disconnect();
                    },
                    _ => ()
                };
                Some(message)
            } else {
                None
            }
        } else {
            None
        }

    }

    pub fn send_text(&mut self, message: String) {
        self.send(ewebsock::WsMessage::Text(message));
    }

    pub fn send(&mut self, message: ewebsock::WsMessage) {
        if let Some(sender) = &mut self.sender {
            sender.send(message)
        }
    }

    pub fn disconnect(&mut self) -> bool {
        if self.sender.is_some() {
            let _ = std::mem::replace(&mut self.sender, None);
            let _ = std::mem::replace(&mut self.receiver, None);
            self.state = ConnectionState::Disconnected;
            self.host.clear();
            true
        } else {
            false
        }
    }

}