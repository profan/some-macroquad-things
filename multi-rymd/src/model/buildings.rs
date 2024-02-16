use hecs::{Entity, World};
use macroquad::{math::{Vec2, Rect, vec2}, miniquad::KeyCode};

use crate::PlayerID;
use super::{Controller, Health, Sprite, Transform, DynamicBody, create_default_kinematic_body, Orderable, Cost, BlueprintIdentity};

pub use i32 as BlueprintID;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EntityState {
    Ghost,
    Destroyed,
    Constructed
}

#[derive(Debug, Clone)]
pub struct Building {
    pub state: EntityState
}

#[derive(Debug, Clone)]
pub struct Constructor {
    pub current_target: Option<Entity>,
    pub constructibles: Vec<BlueprintID>,
    pub build_range: i32,
    pub build_speed: i32,
    pub beam_offset: Vec2,
    pub can_assist: bool
}

impl Constructor {
    pub fn is_constructing(&self) -> bool {
        self.current_target.is_some()
    }

    pub fn has_blueprint(&self, id: BlueprintID) -> bool {
        self.constructibles.contains(&id)
    }
}

#[derive(Debug, Clone)]
pub struct Spawner {
    /// This position is a local offset from the position of the transform this is attached to, and is where units will spawn.
    pub position: Vec2
}

#[derive(Debug, Clone)]
pub struct Blueprint {
    pub id: BlueprintID,
    pub name: String,
    pub texture: String,
    pub shortcut: KeyCode,
    pub constructor: fn(&mut World, PlayerID, Vec2) -> Entity,
    pub is_building: bool,
    pub cost: Cost
}

pub fn create_solar_collector_blueprint() -> Blueprint {
    Blueprint {
        id: 0,
        shortcut: KeyCode::Key1,
        name: String::from("Solar Collector"),
        texture: String::from("SOLAR_COLLECTOR"),
        constructor: build_solar_collector,
        cost: Cost { metal: 25.0, energy: 25.0 },
        is_building: true
    }
}

pub fn create_shipyard_blueprint() -> Blueprint {
    Blueprint {
        id: 1,
        shortcut: KeyCode::Key2,
        name: String::from("Shipyard"),
        texture: String::from("SHIPYARD"),
        constructor: build_shipyard,
        cost: Cost { metal: 50.0, energy: 25.0 },
        is_building: true
    }
}

pub fn build_solar_collector(world: &mut World, owner: PlayerID, position: Vec2) -> Entity {

    let solar_collector_size = 64.0;
    let bounds = Rect { x: 0.0, y: 0.0, w: solar_collector_size, h: solar_collector_size };
    let is_enabled = true;
    let is_static = true;

    let kinematic = create_default_kinematic_body(position, 0.0);

    let full_solar_collector_health = 1000;
    let initial_solar_collector_health = 10;

    let controller = Controller { id: owner };
    let transform = Transform::new(position, 0.0, None);
    let blueprint_identity = BlueprintIdentity { blueprint_id: 0 };
    let health = Health::new_with_current_health(full_solar_collector_health, initial_solar_collector_health);
    let sprite = Sprite { texture: "SOLAR_COLLECTOR".to_string() };
    let dynamic_body = DynamicBody { is_enabled, is_static, bounds, kinematic };
    let state = EntityState::Ghost;

    world.spawn((controller, transform, blueprint_identity, health, sprite, dynamic_body, state))

}

pub fn build_shipyard(world: &mut World, owner: PlayerID, position: Vec2) -> Entity {

    let shipyard_size = 128.0;
    let bounds = Rect { x: shipyard_size / 4.0, y: 0.0, w: shipyard_size / 2.0, h: shipyard_size };
    let is_enabled = true;
    let is_static = true;

    let kinematic = create_default_kinematic_body(position, 0.0);

    let full_shipyard_health = 1000;
    let initial_shipyard_health = 10;

    let controller = Controller { id: owner };
    let transform = Transform::new(position, 0.0, None);
    let blueprint_identity = BlueprintIdentity { blueprint_id: 1 };
    let spawner = Spawner { position: vec2(-(shipyard_size / 5.0), 0.0) };
    let orderable = Orderable::new();
    let constructor = Constructor { current_target: None, constructibles: vec![2], build_range: shipyard_size as i32 / 2, build_speed: 100, beam_offset: -vec2(0.0, 8.0), can_assist: false };
    let health = Health::new_with_current_health(full_shipyard_health, initial_shipyard_health);
    let sprite = Sprite { texture: "SHIPYARD".to_string() };
    let dynamic_body = DynamicBody { is_enabled, is_static, bounds, kinematic};
    let state = EntityState::Ghost;

    world.spawn((controller, transform, blueprint_identity, spawner, orderable, constructor, health, sprite, dynamic_body, state))

}