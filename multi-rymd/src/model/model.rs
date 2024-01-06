use std::collections::HashMap;

use hecs::{World, Entity};
use macroquad::{*, math::vec2};
use nanoserde::DeJson;
use utility::RotatedBy;

use crate::EntityID;
use crate::model::BlueprintID;
use crate::model::GameMessage;
use crate::game::RymdGameParameters;

use super::EntityState;
use super::GameOrderType;
use super::Health;
use super::PhysicsManager;
use super::create_commander_ship_blueprint;
use super::create_shipyard_blueprint;
use super::create_solar_collector_blueprint;
use super::{build_commander_ship, GameOrder, Orderable, Transform, DynamicBody, Blueprint};

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

        // buildings
        let solar_collector_blueprint = create_solar_collector_blueprint();
        let shipyard_blueprint = create_shipyard_blueprint();

        blueprints.insert(solar_collector_blueprint.id, solar_collector_blueprint);
        blueprints.insert(shipyard_blueprint.id, shipyard_blueprint);

        // units
        let commander_ship_blueprint = create_commander_ship_blueprint();

        blueprints.insert(commander_ship_blueprint.id, commander_ship_blueprint);

        BlueprintManager {
            blueprints
        }

    }

    pub fn get_blueprint(&self, id: BlueprintID) -> Option<&Blueprint> {
        self.blueprints.get(&id)
    }

}

impl RymdGameModel {

    pub const TIME_STEP: f32 = 1.0 / 60.0;

    pub fn new() -> RymdGameModel {
        RymdGameModel {
            physics_manager: PhysicsManager::new(Self::TIME_STEP),
            blueprint_manager: BlueprintManager::new(),
            world: World::new()
        }
    }

    pub fn start(&mut self, parameters: RymdGameParameters) {

        for player in &parameters.players {

            for i in 0..1 {

                let random_x = rand::gen_range(200, 400);
                let random_y = rand::gen_range(200, 400);

                let commander_ship = build_commander_ship(&mut self.world, player.id, vec2(random_x as f32, random_y as f32));
                if let Ok(mut health) = self.world.get::<&mut Health>(commander_ship) {
                    health.heal_to_full_health();
                }

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
                    orderable.queue_order(order);
                } else {
                    orderable.cancel_orders(order.order_type());
                    orderable.queue_order(order);
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

    fn tick_entity_states(&mut self) {

        for (e, (state, health, body)) in self.world.query_mut::<(&mut EntityState, &Health, Option<&mut DynamicBody>)>() {
            if *state == EntityState::Ghost {
                if health.is_at_full_health() {
                    *state = EntityState::Constructed;
                    if let Some(body) = body {
                        body.is_enabled = true;
                    }
                }
            }       
        }

    }

    fn tick_orderables(&mut self) {

        self.tick_order_queue(GameOrderType::Order);
        self.tick_order_queue(GameOrderType::Construct);

    }

    fn tick_order_queue(&mut self, order_type: GameOrderType) {

        let mut in_progress_orders = Vec::new();
        let mut completed_orders = Vec::new();
        let mut canceled_orders = Vec::new();

        for (e, (orderable, &state)) in self.world.query::<(&Orderable, &EntityState)>().iter() {

            if let Some(order) = orderable.first_order(order_type) && Self::is_processing_orders(state) {
                if order.is_order_completed(e, &self) {
                    completed_orders.push(e);
                } else {
                    in_progress_orders.push(e);
                }
            }

            if orderable.canceled_orders(order_type).is_empty() == false {
                canceled_orders.push(e);
            }

        }

        for &e in &in_progress_orders {

            let maybe_order = if let Ok(orderable) = self.world.query_one_mut::<&mut Orderable>(e) {
                orderable.first_order(order_type).cloned()
            } else {
                None
            };

            if let Some(order) = maybe_order {
                order.tick(e, self, Self::TIME_STEP);
            }

        }

        for &e in &completed_orders {

            let maybe_order = if let Ok(orderable) = self.world.query_one_mut::<&mut Orderable>(e) {
                orderable.pop_first_order(order_type)
            } else {
                None
            };

            if let Some(order) = maybe_order {
                order.on_order_completed(e, self);
            }

        }

        for &e in &canceled_orders {
            while let Ok(orderable) = self.world.query_one_mut::<&mut Orderable>(e) && let Some(order) = orderable.pop_first_canceled_order(order_type) {
                order.on_order_completed(e, self);
            }
            if let Ok(orderable) = self.world.query_one_mut::<&mut Orderable>(e) {
                orderable.clear_canceled_orders(order_type);
            }
        }

    }

    fn is_processing_orders(state: EntityState) -> bool {
        state == EntityState::Constructed
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
        self.tick_entity_states();
        self.tick_orderables();
        self.tick_transforms();
        self.tick_physics_engine();
        self.tick_transform_updates();
    }

}