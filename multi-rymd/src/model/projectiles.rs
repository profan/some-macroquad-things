use hecs::{CommandBuffer, Entity, World};
use macroquad::{math::Rect, prelude::Vec2};
use utility::{AsAngle, Kinematic};
use crate::PlayerID;

use super::{create_default_kinematic_body, create_impact_effect_in_buffer, create_muzzle_flash_effect_in_world, get_entity_physics_position, Controller, DynamicBody, DynamicBodyCallback, Health, PhysicsBody, Projectile, Sprite, Transform};

#[derive(Clone, Copy, Debug)]
pub struct BulletParameters {

    pub health: f32,
    pub lifetime: f32,
    pub velocity: f32,
    pub damage: f32,

    pub bounds: Rect,
    pub texture: &'static str

}

fn on_bullet_impact(world: &World, buffer: &mut CommandBuffer, a: Entity, b: Entity, b_body: &DynamicBody) {
    
    if let Ok(mut bullet_health) = world.get::<&mut Health>(a) {
        bullet_health.kill();
    }

    if let Ok(projectile) = world.get::<&Projectile>(a) && let Ok(mut target_health) = world.get::<&mut Health>(b) {
        target_health.damage(projectile.damage);
    }

    let (target_position, target_bounds) = (b_body.position(), b_body.bounds());
    let target_direction = -(target_position - get_entity_physics_position(world, a).unwrap()).normalize();
    let target_radius = target_bounds.size().max_element() / 2.0;

    let position_on_target_radius = target_position + target_direction * target_radius;
    let normal_on_targeted_entity = (position_on_target_radius - target_position).normalize();

    create_impact_effect_in_buffer(buffer, position_on_target_radius, -normal_on_targeted_entity);

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

    let simple_bullet_health = 10.0;
    let simple_bullet_lifetime = 4.0;
    let simple_bullet_velocity = 256.0;
    let simple_bullet_damage = 25.0;

    let simple_bullet_bounds = Rect { x: 0.0, y: 0.0, w: 2.0, h: 2.0 };
    let simple_bullet_texture = "SIMPLE_BULLET";

    let simple_bullet_parameters = BulletParameters {
        health: simple_bullet_health,
        lifetime: simple_bullet_lifetime,
        velocity: simple_bullet_velocity,
        damage: simple_bullet_damage,
        bounds: simple_bullet_bounds,
        texture: simple_bullet_texture
    };

    create_bullet(world, owner, position, direction, simple_bullet_parameters)

}