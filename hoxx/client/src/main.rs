use camera::GameCamera2D;
use drawing::draw_hex;
use hoxx_shared::{ClientColor, ClientID, ClientMessage, GameState, HEX_SIZE, SERVER_ADDRESS, SERVER_PORT};
use macroquad::prelude::*;
use nanoserde::{DeJson, SerJson};
use network::{ConnectionState, NetworkClient};
use utility::{draw_text_centered, AdjustHue, DebugText, TextPosition, WithAlpha};
use utils::to_macroquad_color;

mod camera;
mod drawing;
mod network;
mod utils;

struct HoxxClientDebug {
    should_draw_coordinates: bool
}

impl HoxxClientDebug {
    pub fn new() -> HoxxClientDebug {
        HoxxClientDebug {
            should_draw_coordinates: false
        }
    }
}

struct HoxxClient {

    camera: GameCamera2D,
    debug_text: DebugText,
    state: GameState,
    net: NetworkClient,
    id: ClientID,
    
    // ... debug stuff? :D
    debug_state: HoxxClientDebug
}

impl HoxxClient {

    pub fn new() -> HoxxClient {
        HoxxClient {
            camera: GameCamera2D::new(),
            debug_text: DebugText::new(),
            state: GameState::new(),
            net: NetworkClient::new(),
            id: ClientID::INVALID,

            debug_state: HoxxClientDebug::new()
        }
    }

    pub fn current_server_address(&self) -> &str {
        self.net.connected_host()
    }

    pub fn connection_state(&self) -> ConnectionState {
        self.net.connection_state()
    }

    pub fn connect(&mut self) -> bool {
        let combined_server_address = format!("wss://{}:{}", SERVER_ADDRESS, SERVER_PORT);
        self.net.connect(&combined_server_address)
    }

    pub fn disconnect(&mut self) -> bool {
        self.net.disconnect()
    }

    pub fn update(&mut self) {

        let was_toggle_coords_pressed = is_key_pressed(KeyCode::T);
        if was_toggle_coords_pressed {
            self.debug_state.should_draw_coordinates = !self.debug_state.should_draw_coordinates;
        }

        let was_reset_camera_pressed = is_key_pressed(KeyCode::R);
        if was_reset_camera_pressed {
            self.camera.move_camera_to_position(Vec2::ZERO);
        }

        let was_claim_pressed = is_mouse_button_pressed(MouseButton::Left);
        if was_claim_pressed {
            let mouse_world_position = self.camera.mouse_world_position();
            self.send_claim_message(mouse_world_position.x as i32, mouse_world_position.y as i32);
        }

        self.handle_network_messages();

    }

    fn send_claim_message(&mut self, world_x: i32, world_y: i32) {
        self.net.send_text(ClientMessage::Claim { world_x, world_y }.serialize_json());
    }

    fn handle_message(&mut self, message: &ClientMessage) {
        match message {
            // never sent by server to client
            ClientMessage::Claim { .. } => (),

            // sent by server to client
            ClientMessage::Update { state } => self.state.update_state_from(state.clone()),
            ClientMessage::Join { id } => self.id = *id
        }
    }

    fn handle_network_messages(&mut self) {
        match self.net.try_recv() {
            Some(msg) => {
                match msg {
                    ewebsock::WsEvent::Opened => {
                        // some nice initialization maybe?
                    },
                    ewebsock::WsEvent::Message(ewebsock::WsMessage::Text(text)) => {
                        // handle some messages, paint some hexes! do some stuff!
                        match ClientMessage::deserialize_json(&text) {
                            Ok(client_message) => {
                                self.handle_message(&client_message);
                            },
                            Err(error) => {
                                // received some sort of invalid payload, ignore it?
                                error!("[hoxx-client] got invalid payload with error: {}, ignoring message!", error);
                            },
                        }
                    },
                    ewebsock::WsEvent::Closed | ewebsock::WsEvent::Error(_) => {
                        // nuke the game state? :D ... or do something else? show a nice message?
                    },
                    _ => ()
                }
            },
            None => (),
        }
    }

