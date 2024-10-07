use macroquad::prelude::*;
use hecs::{CommandBuffer, Entity, World};
use utility::{AsPerpendicular, intersect_rect};

use super::{spatial::SpatialQueryManager, DynamicBody, DynamicBodyCallback};

const COLLISION_ELASTICITY: f32 = 1.0;

pub trait PhysicsBody {

    fn enabled(&self) -> bool;

    fn bounds(&self) -> Rect;
    fn position(&self) -> Vec2;
    fn visual_position(&self) -> Vec2;
    fn orientation(&self) -> f32;
    fn velocity(&self) -> Vec2;
    fn angular_velocity(&self) -> f32;
    fn friction(&self) -> f32;
    fn mass(&self) -> f32;

    fn position_mut(&mut self) -> &mut Vec2;
    fn velocity_mut(&mut self) -> &mut Vec2;
    fn angular_velocity_mut(&mut self) -> &mut f32;
    fn friction_mut(&mut self) -> &mut f32;
    fn mass_mut(&mut self) -> &mut f32;

}

pub struct PhysicsManager {
    collision_responses: Vec<(Entity, Entity, bool)>,
    timestep: f32
}

impl PhysicsManager {

    pub fn new(timestep: f32) -> PhysicsManager {
        PhysicsManager {
            collision_responses: Vec::new(),
            timestep
        }
    }

    pub fn number_of_active_collision_responses(&self) -> usize {
        self.collision_responses.len()
    }

    pub fn clear(&mut self) {
        self.collision_responses.clear();
    }

    pub fn integrate(&mut self, world: &mut World) {

        for (e, body) in world.query_mut::<&mut DynamicBody>() {
            if body.is_static == false {
                body.kinematic.integrate(self.timestep);
                body.kinematic.apply_friction(self.timestep);
            } else {
                body.kinematic.velocity = Vec2::ZERO;
                body.kinematic.angular_velocity = 0.0;
            }
        }

    }

    pub fn handle_overlaps(&mut self, world: &mut World, spatial_query_manager: &SpatialQueryManager) {

        for (e1, body) in world.query::<&DynamicBody>().iter() {

            if body.is_enabled == false {
                continue;
            }

            for e2 in spatial_query_manager.entities_within_rect(body.bounds()) {

                if e1 == e2 {
                    continue;
                }

                if let Ok(other_body) = unsafe { world.get_unchecked::<&DynamicBody>(e2) } && other_body.is_enabled {

                    let should_collide = self.collides_with(body, &other_body);
                    if should_collide && intersect_rect(&body.bounds(), &other_body.bounds()) {
                        let new_response = (e1, e2, false);
                        let pair_already_has_collision_response = self.collision_responses.iter().any(|(a_entity, b_entity, _)| *a_entity == new_response.0 && *b_entity == new_response.1);
                        if pair_already_has_collision_response == false {
                            self.collision_responses.push(new_response);
                        }
                    }

                }

            }

        }

    }

    fn handle_destroyed_entities(&mut self, world: &mut World) {

        let mut destroyed_responses = Vec::new();

        for r @ (entity, other, active) in &mut self.collision_responses.iter() {
            if world.contains(*entity) == false || world.contains(*other) == false {
                destroyed_responses.push(*r);
            }
        }

        // retain everything we haven't yeeted
        self.collision_responses.retain(|r| destroyed_responses.contains(&r) == false);

    }

