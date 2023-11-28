use hecs::{Entity, World};
use macroquad::math::Vec2;
use crate::PlayerID;

use super::{Controller, Health, Sprite, Transform, Orderable};

#[derive(Debug, Clone)]
pub struct Constructor {
    pub constructibles: Vec<Blueprint>,
    pub build_range: i32,
    pub build_speed: i32
}

#[derive(Debug, Clone)]
pub struct Blueprint {
    pub name: String,
    pub constructor: fn(&mut World, PlayerID, Vec2) -> Entity
}

pub fn build_solar_collector(world: &mut World, owner: PlayerID, position: Vec2) -> Entity {

    let full_solar_collector_health = 1000;
    let initial_solar_collector_health = 10;

    let controller = Controller { id: owner };
    let transform = Transform::new(position, 0.0, None);
    let health = Health { full_health: full_solar_collector_health, health: initial_solar_collector_health };
    let sprite = Sprite { texture: "POWER_STATION".to_string() };
    let orderable = Orderable::new();

    world.spawn((controller, transform, health, sprite, orderable))

}