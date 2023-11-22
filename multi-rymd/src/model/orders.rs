use hecs::{Entity, World};
use lockstep_client::step::LockstepClient;
use macroquad::prelude::*;
use nanoserde::{SerJson, DeJson};
use utility::{Kinematic, arrive_ex, face_ex};

use crate::{EntityID, ship_apply_steering, get_entity_position};
use crate::model::GameMessage;

use super::{Transform, DynamicBody, DEFAULT_STEERING_PARAMETERS};

pub trait GameOrdersExt {
    fn send_move_order(&mut self, entity: Entity, target_position: Vec2, should_add: bool);
}

impl GameOrdersExt for LockstepClient {

    fn send_move_order(&mut self, entity: Entity, target_position: Vec2, should_add: bool) {
        let move_order = GameOrder::Move(MoveOrder { x: target_position.x, y: target_position.y });
        let move_unit_message = GameMessage::Order { entities: vec![entity.to_bits().into()], order: move_order, add: should_add };
        self.send_command(move_unit_message.serialize_json());
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
            GameOrder::AttackMove(order) => order.is_order_completed(entity, world)
        }
    }

    pub fn get_target_position(&self, world: &World) -> Option<Vec2> {
        match self {
            GameOrder::Move(order) => order.get_target_position(world),
            GameOrder::Attack(order) => order.get_target_position(world),
            GameOrder::AttackMove(order) => order.get_target_position(world)
        }
    }

    pub fn tick(&self, entity: Entity, world: &mut World, dt: f32) {
        match self {
            GameOrder::Move(order) => order.tick(entity, world, dt),
            GameOrder::Attack(order) => order.tick(entity, world, dt),
            GameOrder::AttackMove(order) => order.tick(entity, world, dt),
        }
    }
 
}

#[derive(Debug, Copy, Clone, SerJson, DeJson)]
pub enum GameOrder {
    Move(MoveOrder),
    Attack(AttackOrder),
    AttackMove(AttackMoveOrder),
}

#[derive(Debug, Copy, Clone, SerJson, DeJson)]
pub struct MoveOrder {
    x: f32,
    y: f32
}

impl Order for MoveOrder {
    fn is_order_completed(&self, entity: Entity, world: &World) -> bool {
        if let Ok(transform) = world.get::<&Transform>(entity) {
            let arbitrary_distance_threshold = 64.0;
            transform.world_position.distance(vec2(self.x, self.y)) < arbitrary_distance_threshold
        } else {
            false
        }
    }

    fn get_target_position(&self, world: &World) -> Option<Vec2> {
        Some(vec2(self.x, self.y))
    }

    fn tick(&self, entity: Entity, world: &mut World, dt: f32) {

        if let Ok(mut dynamic_body) = world.get::<&mut DynamicBody>(entity) {

            let target_kinematic = Kinematic { position: vec2(self.x, self.y), ..Default::default() };
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
        get_entity_position(world, self.entity_id)
    }

    fn tick(&self, entity: Entity, world: &mut World, dt: f32) {
        todo!()
    }
}

#[derive(Debug, Copy, Clone, SerJson, DeJson)]
pub struct AttackMoveOrder {
    entity_id: EntityID
}

impl Order for AttackMoveOrder {
    fn is_order_completed(&self, entity: Entity, world: &World) -> bool {
        todo!()
    }

    fn get_target_position(&self, world: &World) -> Option<Vec2> {
        get_entity_position(world, self.entity_id)
    }

    fn tick(&self, entity: Entity, world: &mut World, dt: f32) {
        todo!()
    }
}