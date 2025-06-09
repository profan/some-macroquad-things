#[derive(Clone, Copy, PartialEq, Debug)]
pub enum ConnectionState {
    Connected,
    Connecting,
    Disconnected
}

pub struct NetworkClientSwitch {
    implementation: Option<Box<dyn NetworkClient>>
}

impl NetworkClientSwitch {
    pub fn new() -> NetworkClientSwitch {
        NetworkClientSwitch {
            implementation: None
        }
    }

    pub fn start_singleplayer(&mut self) {
        self.implementation = Some(Box::new(NetworkClientLocal::new()));
    }

    pub fn start_multiplayer(&mut self) {
        self.implementation = Some(Box::new(NetworkClientWebSocket::new()));
    }

    pub fn stop(&mut self) {
        self.implementation = None;
    }
}

impl NetworkClient for NetworkClientSwitch {
    fn is_connected(&self) -> bool {
        if let Some(client) = &self.implementation {
            client.is_connected()
        } else {
            false
        }
    }

    fn is_connecting(&self) -> bool {
        if let Some(client) = &self.implementation {
            client.is_connecting()
        } else {
            false
        }
    }

    fn is_disconnected(&self) -> bool {
        if let Some(client) = &self.implementation {
            client.is_disconnected()
        } else {
            false
        }
    }

    fn connected_host(&self) -> &str {
        if let Some(client) = &self.implementation {
            client.connected_host()
        } else {
            "INVALID"
        }
    }

    fn connection_state(&self) -> ConnectionState {
        if let Some(client) = &self.implementation {
            client.connection_state()
        } else {
            ConnectionState::Disconnected
        }
    }

    fn connect(&mut self, address: &str) -> bool {
        if let Some(client) = &mut self.implementation {
            client.connect(address)
        } else {
            false
        }
    }

    fn try_recv(&mut self) -> Option<ewebsock::WsEvent> {
        if let Some(client) = &mut self.implementation {
            client.try_recv()
        } else {
            None
        }
    }

    fn send_text(&mut self, message: String) {
        if let Some(client) = &mut self.implementation {
            client.send_text(message)
        }
    }

    fn send(&mut self, message: ewebsock::WsMessage) {
        if let Some(client) = &mut self.implementation {
            client.send(message)
        }
    }

    fn disconnect(&mut self) -> bool {
        if let Some(client) = &mut self.implementation {
            client.disconnect()
        } else {
            false
        }
    }
}

pub struct NetworkClientLocal {
    queued_messages: Vec<String>,
    state: ConnectionState
}

impl NetworkClientLocal {
    pub fn new() -> NetworkClientLocal {
        NetworkClientLocal {
            queued_messages: Vec::new(),
            state: ConnectionState::Disconnected
        }
    }
}

impl NetworkClient for NetworkClientLocal {

    fn is_connected(&self) -> bool {
        self.state == ConnectionState::Connected
    }

    fn is_connecting(&self) -> bool {
        self.state == ConnectionState::Connecting
    }

    fn is_disconnected(&self) -> bool {
        self.state == ConnectionState::Disconnected
    }

    fn connected_host(&self) -> &str {
        "localhost"
    }

    fn connection_state(&self) -> ConnectionState {
        ConnectionState::Connected
    }

    fn connect(&mut self, address: &str) -> bool {
        self.state == ConnectionState::Connected
    }

    fn try_recv(&mut self) -> Option<ewebsock::WsEvent> {
        None
    }

    fn send_text(&mut self, message: String) {
        
    }

    fn send(&mut self, message: ewebsock::WsMessage) {
        
    }

    fn disconnect(&mut self) -> bool {
        self.state = ConnectionState::Disconnected;
        true
    }

}

pub struct NetworkClientWebSocket {
    sender: Option<ewebsock::WsSender>,
    receiver: Option<ewebsock::WsReceiver>,
    state: ConnectionState,
    host: String
}

impl NetworkClientWebSocket {

    fn new() -> NetworkClientWebSocket {
        NetworkClientWebSocket {
            sender: None,
            receiver: None,
            state: ConnectionState::Disconnected,
            host: String::new()
        }
    }

}

pub trait NetworkClient {

     fn is_connected(&self) -> bool;

     fn is_connecting(&self) -> bool;

     fn is_disconnected(&self) -> bool;

     fn connected_host(&self) -> &str;

     fn connection_state(&self) -> ConnectionState;
    
     fn connect(&mut self, address: &str) -> bool;

     fn try_recv(&mut self) -> Option<ewebsock::WsEvent>;

     fn send_text(&mut self, message: String);

     fn send(&mut self, message: ewebsock::WsMessage);

     fn disconnect(&mut self) -> bool;

}

impl NetworkClient for NetworkClientWebSocket {

     fn is_connected(&self) -> bool {
        self.state == ConnectionState::Connected
    }

     fn is_connecting(&self) -> bool {
        self.state == ConnectionState::Connecting
    }

     fn is_disconnected(&self) -> bool {
        self.state == ConnectionState::Disconnected
    }

     fn connected_host(&self) -> &str {
        self.host.as_str()
    }

     fn connection_state(&self) -> ConnectionState {
        self.state
    }
    
     fn connect(&mut self, address: &str) -> bool {
        match ewebsock::connect(address) {
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

     fn try_recv(&mut self) -> Option<ewebsock::WsEvent> {

        if let Some(receiver) = &mut self.receiver {
            if let Some(message) = receiver.try_recv() {
                match &message {
                    ewebsock::WsEvent::Opened => {
                        self.state = ConnectionState::Connected
                    },
                    ewebsock::WsEvent::Error(_) => {
                        // NOTE: we currently just disconnect on any error, this should probably log later?
                        self.state = ConnectionState::Disconnected;
                        self.disconnect();
                    },
                    ewebsock::WsEvent::Closed => {
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

     fn send_text(&mut self, message: String) {
        self.send(ewebsock::WsMessage::Text(message));
    }

     fn send(&mut self, message: ewebsock::WsMessage) {
        if let Some(sender) = &mut self.sender {
            sender.send(message)
        }
    }

     fn disconnect(&mut self) -> bool {
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