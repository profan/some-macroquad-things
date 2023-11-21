use hecs::Entity;
use lockstep_client::step::LockstepClient;
use macroquad::prelude::*;
use nanoserde::SerJson;

use crate::{GameMessage, GameOrder};

pub trait GameOrdersExt {
    fn send_move_order(&mut self, entity: Entity, target_position: Vec2, should_add: bool);
}

impl GameOrdersExt for LockstepClient {

    fn send_move_order(&mut self, entity: Entity, target_position: Vec2, should_add: bool) {
        let move_unit_message = GameMessage::Order { entities: vec![entity.to_bits().into()], order: GameOrder::Move { x: target_position.x, y: target_position.y }, add: should_add };
        self.send_command(move_unit_message.serialize_json());
    }

}