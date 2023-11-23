use hecs::{World, Entity};
use macroquad::{*, math::vec2};
use nanoserde::DeJson;
use utility::RotatedBy;

use crate::EntityID;
use crate::model::GameMessage;
use crate::game::RymdGameParameters;
use super::{create_player_ship, GameOrder, Orderable, Transform, DynamicBody};

pub struct RymdGameModel {
    pub world: World
}

impl RymdGameModel {

    const TIME_STEP: f32 = 1.0 / 60.0;

    pub fn new() -> RymdGameModel {
        RymdGameModel {
            world: World::new()
        }
    }

    pub fn start(&mut self, parameters: RymdGameParameters) {

        for player in &parameters.players {

            let random_x = rand::gen_range(200, 400);
            let random_y = rand::gen_range(200, 400);

            create_player_ship(&mut self.world, vec2(random_x as f32, random_y as f32));

        }
        
    }

    pub fn stop(&mut self) {
        self.world.clear();
    }

    fn handle_order(&mut self, entities: Vec<EntityID>, order: GameOrder, should_add: bool) {

        for entity_id in entities {
            let Some(entity) = Entity::from_bits(entity_id) else { continue; };
            if let Ok(orderable) = self.world.query_one_mut::<&mut Orderable>(entity) {
                if should_add {
                    orderable.orders.push_back(order);
                } else {
                    orderable.orders.clear();
                    orderable.orders.push_back(order);
                }
            }
        }
        
    }

    pub fn handle_message(&mut self, message: &str) {
        let msg = match GameMessage::deserialize_json(message) {
            Ok(message) => message,
            Err(err) => {
                println!("[RymdGameModel] failed to parse message: {}!", message);
                return;
            }
        };

        println!("[RymdGameModel] got message: {:?}", msg);

        match msg {
            GameMessage::Order { entities, order, add } => self.handle_order(entities, order, add),
        }
    }

    fn tick_orderables(&mut self) {

        let mut in_progress_orders = Vec::new();
        let mut completed_orders = Vec::new();

        for (e, orderable) in self.world.query::<&Orderable>().iter() {
            if let Some(order) = orderable.orders.front() {
                if order.is_order_completed(e, &self.world) {
                    completed_orders.push(e);
                } else {
                    in_progress_orders.push(e);
                }
            }
        }

        for &e in &in_progress_orders {
            if let Ok(orderable) = self.world.query_one_mut::<&Orderable>(e).cloned() {
                if let Some(order) = orderable.orders.front() {
                    order.tick(e, &mut self.world, Self::TIME_STEP);
                }
            }
        }

        for &e in &completed_orders {
            if let Ok(orderable) = self.world.query_one_mut::<&mut Orderable>(e) {
                orderable.orders.pop_front();
            }
        }

    }

    fn tick_transforms(&mut self) {

        let mut updated_transforms = Vec::new();

        for (e, transform) in self.world.query::<&Transform>().iter() {
            updated_transforms.push(
                (e, transform.calculate_transform(&self.world, e))
            );
        }

        for (e, transform) in updated_transforms {
            let _ = self.world.insert_one(e, transform);
        }

    }

    fn tick_physics_bodies(&mut self) {

        for (e, (transform, body)) in self.world.query_mut::<(&mut Transform, &mut DynamicBody)>() {

            body.kinematic.integrate(Self::TIME_STEP);
            body.kinematic.apply_friction(Self::TIME_STEP);

            transform.local_position = body.kinematic.position;
            transform.local_rotation = body.kinematic.orientation;

        }

    }

    pub fn calculate_transform(world: &World, entity: Entity, transform: &Transform) -> Transform {

        let mut current_entity = entity;
        let mut calculated_transform = Transform {
            world_position: transform.local_position,
            world_rotation: transform.local_rotation,
            local_position: transform.local_position,
            local_rotation: transform.local_rotation,
            parent: transform.parent
        };

        while let Ok(mut current_query) = world.query_one::<&Transform>(current_entity) && let Some(current_transform) = current_query.get() && let Some(parent_entity) = current_transform.parent {
            if let Ok(mut parent_query) = world.query_one::<&Transform>(parent_entity) && let Some(parent_transform) = parent_query.get() {
                calculated_transform.world_position = calculated_transform.world_position.rotated_by(parent_transform.local_rotation) + parent_transform.local_position;
                calculated_transform.world_rotation += parent_transform.local_rotation;
                current_entity = parent_entity;
            } else {
                break;
            }
        }

        calculated_transform

    }

    pub fn tick(&mut self) {
        self.tick_orderables();
        self.tick_transforms();
        self.tick_physics_bodies();
    }

}