    fn draw_hex_highlight(&self) {

        let mouse_world_position = self.camera.mouse_world_position();
        let world_position_as_hex = self.state.world_to_hex(mouse_world_position.x as i32, mouse_world_position.y as i32);
        let hex_world_position = self.state.hex_to_world(world_position_as_hex.x, world_position_as_hex.y);

        let hex_value = self.state.get_world(world_position_as_hex.x, world_position_as_hex.y).unwrap_or(*self.id);
        let hex_colour = to_macroquad_color(self.state.get_client_colour(ClientID { id: hex_value }).unwrap_or(ClientColor::white())).with_alpha(0.25);
        let hex_border_colour = hex_colour.darken(0.1);

        draw_hex(
            hex_world_position.x as f32, hex_world_position.y as f32,
            hex_border_colour,
            hex_colour
        );
        
    }
    
    fn draw_hex_coordinates(&self) {

        let screen_size_x = screen_width();
        let screen_size_y = screen_height();

        if self.debug_state.should_draw_coordinates {

            for x in 0..(screen_size_x / HEX_SIZE) as i32 {
                for y in 0..(screen_size_y / HEX_SIZE) as i32 {

                    let screen_position_in_world = self.camera.screen_to_world((vec2(x as f32, y as f32) * HEX_SIZE));
                    let clamped_world_position = self.state.clamp_world_to_hex(screen_position_in_world.x as i32, screen_position_in_world.y as i32);
                    let offset_hex_position_of_world_position = vec2(clamped_world_position.x, clamped_world_position.y) + vec2(0.0, HEX_SIZE / 2.0);
                    let hex_position_of_world_position = self.state.world_to_hex(clamped_world_position.x as i32, clamped_world_position.y as i32);

                    draw_text_centered(
                        &format!("{},{}", hex_position_of_world_position.x, hex_position_of_world_position.y),
                        offset_hex_position_of_world_position.x as f32,
                        offset_hex_position_of_world_position.y as f32,
                        16.0,
                        BLACK
                    );

                }
            }

        }

    }

    fn draw_game_state(&self) {

        self.draw_hex_highlight();
        
        for (&(x, y), &value) in self.state.get_hexes() {

            let colour = self.state.get_client_colour(ClientID { id: value }).unwrap_or(ClientColor::white());

            let hex_colour = to_macroquad_color(colour);
            let hex_border_colour = hex_colour.darken(0.1);
            let hex_world_position = self.state.hex_to_world(x, y);

            draw_hex(
                hex_world_position.x as f32, hex_world_position.y as f32,
                hex_border_colour,
                hex_colour
            );
            
        }

        self.draw_hex_coordinates();

    }

    fn draw_debug_state(&mut self) {

        self.draw_network_debug_state();
    
        self.debug_text.skip_line(TextPosition::TopLeft);
    
        self.draw_game_debug_state();
    
    }

    fn draw_game_debug_state(&mut self) {

        self.debug_text.draw_text(format!("camera position: {} (r to reset)", self.camera.world_position()), TextPosition::TopLeft, BLACK);
            
        let mouse_world_position = self.camera.mouse_world_position();
        let hex_mouse_position = self.state.world_to_hex(mouse_world_position.x as i32, mouse_world_position.y as i32);
        let hex_under_mouse = self.state.get_world(mouse_world_position.x as i32, mouse_world_position.y as i32);
        
        self.debug_text.draw_text(format!("hex under mouse: {} (value: {:?})", hex_mouse_position, hex_under_mouse), TextPosition::TopLeft, BLACK);

    }
    
    fn draw_network_debug_state(&mut self) {
    
        self.debug_text.draw_text(format!("connection state: {:?}", self.connection_state()), TextPosition::TopLeft, BLACK);
    
        if self.current_server_address().is_empty() == false {
            self.debug_text.draw_text(format!("current host: {}", self.current_server_address()), TextPosition::TopLeft, BLACK);
        } else {
            self.debug_text.draw_text("no current host!", utility::TextPosition::TopLeft, BLACK);
        }
    
        if self.connection_state() == ConnectionState::Connected {
            self.debug_text.draw_text(format!("client id: {}", self.id), TextPosition::TopLeft, BLACK);
        }
    
    }
    
    pub fn draw(&mut self, dt: f32) {

        clear_background(WHITE);

        self.camera.tick(dt);

        self.camera.push();
        self.draw_game_state();
        self.camera.pop();

        self.draw_debug_state();
        self.debug_text.new_frame();

    }

}

#[macroquad::main("hoxx-client")]
async fn main() {

    let mut client = HoxxClient::new();
    client.connect();

    loop {
        let dt = get_frame_time();
        client.update();
        client.draw(dt);
        next_frame().await;
    }

}
