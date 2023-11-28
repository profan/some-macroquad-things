use hecs::{Entity, World};
use macroquad::prelude::*;
use nanoserde::{SerJson, DeJson};
use lockstep_client::step::LockstepClient;
use utility::{Kinematic, arrive_ex, face_ex, SteeringOutput, AsVector};

use crate::EntityID;
use crate::model::GameMessage;

use super::{Transform, DynamicBody, DEFAULT_STEERING_PARAMETERS, Constructor, Controller, Health, Orderable};

fn ship_apply_steering(kinematic: &mut Kinematic, steering_maybe: Option<SteeringOutput>, dt: f32) {

    let turn_rate = 4.0;
    if let Some(steering) = steering_maybe {

        let desired_linear_velocity = steering.linear * dt;

        // project our desired velocity along where we're currently pointing first
        let projected_linear_velocity = desired_linear_velocity * desired_linear_velocity.dot(-kinematic.orientation.as_vector()).max(0.0);
        kinematic.velocity += projected_linear_velocity;

        let turn_delta = steering.angular * turn_rate * dt;
        kinematic.angular_velocity += turn_delta;

    }

}

pub fn get_entity_position(world: &World, entity: Entity) -> Option<Vec2> {
    world.get::<&Transform>(entity).and_then(|t| Ok(t.world_position)).or(Err(())).ok()
}

pub fn get_entity_position_from_id(world: &World, entity_id: u64) -> Option<Vec2> {
    world.get::<&Transform>(Entity::from_bits(entity_id).unwrap()).and_then(|t| Ok(t.world_position)).or(Err(())).ok()
}

fn steer_ship_towards_target(world: &mut World, entity: Entity, x: f32, y: f32, dt: f32) {

    if let Ok(mut dynamic_body) = world.get::<&mut DynamicBody>(entity) {

        let target_kinematic = Kinematic { position: vec2(x, y), ..Default::default() };
        let time_to_target = 1.0;

        let arrive_steering_output = arrive_ex(
            &dynamic_body.kinematic,
            &target_kinematic,
            DEFAULT_STEERING_PARAMETERS,
            time_to_target
        ).unwrap_or_default();

        let face_steering_output = face_ex(
            &dynamic_body.kinematic,
            &target_kinematic,
            DEFAULT_STEERING_PARAMETERS,
            time_to_target
        ).unwrap_or_default();

        let final_steering_output = arrive_steering_output + face_steering_output;
        ship_apply_steering(&mut dynamic_body.kinematic, Some(final_steering_output), dt);

    }

}

pub trait GameOrdersExt {
    fn send_move_order(&mut self, entity: Entity, target_position: Vec2, should_add: bool);
    fn send_build_order(&mut self, entity: Entity, target_position: Vec2, blueprint_idx: usize, should_add: bool);
    fn send_repair_order(&mut self, entity: Entity, target_position: Vec2, target: Entity, should_add: bool);
}

impl GameOrdersExt for LockstepClient {

    fn send_move_order(&mut self, entity: Entity, target_position: Vec2, should_add: bool) {
        let move_order = GameOrder::Move(MoveOrder { x: target_position.x, y: target_position.y });
        let move_unit_message = GameMessage::Order { entities: vec![entity.to_bits().into()], order: move_order, add: should_add };
        self.send_command(move_unit_message.serialize_json());
    }

    fn send_build_order(&mut self, entity: Entity, target_position: Vec2, blueprint_idx: usize, should_add: bool) {
        let build_order = GameOrder::Construct(ConstructOrder { entity_id: None, blueprint_idx: Some(blueprint_idx), x: target_position.x, y: target_position.y });
        let build_unit_message = GameMessage::Order { entities: vec![entity.to_bits().into()], order: build_order, add: should_add };
        self.send_command(build_unit_message.serialize_json());
    }

    fn send_repair_order(&mut self, entity: Entity, target_position: Vec2, target: Entity, should_add: bool) {
        let build_order = GameOrder::Construct(ConstructOrder { entity_id: Some(target.to_bits().get()), blueprint_idx: None, x: target_position.x, y: target_position.y });
        let build_unit_message = GameMessage::Order { entities: vec![entity.to_bits().into()], order: build_order, add: should_add };
        self.send_command(build_unit_message.serialize_json());
    }

}

trait Order {

    fn is_order_completed(&self, entity: Entity, world: &World) -> bool;
    fn get_target_position(&self, world: &World) -> Option<Vec2>;
    fn tick(&self, entity: Entity, world: &mut World, dt: f32);

}

impl GameOrder {

    pub fn is_order_completed(&self, entity: Entity, world: &World) -> bool {
        match self {
            GameOrder::Move(order) => order.is_order_completed(entity, world),
            GameOrder::Attack(order) => order.is_order_completed(entity, world),
            GameOrder::AttackMove(order) => order.is_order_completed(entity, world),
            GameOrder::Construct(order) => order.is_order_completed(entity, world)
        }
    }

    pub fn get_target_position(&self, world: &World) -> Option<Vec2> {
        match self {
            GameOrder::Move(order) => order.get_target_position(world),
            GameOrder::Attack(order) => order.get_target_position(world),
            GameOrder::AttackMove(order) => order.get_target_position(world),
            GameOrder::Construct(order) => order.get_target_position(world)
        }
    }

    pub fn tick(&self, entity: Entity, world: &mut World, dt: f32) {
        match self {
            GameOrder::Move(order) => order.tick(entity, world, dt),
            GameOrder::Attack(order) => order.tick(entity, world, dt),
            GameOrder::AttackMove(order) => order.tick(entity, world, dt),
            GameOrder::Construct(order) => order.tick(entity, world, dt)
        }
    }
 
}

