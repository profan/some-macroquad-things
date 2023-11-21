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
        let move_unit_message = GameMessage::Order { entities: vec![entity.to_bits().into()], order: GameOrder::Move { x: target_position.x, y: target_position.y }, add: should_add };
        self.send_command(move_unit_message.serialize_json());
    }

}

impl GameOrder {

    pub fn is_order_completed(&self, entity: Entity, world: &World) -> bool {
        match self {
            GameOrder::Move { x, y } => {
                if let Ok(transform) = world.get::<&Transform>(entity) {
                    let arbitrary_distance_threshold = 64.0;
                    transform.world_position.distance(vec2(*x, *y)) < arbitrary_distance_threshold
                } else {
                    false
                }
            },
            GameOrder::Attack { entity_id } => todo!(),
            GameOrder::AttackMove { entity_id } => todo!(),
        }
    }

    pub fn get_target_position(&self, world: &World) -> Option<Vec2> {
        match self {
            GameOrder::Move { x, y } => Some(vec2(*x, *y)),
            GameOrder::Attack { entity_id } => get_entity_position(world, *entity_id),
            GameOrder::AttackMove { entity_id } => get_entity_position(world, *entity_id),
        }
    }

    pub fn tick(&self, entity: Entity, world: &mut World, dt: f32) {
        match self {
            GameOrder::Move { x, y } => {

                if let Ok(mut dynamic_body) = world.get::<&mut DynamicBody>(entity) {

                    let target_kinematic = Kinematic { position: vec2(*x, *y), ..Default::default() };
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

            },
            GameOrder::Attack { entity_id } => (),
            GameOrder::AttackMove { entity_id } => (),
        }
    }
 
}

#[derive(Debug, Copy, Clone, SerJson, DeJson)]
pub enum GameOrder {
    Move {x : f32, y: f32 },
    Attack { entity_id: EntityID },
    AttackMove { entity_id: EntityID },
}