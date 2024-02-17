use macroquad::prelude::Vec2;
use hecs::{World, Entity};

pub fn create_explosion_effect(world: &mut World, position: Vec2) -> Entity {
    world.spawn(())
}