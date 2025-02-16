use std::collections::HashMap;

use macroquad::prelude::*;
use hecs::{CommandBuffer, Entity, Without, World};
use rapier2d::{crossbeam, prelude::*};
use utility::line_segment_rect_intersection;
use super::{spatial::{entity_distance_sort_function, SpatialQueryManager}, DynamicBody, DynamicBodyCallback};

const COLLISION_ELASTICITY: f32 = 1.0;

struct PhysicsBodyHandle {
    rigid_body_handle: RigidBodyHandle
}

pub trait PhysicsBody {

    fn enabled(&self) -> bool;

    fn bounds(&self) -> Rect;
    fn position(&self) -> Vec2;
    fn visual_position(&self) -> Vec2;
    fn physics_bounds(&self) -> Rect;
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

    fn apply_impulse(&mut self, impulse: Vec2, offset: Vec2);

}

struct PhysicsManagerRapierCoreState {
    integration_parameters: IntegrationParameters,
    physics_pipeline: PhysicsPipeline,
    island_manager: IslandManager,
    broad_phase: DefaultBroadPhase,
    narrow_phase: NarrowPhase,
    rigid_body_set: RigidBodySet,
    collider_set: ColliderSet,
    impulse_joint_set: ImpulseJointSet,
    multibody_joint_set: MultibodyJointSet,
    ccd_solver: CCDSolver,
    query_pipeline: QueryPipeline
}

impl PhysicsManagerRapierCoreState {

    pub fn new(timestep: f32) -> PhysicsManagerRapierCoreState {

        let mut default_integration_parameters = IntegrationParameters::default();
        default_integration_parameters.dt = timestep;

        PhysicsManagerRapierCoreState {
            integration_parameters: default_integration_parameters,
            physics_pipeline: PhysicsPipeline::new(),
            island_manager: IslandManager::new(),
            broad_phase: DefaultBroadPhase::new(),
            narrow_phase: NarrowPhase::new(),
            rigid_body_set: RigidBodySet::new(),
            collider_set: ColliderSet::new(),
            impulse_joint_set: ImpulseJointSet::new(),
            multibody_joint_set: MultibodyJointSet::new(),
            ccd_solver: CCDSolver::new(),
            query_pipeline: QueryPipeline::new(),
        }

    }

    pub fn step(&mut self, mut on_collision_event: impl FnMut(RigidBodyHandle, RigidBodyHandle) -> ()) {

        let physics_hooks = ();

        let (collision_send, collision_recv) = crossbeam::channel::unbounded();
        let (contact_force_send, contact_force_recv) = crossbeam::channel::unbounded();
        let event_handler = ChannelEventCollector::new(collision_send, contact_force_send);
        
        self.physics_pipeline.step(
            &vector![0.0, 0.0],
            &mut self.integration_parameters,
            &mut self.island_manager,
            &mut self.broad_phase,
            &mut self.narrow_phase,
            &mut self.rigid_body_set,
            &mut self.collider_set,
            &mut self.impulse_joint_set,
            &mut self.multibody_joint_set,
            &mut self.ccd_solver,
            Some(&mut self.query_pipeline),
            &physics_hooks,
            &event_handler
        );

        while let Ok(collision_event) = collision_recv.try_recv() {

            if collision_event.removed() {
                continue
            }

            if collision_event.stopped() {
                continue
            }

            let rigid_body_handle_1 = self.collider_set.get(collision_event.collider1()).unwrap().parent();
            let rigid_body_handle_2 = self.collider_set.get(collision_event.collider2()).unwrap().parent();
            if let Some(r1) = rigid_body_handle_1 && let Some(r2) = rigid_body_handle_2 {
                on_collision_event(r1, r2)
            }

        }

        while let Ok(contact_force_event) = contact_force_recv.try_recv() {
            // do a thing?
        }

    }

}

pub struct PhysicsManager {
    core_state: PhysicsManagerRapierCoreState,
    rigid_body_handle_to_entity: HashMap<RigidBodyHandle, Entity>
}

