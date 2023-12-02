use macroquad::prelude::*;
use hecs::{Entity, World};
use utility::{AsPerpendicular, intersect_rect};

use super::DynamicBody;

const COLLISION_ELASTICITY: f32 = 1.0;

pub trait PhysicsBody {

    fn bounds(&self) -> Rect;
    fn position(&self) -> Vec2;
    fn velocity(&self) -> Vec2;
    fn angular_velocity(&self) -> f32;
    fn friction(&self) -> f32;
    fn mass(&self) -> f32;

    fn bounds_mut(&mut self) -> &mut Rect;
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
            body.kinematic.integrate(self.timestep);
            body.kinematic.apply_friction(self.timestep);
        }

    }

    pub fn handle_overlaps(&mut self, world: &mut World) {

        for (e1, body) in world.query::<&DynamicBody>().iter() {
            for (e2, other_body) in world.query::<&DynamicBody>().iter() {

                let should_collide = self.collides_with(body, other_body);
                if e1 != e2 && intersect_rect(&body.bounds(), &other_body.bounds()) {
                    let new_response = (e1, e2, false);
                    if self.collision_responses.iter().any(|(a_entity, b_entity, _)| *a_entity == new_response.0 && *b_entity == new_response.1) {
                        // we already have a pre-existing response, do not add another one
                    } else {
                        self.collision_responses.push(new_response);
                    }
                }

            }
        }

    }

    pub fn handle_collisions(&mut self, world: &mut World) {

        for (entity, other, active) in &mut self.collision_responses {

            let resolved_impact_velocity = {

                let other_body = {
                    (*world.get::<&DynamicBody>(*other).expect("must have dynamic body!")).clone()
                };

                let mut this_body = world.get::<&mut DynamicBody>(*entity).expect("must have dynamic body!");

                let resolved_impact_velocity = if !(*active) {
                    Self::collision_response_with_entity(&mut this_body, &other_body, self.timestep)
                } else {
                    Self::collision_separate_from_entity(&mut this_body, &other_body, self.timestep);
                    Vec2::ZERO
                };

                *active = intersect_rect(&this_body.bounds(), &other_body.bounds());

                resolved_impact_velocity

            };

            // resolve impact velocity...
            let mut other_body = world.get::<&mut DynamicBody>(*other).expect("must have dynamic body!");
            let other_body_mass = other_body.mass();
            *other_body.velocity_mut() -= resolved_impact_velocity * other_body_mass;

        }

        // retain only active collisions
        self.collision_responses.retain(|(a_entity, b_entity, active)| *active);

    }

    pub fn standard_collision_response_with_entity(entity: &mut DynamicBody, other: &DynamicBody, timestep: f32) -> Vec2 {

        let average_size = (entity.bounds().w + entity.bounds().h) / 2.0;
    
        let separating_vector = -(other.position() - entity.position());
        let normalized_offset = separating_vector.normalize();
        let offset_magnitude = average_size / separating_vector.length();

        let other_velocity_normal = other.velocity().normalize();
        let left_of_center_of_mass = other_velocity_normal.dot(normalized_offset.perpendicular()) < 0.0;
        let offset_multiplier = if left_of_center_of_mass { 1.0 } else { -1.0 };

        let their_velocity_normal = other.velocity().normalize();
        let other_velocity_towards_us_factor = their_velocity_normal.normalize().dot(separating_vector.normalize()).max(0.0);
        let their_velocity_projection = other.velocity() * other_velocity_towards_us_factor;

        let impact_velocity_vector = other.mass() * their_velocity_projection;
        let impact_velocity_angular = other.mass() * offset_magnitude * offset_multiplier * (other.velocity().length() / 16.0);
        
        *entity.velocity_mut() += impact_velocity_vector * COLLISION_ELASTICITY * timestep;
        *entity.angular_velocity_mut() += impact_velocity_angular * timestep;

        impact_velocity_vector

    }

    pub fn collision_response_with_entity(entity: &mut DynamicBody, other: &DynamicBody, timestep: f32) -> Vec2 {

        let response = Self::standard_collision_response_with_entity(entity, other, timestep);

        if response.is_nan() {
            Vec2::ZERO
        } else {
            response
        }

    }

    pub fn collision_separate_from_entity(entity: &mut DynamicBody, other: &DynamicBody, timestep: f32) {

        let average_size = (entity.bounds().w + entity.bounds().h) / 2.0;
        let separating_vector = -(other.position() - entity.position());
        let separating_vector_length = separating_vector.length();

        let offset_magnitude = ((separating_vector_length*separating_vector_length) / average_size) * 8.0;
        *entity.velocity_mut() += separating_vector.normalize() * offset_magnitude * timestep;

    }

    pub fn collides_with(&self, entity: &DynamicBody, other: &DynamicBody) -> bool {
        true
    }

}