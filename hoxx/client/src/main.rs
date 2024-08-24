use std::collections::VecDeque;

use camera::GameCamera2D;
use drawing::draw_hex;
use hexx::Hex;
use hoxx_shared::{utils::{trace_hex_boundary}, ClientColor, ClientID, ClientMessage, GameState, HEX_SIZE, IS_RUNNING_LOCALLY, SERVER_ADDRESS, SERVER_INTERNAL_PORT};
use macroquad::{experimental::camera::mouse, prelude::*};
use nanoserde::{DeJson, SerJson};
use network::{ConnectionState, NetworkClient};
use utility::{draw_text_centered, screen_dimensions, AdjustHue, DebugText, TextPosition, WithAlpha};
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

struct HoxxClientState {
    claim_start_position: Option<Vec2>,
    queued_hex_claims: VecDeque<Vec2>,
    claim_cooldown: f32
}

impl HoxxClientState {

    pub fn new() -> HoxxClientState {
        HoxxClientState {
            claim_start_position: None,
            queued_hex_claims: VecDeque::new(),
            claim_cooldown: 0.0
        }
    }

    pub fn push_claim_world(&mut self, new_world_position: Vec2) {
        self.queued_hex_claims.push_back(new_world_position);
    }

    pub fn try_pop_claim_world(&mut self) -> Option<Vec2> {

        if self.claim_cooldown > 0.0 {
            return None;
        }

        self.queued_hex_claims.pop_front()

    }

    pub fn update(&mut self, net: &mut NetworkClient, dt: f32) {
        self.claim_cooldown = (self.claim_cooldown - dt).max(0.0);
    }

}

trait HoxxNetworkClient {
    fn send_claim_message(&mut self, world_x: i32, world_y: i32);
}

impl HoxxNetworkClient for NetworkClient {
    fn send_claim_message(&mut self, world_x: i32, world_y: i32) {
        self.send_text(ClientMessage::Claim { world_x, world_y }.serialize_json());
    }
}

struct HoxxClient {

    camera: GameCamera2D,
    debug_text: DebugText,
    state: GameState,
    net: NetworkClient,
    id: ClientID,

    // intermediate client state
    client_state: HoxxClientState,
    
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