#[derive(Debug, Copy, Clone, SerJson, DeJson)]
pub enum GameOrder {
    Move(MoveOrder),
    Attack(AttackOrder),
    AttackMove(AttackMoveOrder),
    Construct(ConstructOrder)
}

#[derive(Debug, Copy, Clone, SerJson, DeJson)]
pub struct MoveOrder {
    x: f32,
    y: f32
}

impl Order for MoveOrder {
    fn is_order_completed(&self, entity: Entity, world: &World) -> bool {
        let arbitrary_distance_threshold = 64.0;
        let position = get_entity_position(world, entity).expect("could not get position for move order, should never happen!");
        position.distance(vec2(self.x, self.y)) < arbitrary_distance_threshold
    }

    fn get_target_position(&self, world: &World) -> Option<Vec2> {
        Some(vec2(self.x, self.y))
    }

    fn tick(&self, entity: Entity, world: &mut World, dt: f32) {
        steer_ship_towards_target(world, entity, self.x, self.y, dt);
    }
}

#[derive(Debug, Copy, Clone, SerJson, DeJson)]
pub struct AttackOrder {
    entity_id: EntityID
}

impl Order for AttackOrder {
    fn is_order_completed(&self, entity: Entity, world: &World) -> bool {
        todo!()
    }

    fn get_target_position(&self, world: &World) -> Option<Vec2> {
        get_entity_position_from_id(world, self.entity_id)
    }

    fn tick(&self, entity: Entity, world: &mut World, dt: f32) {
        todo!()
    }
}

#[derive(Debug, Copy, Clone, SerJson, DeJson)]
pub struct AttackMoveOrder {
    x: f32,
    y: f32
}

impl Order for AttackMoveOrder {
    fn is_order_completed(&self, entity: Entity, world: &World) -> bool {
        todo!()
    }

    fn get_target_position(&self, world: &World) -> Option<Vec2> {
        Some(vec2(self.x, self.y))
    }

    fn tick(&self, entity: Entity, world: &mut World, dt: f32) {
        todo!()
    }
}

#[derive(Debug, Copy, Clone, SerJson, DeJson)]
pub struct ConstructOrder {
    entity_id: Option<EntityID>,
    blueprint_idx: Option<usize>,
    x: f32,
    y: f32
}

impl Order for ConstructOrder {

    fn is_order_completed(&self, entity: Entity, world: &World) -> bool {
        
        if let Some(entity_id) = self.entity_id && let Some(constructing_entity) = Entity::from_bits(entity_id) {
            let entity_health = world.get::<&Health>(constructing_entity).expect("building must have entity health component to be able to construct!");
            entity_health.health >= entity_health.full_health
        } else {
            false
        }

    }

    fn get_target_position(&self, world: &World) -> Option<Vec2> {
        if let Some(entity_id) = self.entity_id {
            get_entity_position_from_id(world, entity_id)
        } else {
            Some(vec2(self.x, self.y))
        }
    }

    fn tick(&self, entity: Entity, world: &mut World, dt: f32) {

        // we're building/repairing an existing construction, this could be tick 2 of the "constructing a blueprint" case
        if let Some(entity_id) = self.entity_id && let Some(constructing_entity) = Entity::from_bits(entity_id) {

            let target_position = get_entity_position_from_id(world, entity_id).expect("could not unpack target position?");
            if self.is_within_constructor_range(entity, world, target_position) == false {
                steer_ship_towards_target(world, entity, target_position.x, target_position.y, dt);
                return;
            }

            let constructor = world.get::<&Constructor>(entity).expect("must have constructor to be issuing construct order!");

            if self.is_order_completed(entity, world) == false {
                let mut entity_health = world.get::<&mut Health>(constructing_entity).expect("building must have entity health component to be able to construct!");
                entity_health.health = (entity_health.health + (constructor.build_speed as f32 * dt) as i32).min(entity_health.full_health);
            }

        }

        // we're constructing something new given a blueprint
        if let Some(blueprint_idx) = self.blueprint_idx {

            let construction_position = vec2(self.x, self.y);

            if self.is_within_constructor_range(entity, world, construction_position) == false {
                steer_ship_towards_target(world, entity, construction_position.x, construction_position.y, dt);
                return;
            }

            let controller_id = world.get::<&Controller>(entity).expect("must have controller to be issuing construct order!").id;
            let blueprint = world.get::<&Constructor>(entity).expect("must have constructor to be issuing construct order!").constructibles[blueprint_idx].clone();
            let new_entity_id = (blueprint.constructor)(world, controller_id, construction_position);

            // cancel our current order now
            self.cancel_current_order(entity, world);

            // now that this is created, issue a local order to help build this new building
            self.construct_building(entity, world, new_entity_id, construction_position);

        }

    }
    
}

impl ConstructOrder {

    fn is_within_constructor_range(&self, entity: Entity, world: &World, target: Vec2) -> bool {

        let entity_position = get_entity_position_from_id(world, entity.to_bits().get()).expect("must have position!");
        let constructor = world.get::<&Constructor>(entity).expect("must have constructor to be issuing construct order!");

        (entity_position.distance(target) as i32) < constructor.build_range

    }

    fn cancel_current_order(&self, entity: Entity, world: &mut World) {
        let mut orderable = world.get::<&mut Orderable>(entity).expect("must have orderable!");
        orderable.orders.pop_front();
    }

    fn construct_building(&self, entity: Entity, world: &mut World, building_entity: Entity, position: Vec2) {
        let mut orderable = world.get::<&mut Orderable>(entity).expect("must have orderable!");
        orderable.orders.push_front(GameOrder::Construct(ConstructOrder { entity_id: Some(building_entity.to_bits().get()), blueprint_idx: None, x: position.x, y: position.y }))
    }

}