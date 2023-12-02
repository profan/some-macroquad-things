use std::collections::HashMap;

use hecs::{World, Entity};
use macroquad::{*, math::vec2};
use nanoserde::DeJson;
use utility::RotatedBy;

use crate::EntityID;
use crate::model::BlueprintID;
use crate::model::GameMessage;
use crate::game::RymdGameParameters;

use super::PhysicsManager;
use super::create_solar_collector_blueprint;
use super::{create_commander_ship, GameOrder, Orderable, Transform, DynamicBody, Blueprint};

pub struct RymdGameModel {
    pub physics_manager: PhysicsManager,
    pub blueprint_manager: BlueprintManager,
    pub world: World
}

pub struct BlueprintManager {
    blueprints: HashMap<BlueprintID, Blueprint>
}

impl BlueprintManager {

    pub fn new() -> BlueprintManager {

        let mut blueprints = HashMap::new();
        let solar_collector_blueprint = create_solar_collector_blueprint();
        blueprints.insert(solar_collector_blueprint.id, solar_collector_blueprint);

        BlueprintManager {
            blueprints
        }

    }

    pub fn get_blueprint(&self, id: BlueprintID) -> Option<&Blueprint> {
        self.blueprints.get(&id)
    }

}

impl RymdGameModel {

    const TIME_STEP: f32 = 1.0 / 60.0;

    pub fn new() -> RymdGameModel {
        RymdGameModel {
            physics_manager: PhysicsManager::new(Self::TIME_STEP),
            blueprint_manager: BlueprintManager::new(),
            world: World::new()
        }
    }

    pub fn start(&mut self, parameters: RymdGameParameters) {

        for player in &parameters.players {

            for i in 0..16 {

                let random_x = rand::gen_range(200, 400);
                let random_y = rand::gen_range(200, 400);

                create_commander_ship(&mut self.world, player.id, vec2(random_x as f32, random_y as f32));

            }
            
        }
        
    }

    pub fn stop(&mut self) {
        self.physics_manager.clear();
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
                if order.is_order_completed(e, &self) {
                    completed_orders.push(e);
                } else {
                    in_progress_orders.push(e);
                }
            }
        }

        for &e in &in_progress_orders {
            if let Ok(mut orderable) = self.world.query_one_mut::<&mut Orderable>(e).cloned() {
                if let Some(order) = orderable.orders.front_mut() {
                    order.tick(e, self, Self::TIME_STEP);
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

    fn tick_transform_updates(&mut self) {

        for (e, (transform, body)) in self.world.query_mut::<(&mut Transform, &mut DynamicBody)>() {
            transform.local_position = body.kinematic.position;
            transform.local_rotation = body.kinematic.orientation;
        }

    }

    fn tick_physics_engine(&mut self) {
        self.physics_manager.integrate(&mut self.world);
        self.physics_manager.handle_overlaps(&mut self.world);
        self.physics_manager.handle_collisions(&mut self.world);
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

        while let Ok(current_transform) = world.get::<&Transform>(current_entity) && let Some(parent_entity) = current_transform.parent {
            if let Ok(parent_transform) = world.get::<&Transform>(parent_entity) {
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
        self.tick_physics_engine();
        self.tick_transform_updates();
    }

}