    pub fn handle_collisions(&mut self, world: &mut World) {

        self.handle_destroyed_entities(world);

        let mut command_buffer = CommandBuffer::new();

        for (entity, other, active) in &mut self.collision_responses {

            let (resolved_impact_velocity, this_body_mass) = {

                let other_body = {
                    (*world.get::<&DynamicBody>(*other).expect("must have dynamic body!")).clone()
                };

                let mut this_body = world.get::<&mut DynamicBody>(*entity).expect("must have dynamic body!");
                let this_body_mass = this_body.mass();

                let resolved_impact_velocity = if !(*active) {
                    Self::collision_response_with_entity(&mut this_body, &other_body, self.timestep)
                } else {
                    Self::collision_separate_from_entity(&mut this_body, &other_body, self.timestep);
                    Vec2::ZERO
                };

                *active = intersect_rect(&this_body.bounds(), &other_body.bounds());

                (resolved_impact_velocity, this_body_mass)

            };

            // resolve impact velocity...
            {
                let mut other_body_mut = world.get::<&mut DynamicBody>(*other).expect("must have dynamic body!");
                let other_body_mass = other_body_mut.mass();
                *other_body_mut.velocity_mut() -= resolved_impact_velocity * other_body_mass;
            }

            // handle collision response callbacks, if it has one (make sure to drop mut ref here, or we get multiple muts)
            if let Ok(dynamic_body_callback) = world.get::<&DynamicBodyCallback>(*entity) {
                let other_body = world.get::<&DynamicBody>(*other).expect("must have dynamic body!");
                (dynamic_body_callback.on_collision)(&world, &mut command_buffer, *entity, *other, &other_body);
            }

        }

        // retain only active collisions
        self.collision_responses.retain(|(a_entity, b_entity, active)| *active);

        // run command buffer now
        command_buffer.run_on(world);

    }

    pub fn standard_collision_response_with_dynamic_entity(entity: &mut DynamicBody, other: &DynamicBody, timestep: f32) -> Vec2 {

        let average_size = (entity.bounds().w + entity.bounds().h) / 2.0;
    
        let separating_vector = -(other.position() - entity.position());
        let normalized_offset = separating_vector.normalize_or_zero();
        let offset_magnitude = average_size / separating_vector.length();

        let other_velocity_normal = other.velocity().normalize_or_zero();
        let left_of_center_of_mass = other_velocity_normal.dot(normalized_offset.perpendicular()) < 0.0;
        let offset_multiplier = if left_of_center_of_mass { 1.0 } else { -1.0 };

        let their_velocity_normal = other.velocity().normalize_or_zero();
        let other_velocity_towards_us_factor = their_velocity_normal.normalize_or_zero().dot(separating_vector.normalize_or_zero()).max(0.0);
        let their_velocity_projection = other.velocity() * other_velocity_towards_us_factor;

        let impact_velocity_vector = other.mass() * their_velocity_projection;
        let impact_velocity_angular = other.mass() * offset_magnitude * offset_multiplier * (other.velocity().length() / 16.0);
        
        *entity.velocity_mut() += impact_velocity_vector * COLLISION_ELASTICITY * timestep;
        *entity.angular_velocity_mut() += impact_velocity_angular * timestep;

        impact_velocity_vector

    }

    pub fn standard_collision_response_with_static_entity(entity: &mut DynamicBody, other: &DynamicBody, timestep: f32) -> Vec2 {

        let separating_vector = -(entity.position() - other.position()).normalize_or_zero();
        other.velocity() * -other.velocity().normalize_or_zero().dot(separating_vector)

    }

    pub fn collision_response_with_entity(entity: &mut DynamicBody, other: &DynamicBody, timestep: f32) -> Vec2 {

        let response = if entity.is_static == false {
            Self::standard_collision_response_with_dynamic_entity(entity, other, timestep)
        } else {
            Self::standard_collision_response_with_static_entity(entity, other, timestep)
        };

        if response.is_nan() {
            Vec2::ZERO
        } else {
            response
        }

    }

    pub fn standard_dynamic_collision_separate_from_entity(entity: &mut DynamicBody, other: &DynamicBody, timestep: f32) {

        let average_size = (entity.bounds().w + entity.bounds().h) / 2.0;
        let separating_vector = -(other.position() - entity.position());
        let separating_vector_length = separating_vector.length();

        let offset_magnitude = ((separating_vector_length*separating_vector_length) / average_size) * 8.0;
        *entity.velocity_mut() += separating_vector.normalize_or_zero() * offset_magnitude * timestep;

    }

    pub fn standard_static_collision_separate_from_entity(entity: &mut DynamicBody, other: &DynamicBody, timestep: f32) {

    }

    pub fn collision_separate_from_entity(entity: &mut DynamicBody, other: &DynamicBody, timestep: f32) {

        if entity.is_static == false {
            Self::standard_dynamic_collision_separate_from_entity(entity, other, timestep);
        } else {
            Self::standard_static_collision_separate_from_entity(entity, other, timestep)
        };

    }

    pub fn collides_with(&self, entity: &DynamicBody, other: &DynamicBody) -> bool {
        entity.mask & other.mask == 0
    }

}