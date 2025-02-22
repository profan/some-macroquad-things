use hecs::{Entity, World};
use macroquad::math::Rect;
use macroquad::prelude::Vec2;
use utility::Kinematic;

use super::{create_default_kinematic_body, DynamicBody, Health, ResourceSource, Sprite, Transform};

const ENV_COLLISION_MASK: u32 = 1 << 31;

pub fn create_asteroid(world: &mut World, position: Vec2, rotation: f32) -> Entity {

    let size = 64.0;
    let bounds = Rect {
        x: 0.0,
        y: 0.0,
        w: size,
        h: size
    };

    let asteroid_metal_amount = 1000.0;
    let asteroid_kinematic_body = Kinematic {
        ..create_default_kinematic_body(position, rotation)
    };

    let body = DynamicBody {
        is_enabled: true,
        is_static: false,
        kinematic: asteroid_kinematic_body,
        mask: ENV_COLLISION_MASK as u64,
        bounds
    };

    let health = Health::new(100.0);
    let transform = Transform::new(position, rotation, None);
    let resource_source = ResourceSource::new_finite_metal_source(asteroid_metal_amount);
    let sprite = Sprite::new("ASTEROID");
    
    world.spawn((transform, resource_source, sprite, health, body))
    
}