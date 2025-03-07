use macroquad::prelude::*;
use hecs::{World, Entity};

use utility::{arrive_ex, face_ex, SteeringOutput, Kinematic, AsVector};
use super::{DynamicBody, MovementTarget, PhysicsBody, RotationTarget, Steering, Transform, DEFAULT_STEERING_PARAMETERS};

pub fn get_entity_physics_position(world: &World, entity: Entity) -> Option<Vec2> {
    world.get::<&DynamicBody>(entity).map(|b| b.position()).or(Err(())).ok()
}

pub fn get_entity_position(world: &World, entity: Entity) -> Option<Vec2> {
    world.get::<&Transform>(entity).map(|t| t.world_position).or(Err(())).ok()
}

pub fn get_entity_position_from_id(world: &World, entity_id: u64) -> Option<Vec2> {
    world.get::<&Transform>(Entity::from_bits(entity_id).unwrap()).map(|t| t.world_position).or(Err(())).ok()
}

pub fn get_entity_direction(world: &World, entity: Entity) -> Option<f32> {
    world.get::<&Transform>(entity).map(|t| t.world_rotation).or(Err(())).ok()
}

pub fn get_entity_direction_from_id(world: &World, entity_id: u64) -> Option<f32> {
    world.get::<&Transform>(Entity::from_bits(entity_id).unwrap()).map(|t| t.world_rotation).or(Err(())).ok()
}

pub fn get_closest_position_with_entity_bounds(world: &World, entity: Entity) -> Option<(Vec2, Rect)> {
    let entity_position = get_entity_position(world, entity)?;
    if let Ok(entity_bounds) = world.get::<&DynamicBody>(entity) {
        Some((entity_position, entity_bounds.bounds()))
    } else {
        None
    }
}

pub fn get_closest_position_with_entity_id_bounds(world: &World, entity_id: u64) -> Option<(Vec2, Rect)> {
    let entity_position = get_entity_position_from_id(world, entity_id)?;
    if let Ok(entity_bounds) = world.get::<&DynamicBody>(Entity::from_bits(entity_id).unwrap()) {
        Some((entity_position, entity_bounds.bounds()))
    } else {
        None
    }
}

fn entity_apply_steering(kinematic: &mut Kinematic, steering_maybe: Option<SteeringOutput>, dt: f32) {

    if let Some(steering) = steering_maybe {

        let desired_linear_velocity = steering.linear * dt;

        // project our desired velocity along where we're currently pointing first
        let projected_linear_velocity = desired_linear_velocity * desired_linear_velocity.dot(kinematic.orientation.as_vector()).max(0.0);
        kinematic.velocity += projected_linear_velocity;

        let turn_delta = steering.angular * dt;
        kinematic.angular_velocity += turn_delta;

    }

}

pub fn entity_apply_raw_steering(kinematic: &mut Kinematic, steering_maybe: Option<SteeringOutput>, dt: f32) {

    if let Some(steering) = steering_maybe {
        kinematic.velocity += steering.linear * dt;
        kinematic.angular_velocity += steering.angular * dt;
    }

}

pub fn steer_entity_towards_target(world: &mut World, entity: Entity, x: f32, y: f32, dt: f32) {

    if let Ok(mut dynamic_body) = world.get::<&mut DynamicBody>(entity) {

        let parameters = if let Ok(steering) = world.get::<&Steering>(entity) {
            steering.parameters
        } else {
            DEFAULT_STEERING_PARAMETERS
        };

        let target_kinematic = Kinematic { position: vec2(x, y), ..Default::default() };
        let time_to_target = 1.0;

        let arrive_steering_output = arrive_ex(
            &dynamic_body.kinematic,
            &target_kinematic,
            parameters,
            time_to_target
        ).unwrap_or_default();

        let face_steering_output = face_ex(
            &dynamic_body.kinematic,
            &target_kinematic,
            parameters,
            time_to_target
        ).unwrap_or_default();

        let final_steering_output = arrive_steering_output + face_steering_output;
        entity_apply_steering(&mut dynamic_body.kinematic, Some(final_steering_output), dt);

    }

}

pub fn point_entity_towards_target(world: &mut World, entity: Entity, x: f32, y: f32, dt: f32) {

    if let Ok(mut dynamic_body) = world.get::<&mut DynamicBody>(entity) {

        let parameters = if let Ok(steering) = world.get::<&Steering>(entity) {
            steering.parameters
        } else {
            DEFAULT_STEERING_PARAMETERS
        };

        let target_kinematic = Kinematic { position: vec2(x, y), ..Default::default() };
        let time_to_target = 1.0;

        let face_steering_output = face_ex(
            &dynamic_body.kinematic,
            &target_kinematic,
            parameters,
            time_to_target
        ).unwrap_or_default();

        entity_apply_steering(&mut dynamic_body.kinematic, Some(face_steering_output), dt);

    }
    
}

pub fn set_movement_target_to_entity(world: &mut World, entity: Entity, target_entity: Entity) {
    let mut movement_target = world.get::<&mut MovementTarget>(entity).expect("anything being issued a move target must have the MovementTarget component!");
    movement_target.target = get_entity_position(world, target_entity);
}

pub fn set_movement_target_to_position(world: &World, entity: Entity, target_position: Option<Vec2>) {
    let mut movement_target = world.get::<&mut MovementTarget>(entity).expect("anything being issued a move target must have the MovementTarget component!");
    movement_target.target = target_position;
}

pub fn set_rotation_target_to_entity(world: &mut World, entity: Entity, target_entity: Entity) {
    let mut rotation_target = world.get::<&mut RotationTarget>(entity).expect("anything being issued a rotation target must have the RotationTarget component!");
    rotation_target.target = get_entity_position(world, target_entity);
}

pub fn set_rotation_target_to_position(world: &World, entity: Entity, target_position: Option<Vec2>) {
    let mut rotation_target = world.get::<&mut RotationTarget>(entity).expect("anything being issued a rotation target must have the RotationTarget component!");
    rotation_target.target = target_position;
}