            client_state: HoxxClientState::new(),
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
        let current_server_address = if IS_RUNNING_LOCALLY {
            format!("ws://{}:{}", SERVER_ADDRESS, SERVER_INTERNAL_PORT)
        } else {
            format!("wss://{}", SERVER_ADDRESS)
        };
        self.net.connect(&current_server_address)
    }

    pub fn disconnect(&mut self) -> bool {
        self.net.disconnect()
    }

    pub fn update(&mut self, dt: f32) {

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
            self.client_state.claim_start_position = Some(self.camera.mouse_world_position());
        }

        let was_claim_released = is_mouse_button_released(MouseButton::Left);
        if was_claim_released {

            if let Some(claim_start_position) = self.client_state.claim_start_position {

                let mouse_world_position = self.camera.mouse_world_position();
                let mouse_world_hex = self.state.world_to_hex(mouse_world_position.x as i32, mouse_world_position.y as i32);
                let source_world_hex = self.state.world_to_hex(claim_start_position.x as i32, claim_start_position.y as i32);

                for hex in source_world_hex.line_to(mouse_world_hex) {
                    let world_hex_position = self.state.hex_to_world(hex.x, hex.y);
                    self.client_state.push_claim_world(world_hex_position);
                }

                self.client_state.claim_start_position = None;

            }
        }

        self.client_state.update(&mut self.net, dt);

        if let Some(new_world_position_claim) = self.client_state.try_pop_claim_world() {
            self.state.set_world(new_world_position_claim.x as i32, new_world_position_claim.y as i32, *self.id);
            self.net.send_claim_message(new_world_position_claim.x as i32, new_world_position_claim.y as i32);
        }

        self.handle_network_messages();

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

    fn draw_hex_highlights(&self) {

        let (mouse_world_position, world_position_as_hex, hex_colour, hex_border_colour) = self.draw_current_hex_highlight();
        self.draw_hex_boundary(world_position_as_hex, mouse_world_position, hex_border_colour, hex_colour);
        self.draw_pending_hexes(hex_border_colour, hex_colour);
        
    }

    fn draw_current_hex_highlight(&self) -> (Vec2, Hex, Color, Color) {

        let mouse_world_position = self.camera.mouse_world_position();
        let world_position_as_hex = self.state.world_to_hex(mouse_world_position.x as i32, mouse_world_position.y as i32);
    
        let hex_colour = to_macroquad_color(self.state.get_client_colour(self.id).unwrap_or(ClientColor::white())).with_alpha(0.25);
        let hex_border_colour = hex_colour.darken(0.1);
    
        if let Some(start_position) = self.client_state.claim_start_position {
    
            let start_hex = self.state.world_to_hex(start_position.x as i32, start_position.y as i32);
            let end_hex = world_position_as_hex;
    
            for hex in start_hex.line_to(end_hex) {
    
                let hex_world_position = self.state.hex_to_world(hex.x, hex.y);
    
                draw_hex(
                    hex_world_position.x as f32, hex_world_position.y as f32,
                    hex_border_colour,
                    hex_colour
                );
    
            }
    
        } else {
    
            let hex_world_position = self.state.hex_to_world(world_position_as_hex.x, world_position_as_hex.y);
    
            draw_hex(
                hex_world_position.x as f32, hex_world_position.y as f32,
                hex_border_colour,
                hex_colour
            );
    
        }
        
        (mouse_world_position, world_position_as_hex, hex_colour, hex_border_colour)

    }
    
    fn draw_hex_boundary(&self, world_position_as_hex: Hex, mouse_world_position: Vec2, hex_border_colour: Color, hex_colour: Color) {

        let is_in_boundary_fn = |h: Hex| self.state.get_hex(h.x, h.y).and_then(|v| Some(ClientID { id: v })).unwrap_or(ClientID::INVALID) == self.id;
        if let Some(boundary) = trace_hex_boundary(world_position_as_hex, is_in_boundary_fn) {
    
            draw_text_centered(&format!("is loop: {}", boundary.is_loop()), mouse_world_position.x, mouse_world_position.y, 16.0, BLACK);
    
            for (idx, hex) in boundary.inner().into_iter().enumerate() {
    
                let hex_world_position = self.state.hex_to_world(hex.x, hex.y);
    
                draw_text_centered(&idx.to_string(), hex_world_position.x, hex_world_position.y, 16.0, GREEN);
    
                draw_hex(
                    hex_world_position.x, hex_world_position.y,
                    hex_border_colour.lighten(0.25),
                    hex_colour.lighten(0.25)
                );
    
            }
    
            for (idx, hex) in boundary.outer().into_iter().enumerate() {
    
                let hex_world_position = self.state.hex_to_world(hex.x, hex.y);
    
                draw_text_centered(&idx.to_string(), hex_world_position.x, hex_world_position.y, 16.0, RED);
    
                draw_hex(
                    hex_world_position.x, hex_world_position.y,
                    hex_border_colour.lighten(0.25),
                    hex_colour.lighten(0.25)
                );
    
            }
    
            if let Some(inner_hex) = boundary.hex_inside_boundary(|h| self.state.get_hex(h.x, h.y).unwrap_or(*ClientID::INVALID) != *self.id) {
    
                let hex_world_position = self.state.hex_to_world(inner_hex.x, inner_hex.y);
    
                draw_hex(
                    hex_world_position.x, hex_world_position.y,
                    hex_border_colour.darken(0.75),
                    hex_colour.darken(0.75)
                );
    
            }
    
        }

    }
    
    fn draw_pending_hexes(&self, hex_border_colour: Color, hex_colour: Color) {

        for pending_hex_world_position in &self.client_state.queued_hex_claims {
            draw_hex(
                pending_hex_world_position.x, pending_hex_world_position.y,
                hex_border_colour,
                hex_colour
            );
        }

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

        let world_hex_padding = vec2(HEX_SIZE, HEX_SIZE);
        let world_screen_top_left = self.camera.screen_to_world(vec2(0.0, 0.0)) - world_hex_padding;
        let world_screen_bottom_right = self.camera.screen_to_world(screen_dimensions()) + world_hex_padding;
        let world_screen_rect = Rect {
            x: world_screen_top_left.x,
            y: world_screen_top_left.y,
            w: world_screen_bottom_right.x - world_screen_top_left.x,
            h: world_screen_bottom_right.y - world_screen_top_left.y
        };
        
        for (&(x, y), &value) in self.state.get_hexes() {

            let colour = self.state.get_client_colour(ClientID { id: value }).unwrap_or(ClientColor::white());

            let hex_colour = to_macroquad_color(colour);
            let hex_border_colour = hex_colour.darken(0.1);
            let hex_world_position = self.state.hex_to_world(x, y);

            if world_screen_rect.contains(hex_world_position) == false {
                continue;
            }

            draw_hex(
                hex_world_position.x as f32, hex_world_position.y as f32,
                hex_border_colour,
                hex_colour
            );
            
        }

        self.draw_hex_highlights();
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
        
        self.debug_text.draw_text(format!("hex under mouse: {} (value: {:?})", hex_mouse_position.as_ivec2(), hex_under_mouse), TextPosition::TopLeft, BLACK);

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
    let is_connecting = client.connect();
    if is_connecting == false {
        error!("[hoxx-client] failed to attempt to connect to server!");
    } else {
        info!("[hoxx-client] connecting to server: {}", client.current_server_address());
    }

    loop {
        let dt = get_frame_time();
        client.update(dt);
        client.draw(dt);
        next_frame().await;
    }

}
