use std::fmt::Display;
use std::ops::{Add, Deref};
use std::{collections::HashMap, ops::AddAssign};
use glam::{ivec2, vec2, IVec2, Vec2};
use hexx::{hex, Hex};
use hexx::HexLayout;
use nanoserde::{DeJson, SerJson};

pub const SERVER_ADDRESS: &str = "hoxx.prfn.se/ws";
pub const SERVER_INTERNAL_PORT: u16 = 25565;

pub const HEX_SIZE: f32 = 16.0;
pub const HEX_IS_VERTICAL: bool = false;

pub enum ClientState {
    Unconnected,
    Connected
}

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq, DeJson, SerJson)]
pub struct ClientID {
    pub id: i64
}

impl Deref for ClientID {
    type Target = i64;

    fn deref(&self) -> &Self::Target {
        &self.id
    }
}

impl Display for ClientID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if *self == Self::INVALID {
            f.write_str("INVALID")
        }
        else
        {
            f.write_fmt(format_args!("{}", self.id))
        }
    }
}

impl ClientID {
    pub const INVALID: ClientID = ClientID { id: 0 };
}

impl AddAssign<i64> for ClientID {
    fn add_assign(&mut self, rhs: i64) {
        self.id += rhs;
    }
}

impl Add<i64> for ClientID {
    type Output = Self;
    fn add(self, rhs: i64) -> ClientID {
        ClientID { id: self.id + rhs }
    }
}

pub struct Client {
    pub id: ClientID,
    pub state: ClientState
}

#[derive(Debug, Clone, SerJson, DeJson)]
pub enum ClientMessage {
    Join { id: ClientID },
    Claim { world_x: i32, world_y: i32 },
    Update { state: GameState }
}

#[derive(Debug, Copy, Clone, SerJson, DeJson)]
pub struct ClientColor {
    pub r: f32,
    pub g: f32,
    pub b: f32
}

impl ClientColor {

    pub fn white() -> ClientColor {
        ClientColor { 
            r: 1.0,
            g: 1.0,
            b: 1.0
        }
    }

}

#[derive(Debug, Clone, SerJson, DeJson)]
pub struct GameState {
    hexes: HashMap<(i32, i32), i64>,
    colours: HashMap<ClientID, ClientColor>,
    #[nserde(skip)]
    layout: Option<HexLayout>
}

impl GameState {

    fn get_default_hex_layout() -> HexLayout {
        HexLayout {
            hex_size: hexx::Vec2::new(HEX_SIZE, HEX_SIZE),
            orientation: if HEX_IS_VERTICAL { hexx::HexOrientation::Pointy } else { hexx::HexOrientation::Flat },
            invert_x: false,
            invert_y: false,
            ..Default::default()
        }
    }

    pub fn new() -> GameState {
        let layout = Self::get_default_hex_layout();
        GameState {
            hexes: HashMap::new(),
            colours: HashMap::new(),
            layout: Some(layout)
        }
    }

    pub fn update_state_from(&mut self, new_state: GameState) {
        *self = new_state;
        self.layout = Some(Self::get_default_hex_layout());
    }

    pub fn set_client_colour(&mut self, client_id: ClientID, client_colour: ClientColor) {
        self.colours.insert(client_id, client_colour);
    }

    pub fn get_client_colour(&self, client_id: ClientID) -> Option<ClientColor> {
        self.colours.get(&client_id).copied()
    }

    pub fn get_hexes(&self) -> &HashMap<(i32, i32), i64> {
        &self.hexes
    }

    pub fn get_hex_size(&self) -> Vec2 {
        let hex_size = self.layout.as_ref().unwrap().hex_size;
        vec2(hex_size.x, hex_size.y)
    }

    pub fn clamp_world_to_hex(&self, x: i32, y: i32) -> Vec2 {
        let hex_position = self.world_to_hex(x, y);
        self.hex_to_world(hex_position.x, hex_position.y)
    }

    pub fn hex_to_world(&self, x: i32, y: i32) -> Vec2 {
        let world_position: hexx::Vec2 = self.layout.as_ref().unwrap().hex_to_world_pos(hex(x, y));
        vec2(world_position.x, world_position.y)
    }

    pub fn world_to_hex(&self, x: i32, y: i32) -> IVec2 {
        let hex_coordinate: Hex = self.layout.as_ref().unwrap().world_pos_to_hex(hexx::Vec2::new(x as f32, y as f32));
        ivec2(hex_coordinate.x, hex_coordinate.y)
    }

    pub fn get_world(&self, x: i32, y: i32) -> Option<i64> {
        let hex_coordinate = self.world_to_hex(x, y);
        self.hexes.get(&(hex_coordinate.x, hex_coordinate.y)).copied()
    }

    pub fn set_world(&mut self, x: i32, y: i32, v: i64) {
        let hex_coordinate = self.world_to_hex(x, y);
        self.hexes.insert((hex_coordinate.x, hex_coordinate.y), v);
    }

    pub fn set_hex(&mut self, x: i32, y: i32, v: i64) {
        self.hexes.insert((x, y), v);
    }

    pub fn get_hex(&mut self, x: i32, y: i32) -> Option<i64> {
        self.hexes.get(&(x, y)).copied()
    }

}
