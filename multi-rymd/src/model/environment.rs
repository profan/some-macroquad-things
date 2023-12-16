use hecs::{Entity, World};
use macroquad::math::Rect;
use macroquad::prelude::Vec2;

use super::{Transform, Sprite, DynamicBody, Health, create_default_kinematic_body};

pub fn create_asteroid(world: &mut World, position: Vec2, rotation: f32) -> Entity {

    let size = 64.0;
    let bounds = Rect {
        x: -size / 2.0,
        y: -size / 2.0,
        w: size,
        h: size
    };

    let body = DynamicBody {
        is_enabled: true,
        is_static: false,
        kinematic: create_default_kinematic_body(position, rotation),
        bounds
    };

    let health = Health::new(100);
    let transform = Transform::new(position, rotation, None);
    let sprite = Sprite::new("ASTEROID");
    
    world.spawn((transform, sprite, health, body))
    
}