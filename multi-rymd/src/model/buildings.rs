use hecs::{Entity, World};
use macroquad::{math::{Vec2, Rect}, miniquad::KeyCode};
use utility::Kinematic;

use crate::PlayerID;
use super::{Controller, Health, Sprite, Transform, Orderable, DynamicBody};

pub use i32 as BlueprintID;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BuildingState {
    Ghost,
    Destroyed,
    Constructed
}

#[derive(Debug, Clone)]
pub struct Building {
    pub state: BuildingState
}

#[derive(Debug, Clone)]
pub struct Constructor {
    pub constructibles: Vec<BlueprintID>,
    pub build_range: i32,
    pub build_speed: i32
}

#[derive(Debug, Clone)]
pub struct Blueprint {
    pub id: BlueprintID,
    pub name: String,
    pub texture: String,
    pub shortcut: KeyCode,
    pub constructor: fn(&mut World, PlayerID, Vec2) -> Entity
}

pub fn create_solar_collector_blueprint() -> Blueprint {
    Blueprint {
        id: 0,
        shortcut: KeyCode::Key1,
        name: String::from("Solar Collector"),
        texture: String::from("POWER_STATION"),
        constructor: build_solar_collector
    }
}

pub fn build_solar_collector(world: &mut World, owner: PlayerID, position: Vec2) -> Entity {

    let solar_collector_size = 64.0;
    let bounds = Rect { x: -(solar_collector_size / 2.0), y: -(solar_collector_size / 2.0), w: solar_collector_size, h: solar_collector_size };

    let kinematic = Kinematic {
        position: position,
        orientation: 0.0,
        velocity: Vec2::ZERO,
        angular_velocity: 0.0,
        friction_value: 0.975f32,
        mass: 1000.0
    };

    let full_solar_collector_health = 1000;
    let initial_solar_collector_health = 10;

    let controller = Controller { id: owner };
    let transform = Transform::new(position, 0.0, None);
    let building = Building { state: BuildingState::Ghost };
    let health = Health { full_health: full_solar_collector_health, current_health: initial_solar_collector_health };
    let sprite = Sprite { texture: "POWER_STATION".to_string() };
    let dynamic_body = DynamicBody { bounds, kinematic };
    let orderable = Orderable::new();

    world.spawn((controller, transform, building, health, sprite, dynamic_body, orderable))

}