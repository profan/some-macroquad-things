use macroquad::prelude::Vec2;
use hecs::{CommandBuffer, Entity, World};

use super::{AnimatedSprite, Effect, Transform}; 

fn create_explosion_effect(position: Vec2) -> (Transform, AnimatedSprite, Effect) {

    let lifetime = 1.0;
    let transform = Transform::new(position, 0.0, None);
    let sprite = AnimatedSprite { texture: "EXPLOSION".to_string(), current_frame: 0, h_frames: 15 };
    let effect = Effect::new(lifetime);

    (transform, sprite, effect)

}

pub fn create_explosion_effect_in_world(world: &mut World, position: Vec2) -> Entity {
    world.spawn(create_explosion_effect(position))
}

pub fn create_explosion_effect_in_buffer(buffer: &mut CommandBuffer, position: Vec2) {
    buffer.spawn(create_explosion_effect(position))
}