use hecs::{CommandBuffer, Entity, World};
use macroquad::{math::Rect, prelude::Vec2};
use utility::{AsAngle, Kinematic};
use crate::PlayerID;

use super::{create_default_kinematic_body, create_impact_effect_in_buffer, create_muzzle_flash_effect_in_world, get_entity_physics_position, Beam, Controller, DynamicBody, DynamicBodyCallback, Effect, Health, PhysicsBody, Projectile, Sprite, Transform, SIMPLE_BEAM_PARAMETERS, SIMPLE_BULLET_PARAMETERS};

#[derive(Clone, Copy, Debug)]
pub struct BulletParameters {

    pub health: f32,
    pub lifetime: f32,
    pub velocity: f32,
    pub damage: f32,

    pub bounds: Rect,
    pub texture: &'static str

}

#[derive(Clone, Copy, Debug)]
pub struct BeamParameters {

    pub damage: f32,
    pub lifetime: f32,
    pub range: f32

}

fn on_bullet_impact(world: &World, buffer: &mut CommandBuffer, a: Entity, b: Entity, b_body: &DynamicBody) {
    
    if let Ok(mut bullet_health) = world.get::<&mut Health>(a) {
        bullet_health.kill();
    }

    if let Ok(projectile) = world.get::<&Projectile>(a) && let Ok(mut target_health) = world.get::<&mut Health>(b) {
        target_health.damage(projectile.damage);
    }

    let entity_a_physics_position = get_entity_physics_position(world, a).unwrap();
    let (position_on_target_radius, normal_on_targeted_entity) = get_position_and_normal_on_targeted_entity_relative_to(world, b_body, entity_a_physics_position);
    create_impact_effect_in_buffer(buffer, position_on_target_radius, -normal_on_targeted_entity);

}

pub fn get_position_and_normal_on_targeted_entity_relative_to(world: &World, body: &DynamicBody, position: Vec2) -> (Vec2, Vec2) {

    let (target_position, target_bounds) = (body.position(), body.bounds());
    let target_direction = -(target_position - position).normalize();
    let target_radius = target_bounds.size().max_element() / 2.0;

    let position_on_target_radius = target_position + target_direction * target_radius;
    let normal_on_targeted_entity = (position_on_target_radius - target_position).normalize();
    (position_on_target_radius, normal_on_targeted_entity)

}

fn create_bullet(world: &mut World, owner: PlayerID, position: Vec2, direction: Vec2, parameters: BulletParameters) -> Entity {

    let bullet_health = parameters.health;
    let bullet_lifetime = parameters.lifetime;
    let bullet_velocity = parameters.velocity;
    let bullet_damage = parameters.damage;

    let is_static = false;
    let is_enabled = true;
    let bounds = parameters.bounds;
    let mask = 1 << owner;
    
    let orientation = -direction.as_angle();
    let kinematic = Kinematic {
        mass: 0.1,
        ..create_default_kinematic_body(position, orientation)
    };

    let controller = Controller { id: owner };
    let transform = Transform::new(position, orientation, None);
    let sprite = Sprite { texture: parameters.texture.to_string() };
    let dynamic_body = DynamicBody { is_static, is_enabled, bounds, kinematic, mask };
    let dynamic_body_callback = DynamicBodyCallback { on_collision: on_bullet_impact };
    let projectile = Projectile { damage: bullet_damage, lifetime: bullet_lifetime, velocity: bullet_velocity };
    let health = Health::new(bullet_health);

    create_muzzle_flash_effect_in_world(world, position, -direction);
    world.spawn((controller, transform, sprite, dynamic_body, dynamic_body_callback, health, projectile))

}

pub fn create_simple_bullet(world: &mut World, owner: PlayerID, position: Vec2, direction: Vec2) -> Entity {
    create_bullet(world, owner, position, direction, SIMPLE_BULLET_PARAMETERS)
}

fn create_beam(world: &mut World, owner: PlayerID, position: Vec2, direction: Vec2, parameters: BeamParameters) -> Entity {

    let beam_damage = parameters.damage;
    let beam_lifetime = parameters.lifetime;
    let beam_range = parameters.range;

    let is_static = false;
    let is_enabled = true;
    let mask = 1 << owner;
    
    let orientation = -direction.as_angle();
    let controller = Controller { id: owner };
    let transform = Transform::new(position, orientation, None);
    let beam = Beam { position, target: position + direction * beam_range, damage: beam_damage, fired: false };
    let effect = Effect::new(0.5);

    create_muzzle_flash_effect_in_world(world, position, -direction);
    world.spawn((controller, transform, beam, effect))

}

pub fn create_simple_beam(world: &mut World, owner: PlayerID, position: Vec2, direction: Vec2) -> Entity {
    create_beam(world, owner, position, direction, SIMPLE_BEAM_PARAMETERS)
}