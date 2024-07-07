use std::{cell::RefCell, collections::HashMap, rc::Rc};
use game::HoxxGameState;
use hexx::Hex;
use hoxx_shared::{Client, ClientColor, ClientID, ClientMessage, ClientState, SERVER_INTERNAL_PORT};
use nanoserde::{DeJson, SerJson};

const IS_DEBUG: bool = false;

mod game;

struct Router {
    sender: ws::Sender,
    inner: Box<dyn ws::Handler>,
    server: Rc<RefCell<HoxxServer>>,
}

impl ws::Handler for Router {
    fn on_request(&mut self, req: &ws::Request) -> ws::Result<ws::Response> {

        // Clone the sender so that we can move it into the child handler
        let out = self.sender.clone();
        
        // Allocate a client on the server and associate it with our session
        let client_id = self.server.borrow_mut().spawn_client(self.sender.clone());

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

struct Session {
    ws: ws::Sender,
    id: ClientID,
    server: Rc<RefCell<HoxxServer>>
}

impl ws::Handler for Session {

    fn on_open(&mut self, _shake: ws::Handshake) -> ws::Result<()> {
        println!("[id: {:?}] connected!", self.ws.connection_id());
        Ok(())
    }

    fn on_message(&mut self, msg: ws::Message) -> ws::Result<()> {

        let Ok(text) = msg.as_text() else { return Ok(()); };

        match ClientMessage::deserialize_json(text) {
            Ok(msg) => match msg {

                // sent by clients to server :)
                ClientMessage::Claim { world_x, world_y } => self.server.borrow_mut().put_claim(self.id, world_x, world_y),

                // only sent by server to clients, never sent by clients to server
                ClientMessage::Update { .. } => (),
                ClientMessage::Join { .. } => ()

            },
            Err(err) => {
                println!("[id: {:?}]: sent invalid message, with error: {}", self.ws.connection_id(), err);
            }
        };
        
        Ok(())

    }

    fn on_close(&mut self, _code: ws::CloseCode, reason: &str) {
        
        self.server.borrow_mut().despawn_client(self.id);

        if reason.is_empty() == false {
            println!("[id: {}] disconnected with reason: {}!", self.ws.connection_id(), reason);
        } else {
            println!("[id: {}] disconnected!", self.ws.connection_id());
        }

    }

}

struct NotFound;

impl ws::Handler for NotFound {
    fn on_request(&mut self, req: &ws::Request) -> ws::Result<ws::Response> {
        let mut res = ws::Response::from_request(req)?;
        res.set_status(404);
        res.set_reason("Not Found");
        Ok(res)
    }
}

struct HoxxServer {
    current_client_id: ClientID,
    senders: HashMap<ClientID, ws::Sender>,
    clients: HashMap<ClientID, Client>,
    state: HoxxGameState,
    port: u16
}

impl HoxxServer {

    fn new() -> HoxxServer {
        HoxxServer {
            current_client_id: ClientID::INVALID + 1,
            senders: HashMap::new(),
            clients: HashMap::new(),
            state: HoxxGameState::new(),
            port: SERVER_INTERNAL_PORT
        }
    }

    pub fn start() {

        let new_hoxx_server = Rc::new(RefCell::new(HoxxServer::new()));

        // Listen on an address and call the closure for each connection
        if let Err(error) = ws::listen(format!("localhost:{}", new_hoxx_server.borrow().port), |out| {
            Router {
                sender: out,
                inner: Box::new(NotFound),
                server: new_hoxx_server.clone()
            }
        }) {
            println!("[hoxx-server] failed to create WebSocket due to {:?}, exiting!", error);
        }

    }

    fn send_message_to_client(&self, client_id: ClientID, message: &ClientMessage) {
        if self.senders.contains_key(&client_id) == false {
            println!("[hoxx-server] attempted to send message to client: {:?} which seems to have disconnected, ignoring message!", client_id);
            return;
        }

        let client_sender = &self.senders[&client_id];
        let _ = client_sender.send(message.serialize_json());
    }

    fn send_message_to_all_clients(&mut self, message: &ClientMessage) {
        for (&client_id, _client) in &self.clients {
            self.send_message_to_client(client_id, message);
        }
    }

    fn send_updated_game_state_to_all_clients(&mut self) {
        self.send_message_to_all_clients(&ClientMessage::Update { state: self.state.get_game_state().clone() });
    }

    fn send_join_message_to_client(&mut self, client_id: ClientID) {
        self.send_message_to_client(client_id, &ClientMessage::Join { id: client_id });
    }

    fn send_updated_game_state_to_client(&mut self, client_id: ClientID) {
        self.send_message_to_client(client_id, &ClientMessage::Update { state: self.state.get_game_state().clone() });
    }

    fn update_fill_state(&mut self, client_id: ClientID, x: i32, y: i32) {

        return;

        let hex_pos = self.state.get_game_state().world_to_hex(x, y);

        let initial_hex = Hex::new(hex_pos.x, hex_pos.y);
        let mut current_hex = initial_hex;
        let mut done = false;

        while current_hex != initial_hex || done == false {

            for neighbour_hex in current_hex.all_neighbors() {

                let v = self.state.get_claim_hex(neighbour_hex.x, neighbour_hex.y).unwrap_or(ClientID::INVALID);

                if v != client_id {
                    println!("found foreign hex: {:?}", neighbour_hex);
                    current_hex = neighbour_hex;
                }

                println!("walked hex: {:?}", neighbour_hex);
                
            }

            if current_hex == initial_hex {
                done = true;
            }

        }

    }

    fn put_claim(&mut self, client_id: ClientID, x: i32, y: i32) {
        self.state.put_claim_world(x, y, client_id);
        self.update_fill_state(client_id, x, y);
        self.send_updated_game_state_to_all_clients();
    }

    fn get_claim(&mut self, client_id: ClientID, x: i32, y: i32) -> Option<ClientID> {
        self.state.get_claim_world(x, y)
    }

    fn create_client_colour() -> ClientColor {

        let mixer = ClientColor { r: 1.0, g: 1.0, b: 1.0 };

        let random_r = rand::random::<f32>();
        let random_g = rand::random::<f32>();
        let random_b = rand::random::<f32>();

        let mixed_r = (random_r + mixer.r) / 2.0;
        let mixed_g = (random_g + mixer.g) / 2.0;
        let mixed_b = (random_b + mixer.b) / 2.0;

        ClientColor {
            r: mixed_r,
            g: mixed_g,
            b: mixed_b
        }
        
    }

    fn spawn_client(&mut self, sender: ws::Sender) -> ClientID {
        let created_client_id = self.create_client(sender);

        // set up client colour
        self.state.set_client_colour(created_client_id, Self::create_client_colour());

        // send welcome message to client with their id and the game state
        self.send_join_message_to_client(created_client_id);
        self.send_updated_game_state_to_client(created_client_id);

        created_client_id
    }

    fn create_client(&mut self, sender: ws::Sender) -> ClientID {
        let created_client_id = self.current_client_id;
        let created_client = Client { id: created_client_id, state: ClientState::Connected };
        self.clients.insert(created_client_id, created_client);
        self.senders.insert(created_client_id, sender);
        self.current_client_id += 1;
        created_client_id
    }

    fn remove_client(&mut self, client_id: ClientID) {
        self.clients.remove(&client_id);
        self.senders.remove(&client_id);
    }
    
    fn despawn_client(&mut self, client_id: ClientID) {
        self.remove_client(client_id);
    }

}

fn main() {
    HoxxServer::start();    
}