impl PhysicsManager {

    pub fn new(timestep: f32) -> PhysicsManager {
        PhysicsManager {
            core_state: PhysicsManagerRapierCoreState::new(timestep),
            rigid_body_handle_to_entity: HashMap::new()
        }
    }

    pub fn timestep(&self) -> f32 {
        self.core_state.integration_parameters.dt
    }

    pub fn number_of_active_collision_responses(&self) -> usize {
        0
    }

    pub fn clear(&mut self) {

    }

    pub fn ray_cast(&self, source: Vec2, target: Vec2, world: &World, spatial_query_manager: &SpatialQueryManager, test_mask: u64) -> Option<(Entity, Vec2)> {
        
        let mut first_entity_hit = None;
        let mut first_position_hit = None;

        for e in spatial_query_manager.entities_within_overlapping_line_segment_sorted_by(source, target, |a, b| entity_distance_sort_function(world, source, a, b)) {

            let Ok(body) = world.get::<&DynamicBody>(e) else { continue; };
            if body.mask & test_mask != 0 {
                continue;
            }

            if let Some(intersection) = line_segment_rect_intersection(source, target, body.physics_bounds()) {
                first_position_hit = Some(intersection);
                first_entity_hit = Some(e);
                break;
            }

        }
        
        if let Some(entity) = first_entity_hit && let Some(position) = first_position_hit {
            Some((entity, position))
        } else {
            None
        }

    }

    pub fn tick(&mut self, world: &mut World) {

        self.handle_created_entities(world);
        self.handle_destroyed_entities(world);

        for (e, (dynamic_body, physics_body_handle)) in world.query_mut::<(&mut DynamicBody, &PhysicsBodyHandle)>() {

            let current_rigid_body = self.core_state.rigid_body_set.get_mut(physics_body_handle.rigid_body_handle).unwrap();

            let current_dynamic_body_velocity = dynamic_body.kinematic.velocity;
            let current_dynamic_body_angular_velocity = dynamic_body.kinematic.angular_velocity;

            // pass our updated velocities/angles to the physics state, then less the physics engine affect the bodies in between
            current_rigid_body.set_linvel(vector![current_dynamic_body_velocity.x, current_dynamic_body_velocity.y], true);
            current_rigid_body.set_angvel(current_dynamic_body_angular_velocity, true);

            if dynamic_body.is_static == false && current_rigid_body.body_type() == RigidBodyType::Fixed {
                current_rigid_body.set_body_type(RigidBodyType::Dynamic, true);
            }
            else if dynamic_body.is_static && current_rigid_body.body_type() == RigidBodyType::Dynamic {
                current_rigid_body.set_body_type(RigidBodyType::Fixed, true);
            }

        }

        let mut command_buffer = CommandBuffer::new();

        self.core_state.step(|a, b| {

            let a_entity = self.rigid_body_handle_to_entity[&a];
            let b_entity = self.rigid_body_handle_to_entity[&b];

            if let Ok(a_callback) = world.get::<&DynamicBodyCallback>(a_entity) {
                if let Ok(other_body) = world.get::<&DynamicBody>(b_entity) {
                    (a_callback.on_collision)(world, &mut command_buffer, a_entity, b_entity, &other_body);
                }
            }

            if let Ok(b_callback)= world.get::<&DynamicBodyCallback>(b_entity) {
                if let Ok(other_body) = world.get::<&DynamicBody>(a_entity) {
                    (b_callback.on_collision)(world, &mut command_buffer, b_entity, a_entity, &other_body);
                }
            }

        });

        command_buffer.run_on(world);

        for (e, (dynamic_body, physics_body_handle)) in world.query_mut::<(&mut DynamicBody, &PhysicsBodyHandle)>() {

            let current_rigid_body = self.core_state.rigid_body_set.get(physics_body_handle.rigid_body_handle).unwrap();
            let current_rigid_body_translation = current_rigid_body.translation();
            let current_rigid_body_rotation = current_rigid_body.rotation().angle();
            let current_rigid_body_velocity = current_rigid_body.linvel();
            let current_rigid_body_angular_velocity = current_rigid_body.angvel();

            // update position/rotation of kinematic according to new reported physics position!
            dynamic_body.kinematic.position = vec2(current_rigid_body_translation.x, current_rigid_body_translation.y);
            dynamic_body.kinematic.orientation = current_rigid_body_rotation;

            dynamic_body.kinematic.velocity = vec2(current_rigid_body_velocity.x, current_rigid_body_velocity.y);
            dynamic_body.kinematic.angular_velocity = current_rigid_body_angular_velocity;

        }

    }

