use macroquad::prelude::Vec2;
use hecs::{CommandBuffer, Entity, World};
use utility::AsAngle;

use super::{AnimatedSprite, Impact, Effect, RymdGameModel, Transform}; 

fn create_explosion_effect(position: Vec2) -> (Transform, AnimatedSprite, Effect) {

    let lifetime = 0.5;
    let transform = Transform::new(position, 0.0, None);
    let sprite = AnimatedSprite { texture: "EXPLOSION".to_string(), current_frame: 0, h_frames: 15 };
    let effect = Effect::new(lifetime);

    (transform, sprite, effect)

}

fn create_muzzle_flash_effect(position: Vec2, direction: Vec2) -> (Transform, AnimatedSprite, Effect) {

    let lifetime = 0.5;
    let transform = Transform::new(position, -direction.as_angle(), None);
    let sprite = AnimatedSprite { texture: "MUZZLE_FLASH".to_string(), current_frame: 0, h_frames: 5 };
    let effect = Effect::new(lifetime);

    (transform, sprite, effect)

}

fn create_impact_effect(position: Vec2, direction: Vec2) -> (Transform, Effect, Impact) {
    
    let transform = Transform::new(position, -direction.as_angle(), None);
    let effect = Effect { total_lifetime: 0.5, lifetime: RymdGameModel::TIME_STEP };
    let impact = Impact;

    (transform, effect, impact)
    
}

pub fn create_muzzle_flash_effect_in_world(world: &mut World, position: Vec2, direction: Vec2) -> Entity {
    world.spawn(create_muzzle_flash_effect(position, direction))
}

pub fn create_muzzle_flash_effect_in_buffer(buffer: &mut CommandBuffer, position: Vec2, direction: Vec2) {
    buffer.spawn(create_muzzle_flash_effect(position, direction))
}

pub fn create_impact_effect_in_world(world: &mut World, position: Vec2, direction: Vec2) -> Entity {
    world.spawn(create_impact_effect(position, direction))
}

pub fn create_impact_effect_in_buffer(buffer: &mut CommandBuffer, position: Vec2, direction: Vec2) {
    buffer.spawn(create_impact_effect(position, direction))
}

pub fn create_explosion_effect_in_world(world: &mut World, position: Vec2) -> Entity {
    world.spawn(create_explosion_effect(position))
}

pub fn create_explosion_effect_in_buffer(buffer: &mut CommandBuffer, position: Vec2) {
    buffer.spawn(create_explosion_effect(position))
}