    fn create_rigid_body_for_entity(physics_body: &DynamicBody) -> RigidBody {

        let body_translation = physics_body.position();
        let body_orientation = physics_body.orientation();
        let body_type = if physics_body.is_static { RigidBodyType::Fixed } else { RigidBodyType::Dynamic };

        RigidBodyBuilder::new(body_type)
            .translation(vector![body_translation.x, body_translation.y])
            .rotation(body_orientation)
            .linear_damping(physics_body.friction())
            .angular_damping(physics_body.friction())
            .build()
    }

    fn create_collider_for_entity(physics_body: &DynamicBody) -> Collider {

        let body_bounds = physics_body.bounds();

        // #FIXME: this is a tad bit horrible, but it's cool that it's this simple to set up the collision masks, guess we just gotta worry if we have > 32 teams?
        let body_interaction_groups = InteractionGroups::new((physics_body.mask as u32).into(), Group::all().difference((physics_body.mask as u32).into()));

        ColliderBuilder::cuboid(body_bounds.w / 2.0, body_bounds.h / 2.0)
            .active_events(ActiveEvents::COLLISION_EVENTS)
            .collision_groups(body_interaction_groups)
            .build()
    }

    fn handle_created_entities(&mut self, world: &mut World) {

        let mut bodies_to_create: Vec<Entity> = Vec::new();

        for (e, dynamic_body) in world.query_mut::<Without<&DynamicBody, &PhysicsBodyHandle>>() {
            bodies_to_create.push(e);
        }

        for e in bodies_to_create {
            
            let dynamic_body = world.query_one_mut::<&DynamicBody>(e).unwrap();
            let new_rigid_body = Self::create_rigid_body_for_entity(dynamic_body);
            let new_collider = Self::create_collider_for_entity(dynamic_body);

            let new_rigid_body_handle = self.core_state.rigid_body_set.insert(new_rigid_body);
            self.core_state.collider_set.insert_with_parent(new_collider, new_rigid_body_handle, &mut self.core_state.rigid_body_set);

            // register our newly created rigid body handle so we can actually track it
            self.rigid_body_handle_to_entity.insert(new_rigid_body_handle, e);

            // add the physics body handle component to our entity :)
            let _ = world.insert_one(e, PhysicsBodyHandle { rigid_body_handle: new_rigid_body_handle });
            
        }

    }

    fn handle_destroyed_entities(&mut self, world: &mut World) {

        let mut destroyed_bodies: Vec<RigidBodyHandle> = Vec::new();

        for (&rigid_body, &e) in &self.rigid_body_handle_to_entity {
            if world.contains(e) == false {
                destroyed_bodies.push(rigid_body);
            }
        }

        // clean up all destroyed entities and their bodies/colliders!
        for rigid_body_handle in destroyed_bodies {

            let should_remove_attached_colliders = true;

            self.core_state.rigid_body_set.remove(
                rigid_body_handle,
                &mut self.core_state.island_manager,
                &mut self.core_state.collider_set,
                &mut self.core_state.impulse_joint_set,
                &mut self.core_state.multibody_joint_set,
                should_remove_attached_colliders
            );

            self.rigid_body_handle_to_entity.remove(&rigid_body_handle);

        }

    }

}