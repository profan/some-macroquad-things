use std::f32::consts::PI;

use hecs::{Entity, World};
use macroquad::prelude::*;
use nanoserde::{SerJson, DeJson};
use lockstep_client::step::LockstepClient;
use utility::RotatedBy;

use crate::EntityID;
use crate::model::GameMessage;


use super::are_players_allied;
use super::set_movement_target_to_position;
use super::set_rotation_target_to_position;
use super::Attacker;
use super::BlueprintID;
use super::DynamicBody;
use super::EntityState;
use super::Extractor;
use super::PhysicsBody;
use super::get_entity_position;
use super::get_entity_position_from_id;
use super::get_closest_position_with_entity_bounds;
use super::{RymdGameModel, Constructor, Controller, Health, Orderable};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GameOrderType {
    Order,
    Construct,
}

pub trait GameOrdersExt {

    fn send_attack_order(&mut self, entity: Entity, target: Entity, should_add: bool);
    fn send_attack_move_order(&mut self, entity: Entity, target_position: Vec2, should_add: bool);
    fn send_move_order(&mut self, entity: Entity, target_position: Vec2, should_add: bool);
    fn send_build_order(&mut self, entity: Entity, target_position: Vec2, blueprint_id: BlueprintID, should_add: bool, is_self: bool);
    fn send_repair_order(&mut self, entity: Entity, target_position: Vec2, target: Entity, should_add: bool);
    fn send_extract_order(&mut self, entity: Entity, target: Entity, should_add: bool);
    fn cancel_current_orders(&mut self, entity: Entity);

}

impl GameOrdersExt for LockstepClient {

    fn send_attack_order(&mut self, entity: Entity, target_entity: Entity, should_add: bool) {
        let attack_order = GameOrder::Attack(AttackOrder { entity_id: target_entity.to_bits().into() });
        let attack_unit_message = GameMessage::Order { entity: entity.to_bits().into(), order: attack_order, add: should_add };
        self.send_command(attack_unit_message.serialize_json());
    }

    fn send_attack_move_order(&mut self, entity: Entity, target_position: Vec2, should_add: bool) {
        let attack_move_order = GameOrder::AttackMove(AttackMoveOrder { x: target_position.x, y: target_position.y });
        let attack_move_order_message = GameMessage::Order { entity: entity.to_bits().into(), order: attack_move_order, add: should_add };
        self.send_command(attack_move_order_message.serialize_json());  
    }

    fn send_move_order(&mut self, entity: Entity, target_position: Vec2, should_add: bool) {
        let move_order = GameOrder::Move(MoveOrder { x: target_position.x, y: target_position.y });
        let move_unit_message = GameMessage::Order { entity: entity.to_bits().into(), order: move_order, add: should_add };
        self.send_command(move_unit_message.serialize_json());
    }

    fn send_build_order(&mut self, entity: Entity, target_position: Vec2, blueprint_id: BlueprintID, should_add: bool, is_self: bool) {
        let build_order = GameOrder::Construct(ConstructOrder { entity_id: None, blueprint_id: Some(blueprint_id), is_self_order: is_self, x: target_position.x, y: target_position.y });
        let build_unit_message = GameMessage::Order { entity: entity.to_bits().into(), order: build_order, add: should_add || is_self };
        self.send_command(build_unit_message.serialize_json());
    }

    fn send_repair_order(&mut self, entity: Entity, target_position: Vec2, target: Entity, should_add: bool) {
        let build_order = GameOrder::Construct(ConstructOrder { entity_id: Some(target.to_bits().get()), blueprint_id: None, is_self_order: false, x: target_position.x, y: target_position.y });
        let build_unit_message = GameMessage::Order { entity: entity.to_bits().into(), order: build_order, add: should_add };
        self.send_command(build_unit_message.serialize_json());
    }

    fn cancel_current_orders(&mut self, entity: Entity) {
        let cancel_order = GameOrder::Cancel(CancelOrder {});
        let cancel_order_message = GameMessage::Order { entity: entity.to_bits().into(), order: cancel_order, add: false };
        self.send_command(cancel_order_message.serialize_json());
    }
    
    fn send_extract_order(&mut self, entity: Entity, target: Entity, should_add: bool) {
        let extract_order = GameOrder::Extract(ExtractOrder { entity_id: target.to_bits().get() });
        let extract_order_message = GameMessage::Order { entity: entity.to_bits().into(), order: extract_order, add: should_add };
        self.send_command(extract_order_message.serialize_json());
    }

}

trait Order {

    fn is_order_valid(&self, entity: Entity, model: &RymdGameModel) -> bool {
        self.is_order_completed(entity, model) == false
    }

    fn is_order_completed(&self, entity: Entity, model: &RymdGameModel) -> bool;
    fn get_target_position(&self, model: &RymdGameModel) -> Option<Vec2> { None }
    fn tick(&self, entity: Entity, model: &mut RymdGameModel, dt: f32);
    fn on_completed(&self, entity: Entity, model: &mut RymdGameModel) {}

}

impl GameOrder {

    pub fn is_order_valid(&self, entity: Entity, model: &RymdGameModel) -> bool {
        match self {
            GameOrder::Move(order) => order.is_order_valid(entity, model),
            GameOrder::Attack(order) => order.is_order_valid(entity, model),
            GameOrder::AttackMove(order) => order.is_order_valid(entity, model),
            GameOrder::Construct(order) => order.is_order_valid(entity, model),
            GameOrder::Extract(order) => order.is_order_valid(entity, model),
            GameOrder::Cancel(order) => order.is_order_valid(entity, model)
        }
    }

    pub fn is_order_completed(&self, entity: Entity, model: &RymdGameModel) -> bool {
        match self {
            GameOrder::Move(order) => order.is_order_completed(entity, model),
            GameOrder::Attack(order) => order.is_order_completed(entity, model),
            GameOrder::AttackMove(order) => order.is_order_completed(entity, model),
            GameOrder::Construct(order) => order.is_order_completed(entity, model),
            GameOrder::Extract(order) => order.is_order_completed(entity, model),
            GameOrder::Cancel(order) => order.is_order_completed(entity, model)
        }
    }

    pub fn get_target_position(&self, model: &RymdGameModel) -> Option<Vec2> {
        match self {
            GameOrder::Move(order) => order.get_target_position(model),
            GameOrder::Attack(order) => order.get_target_position(model),
            GameOrder::AttackMove(order) => order.get_target_position(model),
            GameOrder::Construct(order) => order.get_target_position(model),
            GameOrder::Extract(order) => order.get_target_position(model),
            GameOrder::Cancel(order) => order.get_target_position(model)
        }
    }

    pub fn tick(&self, entity: Entity, model: &mut RymdGameModel, dt: f32) {
        match self {
            GameOrder::Move(order) => order.tick(entity, model, dt),
            GameOrder::Attack(order) => order.tick(entity, model, dt),
            GameOrder::AttackMove(order) => order.tick(entity, model, dt),
            GameOrder::Construct(order) => order.tick(entity, model, dt),
            GameOrder::Extract(order) => order.tick(entity, model, dt),
            GameOrder::Cancel(order) => order.tick(entity, model, dt)
        }
    }

    pub fn on_order_completed(&self, entity: Entity, model: &mut RymdGameModel) {
        match self {
            GameOrder::Move(order) => order.on_completed(entity, model),
            GameOrder::Attack(order) => order.on_completed(entity, model),
            GameOrder::AttackMove(order) => order.on_completed(entity, model),
            GameOrder::Construct(order) => order.on_completed(entity, model),
            GameOrder::Extract(order) => order.on_completed(entity, model),
            GameOrder::Cancel(order) => order.on_completed(entity, model)
        }     
    }

    pub fn order_type(&self) -> GameOrderType {
        match self {
            GameOrder::Move(_) => GameOrderType::Order,
            GameOrder::Attack(_) => GameOrderType::Order,
            GameOrder::AttackMove(_) => GameOrderType::Order,
            GameOrder::Construct(order) => if order.is_self_order { GameOrderType::Construct } else { GameOrderType::Order },
            GameOrder::Extract(order) => GameOrderType::Order,
            GameOrder::Cancel(_) => GameOrderType::Order
        }
    }
 
}

#[derive(Debug, Copy, Clone, SerJson, DeJson)]
pub enum GameOrder {
    Move(MoveOrder),
    Attack(AttackOrder),
    AttackMove(AttackMoveOrder),
    Construct(ConstructOrder),
    Extract(ExtractOrder),
    Cancel(CancelOrder)
}

#[derive(Debug, Copy, Clone, SerJson, DeJson)]
pub struct MoveOrder {
    x: f32,
    y: f32
}

impl Order for MoveOrder {
    fn is_order_completed(&self, entity: Entity, model: &RymdGameModel) -> bool {
        let arbitrary_distance_threshold_squared = 64.0 * 64.0;
        let position = get_entity_position(&model.world, entity).expect("could not get position for move order, should never happen!");
        position.distance_squared(vec2(self.x, self.y)) < arbitrary_distance_threshold_squared
    }

    fn get_target_position(&self, model: &RymdGameModel) -> Option<Vec2> {
        Some(vec2(self.x, self.y))
    }

    fn tick(&self, entity: Entity, model: &mut RymdGameModel, dt: f32) {
        set_movement_target_to_position(&model.world, entity, self.get_target_position(model));
    }

    fn on_completed(&self, entity: Entity, model: &mut RymdGameModel) {
        set_movement_target_to_position(&model.world, entity, None);
    }
}

#[derive(Debug, Copy, Clone, SerJson, DeJson)]
pub struct AttackOrder {
    entity_id: EntityID
}

impl AttackOrder {
    pub fn entity(&self) -> Option<Entity> {
        Entity::from_bits(self.entity_id)
    }
}

impl Order for AttackOrder {
    fn is_order_completed(&self, entity: Entity, model: &RymdGameModel) -> bool {
        model.world.contains(entity) == false
    }

    fn get_target_position(&self, model: &RymdGameModel) -> Option<Vec2> {
        get_entity_position(&model.world, self.entity()?)
    }

    fn tick(&self, entity: Entity, model: &mut RymdGameModel, dt: f32) {

        // if we don't have no target position just yeet ourselves
        let Some(target_position) = self.get_target_position(model) else { return };

        let attacker = model.world.get::<&Attacker>(entity).expect("must have attacker component to attack!");
        let attacker_position = get_entity_position(&model.world, entity).expect("must have position!");
        let attack_range = attacker.range;
        drop(attacker);

        if attacker_position.distance(target_position) > attack_range {
            set_movement_target_to_position(&model.world, entity, Some(target_position));
        } else {
            let mut attacker = model.world.get::<&mut Attacker>(entity).expect("must have attacker component to attack!");
            attacker.target = self.entity();
        }

    }

    fn on_completed(&self, entity: Entity, model: &mut RymdGameModel) {
        set_movement_target_to_position(&mut model.world, entity, None);
    }

}

#[derive(Debug, Copy, Clone, SerJson, DeJson)]
pub struct AttackMoveOrder {
    x: f32,
    y: f32
}

impl Order for AttackMoveOrder {
    fn is_order_completed(&self, entity: Entity, model: &RymdGameModel) -> bool {
        let arbitrary_distance_threshold_squared = 64.0 * 64.0;
        let position = get_entity_position(&model.world, entity).expect("could not get position for attack move order, should never happen!");
        position.distance_squared(vec2(self.x, self.y)) < arbitrary_distance_threshold_squared
    }

    fn get_target_position(&self, model: &RymdGameModel) -> Option<Vec2> {
        Some(vec2(self.x, self.y))
    }

    fn tick(&self, entity: Entity, model: &mut RymdGameModel, dt: f32) {
        let attacker = model.world.get::<&Attacker>(entity);
        let extractor = model.world.get::<&mut Extractor>(entity);

        // if the entity is an attacker and there's a current target, unset the current movement target
        let target_position = if let Ok(attacker) = attacker && attacker.target.is_some() {
           None
        } else if let Ok(mut extractor) = extractor {
            if extractor.current_target.is_none() {
                extractor.is_searching = true;
                self.get_target_position(model)
            } else {
                None
            }
        } else {
            // if the entity isn't an attacker, just have them move instead of attack
            self.get_target_position(model)
        };

        set_movement_target_to_position(&model.world, entity, target_position);
    }

    fn on_completed(&self, entity: Entity, model: &mut RymdGameModel) {
        set_movement_target_to_position(&model.world, entity, None);
    }
}

#[derive(Debug, Copy, Clone, SerJson, DeJson)]
pub struct ConstructOrder {
    pub entity_id: Option<EntityID>,
    pub blueprint_id: Option<BlueprintID>,
    pub is_self_order: bool,
    pub x: f32,
    pub y: f32
}

impl ConstructOrder {
    pub fn entity(&self) -> Option<Entity> {
        Entity::from_bits(self.entity_id?)
    }
}

/// Returns the constructible entity intersecting with the specific position, if any
fn constructible_at_position(world: &World, position: Vec2) -> Option<Entity> {
    for (e, (body, health, state)) in world.query::<(&DynamicBody, &Health, &EntityState)>().iter() {
        if body.bounds().contains(position) && *state == EntityState::Ghost {
            return Some(e);
        }
    }
    None
}

/// Returns true if there's an existing static body at the given position.
fn existing_static_body_at_position(world: &World, position: Vec2) -> bool {
    for (e, (body, health, state)) in world.query::<(&DynamicBody, &Health, &EntityState)>().iter() {
        if body.bounds().contains(position) && body.is_static {
            return true;
        }
    }
    false
}

impl Order for ConstructOrder {

    fn is_order_completed(&self, entity: Entity, model: &RymdGameModel) -> bool {
        
        if let Some(constructing_entity) = self.entity() {

            // #FIXME: we should resolve these lifetime issues properly somehow
            if model.world.contains(constructing_entity) == false {
                return true
            }

            let entity_health = model.world.get::<&Health>(constructing_entity).expect("entity must have entity health component to be able to construct!");
            entity_health.is_at_full_health()

        } else {
            false
        }

    }

    fn get_target_position(&self, model: &RymdGameModel) -> Option<Vec2> {
        if let Some(entity_id) = self.entity_id {
            get_entity_position_from_id(&model.world, entity_id)
        } else {
            Some(vec2(self.x, self.y))
        }
    }

    fn tick(&self, entity: Entity, model: &mut RymdGameModel, dt: f32) {

        // we're building/repairing an existing construction, this could be tick 2 of the "constructing a blueprint" case
        if let Some(entity_id) = self.entity_id && let Some(constructing_entity) = Entity::from_bits(entity_id) {

            let target_position = get_entity_position_from_id(&model.world, entity_id).expect("could not unpack target position?");
            if self.is_self_order == false && self.is_within_constructor_range(entity, &model.world, target_position) == false {
                set_movement_target_to_position(&model.world, entity, Some(target_position));
                return;
            }

            if self.is_self_order == false && self.is_within_constructor_range(entity, &model.world, target_position) {
                set_rotation_target_to_position(&model.world, entity, Some(target_position));
            }

            let mut constructor = model.world.get::<&mut Constructor>(entity).expect("must have constructor to be issuing construct order!");
            constructor.current_target = Some(constructing_entity);

        }

        // we're constructing something someone else seems to have started!
        if let Some(existing_constructible) = constructible_at_position(&model.world, vec2(self.x, self.y)) && self.is_self_order == false {
            
            // cancel our current order now
            let order_type = if self.is_self_order { GameOrderType::Construct } else { GameOrderType::Order };
            self.cancel_current_order(entity, &mut model.world, order_type);

            // issue order to go construct the new thing!
            self.construct_external_entity(entity, &mut model.world, existing_constructible, vec2(self.x, self.y));

        }

        // we're constructing something new given a blueprint
        if let Some(blueprint_id) = self.blueprint_id && existing_static_body_at_position(&model.world, vec2(self.x, self.y)) == false {

            let construction_position = vec2(self.x, self.y);

            if self.is_self_order == false && self.is_within_constructor_range(entity, &model.world, construction_position) == false {
                set_movement_target_to_position(&model.world, entity, Some(construction_position));
                return;
            }

            let controller_id = model.world.get::<&Controller>(entity).expect("must have controller to be issuing construct order!").id;
            let blueprint = model.blueprint_manager.get_blueprint(blueprint_id);
            
            if let Some(blueprint) = blueprint {

                let new_entity_id = (blueprint.constructor)(&mut model.world, controller_id, construction_position);

                // cancel our current order now
                let order_type = if self.is_self_order { GameOrderType::Construct } else { GameOrderType::Order };
                self.cancel_current_order(entity, &mut model.world, order_type);
    
                // now that this is created, issue a local order to ourselves to help build this new entity
                if self.is_self_order {
                    self.construct_internal_entity(entity, &mut model.world, new_entity_id, construction_position);
                } else {
                    self.construct_external_entity(entity, &mut model.world, new_entity_id, construction_position);
                }

            }

        }

    }

    fn on_completed(&self, entity: Entity, model: &mut RymdGameModel) {

        {
            let mut constructor = model.world.get::<&mut Constructor>(entity).expect("must have constructor to be issuing construct order!");
            constructor.current_target = None;
        }

        if let Some(new_entity) = self.entity() && entity != new_entity {
            self.inherit_orders_from_constructor_if_empty(entity, new_entity, model);
        }

        set_movement_target_to_position(&model.world, entity, None);

        if self.is_self_order == false {
            set_rotation_target_to_position(&model.world, entity, None);
        }

        let our_position = get_entity_position(&model.world, entity).unwrap();
        if self.is_self_order == false && existing_static_body_at_position(&model.world, our_position) && has_pending_orders(&model.world, entity, GameOrderType::Order) == false {
            let Some(constructed_entity) = self.entity() else { return; };
            let Some(target_position) = Self::calculate_movement_position_out_of_constructor(our_position, model, constructed_entity, true) else { return; };
            self.move_entity_to_position(entity, &mut model.world, target_position);
        }

    }
    
}

impl ConstructOrder {

    fn is_within_constructor_range(&self, entity: Entity, world: &World, target: Vec2) -> bool {

        if let Some((entity_position, bounds)) = get_closest_position_with_entity_bounds(world, entity) {
            let constructor = world.get::<&Constructor>(entity).expect("must have constructor to be issuing construct order!");
            (entity_position.distance(target) as i32) < constructor.build_range + (bounds.size().max_element() / 2.0) as i32
        } else {
            let entity_position = get_entity_position_from_id(world, entity.to_bits().get()).expect("must have position!");
            let constructor = world.get::<&Constructor>(entity).expect("must have constructor to be issuing construct order!");
            (entity_position.distance(target) as i32) < constructor.build_range
        }

    }

    fn cancel_current_order(&self, entity: Entity, world: &mut World, order_type: GameOrderType) {
        let mut orderable = world.get::<&mut Orderable>(entity).expect("must have orderable!");
        orderable.cancel_order(order_type);
    }

    fn construct_external_entity(&self, entity: Entity, world: &mut World, building_entity: Entity, position: Vec2) {
        let mut orderable = world.get::<&mut Orderable>(entity).expect("must have orderable!");
        orderable.push_order(GameOrder::Construct(ConstructOrder { entity_id: Some(building_entity.to_bits().get()), blueprint_id: None, is_self_order: false, x: position.x, y: position.y }))
    }

    fn construct_internal_entity(&self, entity: Entity, world: &mut World, building_entity: Entity, position: Vec2) {
        let mut orderable = world.get::<&mut Orderable>(entity).expect("must have orderable!");
        orderable.push_order(GameOrder::Construct(ConstructOrder { entity_id: Some(building_entity.to_bits().get()), blueprint_id: None, is_self_order: true, x: position.x, y: position.y }))  
    }

    fn move_entity_to_position(&self, new_entity: Entity, world: &mut World, position: Vec2) {
        let mut orderable = world.get::<&mut Orderable>(new_entity).expect("must have orderable!");
        orderable.push_order(GameOrder::Move(MoveOrder { x: position.x, y: position.y }));
    }

    fn is_entity_a_constructor(model: &RymdGameModel, new_entity: Entity) -> bool {
        model.world.satisfies::<&Constructor>(new_entity).unwrap()
    }

    fn inherit_orders_from_constructor_if_empty(&self, constructor_entity: Entity, new_entity: Entity, model: &mut RymdGameModel) {

        if constructor_entity == new_entity {
            warn!("ConstructOrder - inherit_orders_from_constructor_if_empty: tried to inherit orders from self?");
            return;
        }

        {

            let mut orderable_query = model.world.query::<&mut Orderable>();
            let mut orderable_view = orderable_query.view();
            let [constructor_orderable, new_orderable] = orderable_view.get_many_mut([constructor_entity, new_entity]);

            if let Some(constructor_orderable) = constructor_orderable
                && let Some(new_orderable) = new_orderable
                && constructor_orderable.is_queue_empty(GameOrderType::Order) == false
                && new_orderable.is_queue_empty(GameOrderType::Order)
            {
                for order in constructor_orderable.orders(GameOrderType::Order) {
                    if Self::is_entity_a_constructor(model, new_entity) == false {
                        if let GameOrder::Construct(_) = order {
                            continue;
                        }
                    }
                    new_orderable.queue_order(*order);
                }
            }

        }

        let our_position = vec2(self.x, self.y);
        let has_orderable = model.world.get::<&Orderable>(new_entity).is_ok();
        let target_position = Self::calculate_movement_position_out_of_constructor(our_position, model, constructor_entity, has_orderable);

        if let Some(target_position) = target_position {
            self.move_entity_to_position(new_entity, &mut model.world, target_position);
        }

    }

    fn calculate_movement_position_out_of_constructor(position: Vec2, model: &mut RymdGameModel, entity: Entity, has_orderable: bool) -> Option<Vec2> {
        let target_position = if let Ok(body) = model.world.get::<&DynamicBody>(entity) && has_orderable {
            let angle_range = PI / 4.0;
            let random_angle = model.random.gen_range(-angle_range, angle_range);
            let dir_to_entity = (position - body.position()).normalize();
            let target_position = body.position() + dir_to_entity.rotated_by(random_angle) * body.bounds.size().max_element();
            Some(target_position)
        } else {
            None
        };
        target_position
    }

}

#[derive(Debug, Copy, Clone, SerJson, DeJson)]
pub struct CancelOrder {

}

impl Order for CancelOrder {
    fn is_order_completed(&self, entity: Entity, model: &RymdGameModel) -> bool {
        has_any_pending_orders(&model.world, entity) == false
    }

    fn tick(&self, entity: Entity, model: &mut RymdGameModel, dt: f32) {
        cancel_pending_orders(&model.world, entity);
    }
}

pub fn has_pending_orders(world: &World, entity: Entity, order_type: GameOrderType) -> bool {
    if let Ok(orderable) = world.get::<&Orderable>(entity) {
        orderable.has_pending_orders(order_type)
    } else {
        false
    }
}

pub fn has_any_pending_orders(world: &World, entity: Entity) -> bool {
    if let Ok(orderable) = world.get::<&Orderable>(entity) {
        orderable.has_pending_orders(GameOrderType::Order)|| orderable.has_pending_orders(GameOrderType::Construct)
    } else {
        false
    }
}

pub fn cancel_pending_orders(world: &World, entity: Entity) {
    if let Ok(mut orderable) = world.get::<&mut Orderable>(entity) {
        orderable.cancel_orders(GameOrderType::Construct);
        orderable.cancel_orders(GameOrderType::Order);
    }
}

#[derive(Debug, Copy, Clone, SerJson, DeJson)]
pub struct ExtractOrder {
    pub entity_id: EntityID
}

impl ExtractOrder {

    pub fn entity(&self) -> Entity {
        Entity::from_bits(self.entity_id).unwrap()
    }

}

impl Order for ExtractOrder {
    fn is_order_completed(&self, entity: Entity, model: &RymdGameModel) -> bool {
        model.world.contains(self.entity()) == false
    }

    fn get_target_position(&self, model: &RymdGameModel) -> Option<Vec2> {
        get_entity_position(&model.world, self.entity())
    }

    fn tick(&self, entity: Entity, model: &mut RymdGameModel, dt: f32) {      
        let mut extractor = model.world.get::<&mut Extractor>(entity).expect("can't issue an extract order to something without an Extractor component!");
        extractor.current_target = Some(self.entity());   
    }

    fn on_completed(&self, entity: Entity, model: &mut RymdGameModel) {
        let mut extractor = model.world.get::<&mut Extractor>(entity).expect("can't issue an extract order to something without an Extractor component!");
        extractor.current_target = None;
    }
}

pub fn is_within_extractor_range(entity: Entity, world: &World, target: Vec2) -> bool {

    let extractor = world.get::<&Extractor>(entity).expect("must have extractor to be asking about if within extraction range!");
    is_within_extractor_range_with_extractor(entity, world, &extractor, target)

}

pub fn is_within_extractor_range_with_extractor(entity: Entity, world: &World, extractor: &Extractor, target: Vec2) -> bool {

    if let Some((entity_position, bounds)) = get_closest_position_with_entity_bounds(world, entity) {
        (entity_position.distance(target) as i32) < extractor.extraction_range + (bounds.size().max_element() / 2.0) as i32
    } else {
        let entity_position = get_entity_position_from_id(world, entity.to_bits().get()).expect("must have position!");
        (entity_position.distance(target) as i32) < extractor.extraction_range
    }

}

pub fn number_of_pending_orders(world: &World, entity: Entity, order_type: GameOrderType) -> i32 {
    if let Ok(orderable) = world.get::<&Orderable>(entity) {
        orderable.number_of_pending_orders(order_type)
    } else {
        0
    }
}

pub trait OrdersExt {
    fn is_current_order_move_order(&self) -> bool;
    fn is_current_order_attack_order(&self) -> bool;
    fn is_current_order_attack_move_order(&self) -> bool;
    fn is_current_order_construct_order(&self, queue_type: GameOrderType) -> bool;
    fn is_current_order_extract_order(&self) -> bool;
}

impl OrdersExt for Orderable {
    fn is_current_order_move_order(&self) -> bool {
        if let Some(GameOrder::Move(_)) = self.first_order(GameOrderType::Order) {
            true
        } else {
            false
        }
    }

    fn is_current_order_attack_order(&self) -> bool {
        if let Some(GameOrder::Attack(_)) = self.first_order(GameOrderType::Order) {
            true
        } else {
            false
        }
    }
    
    fn is_current_order_attack_move_order(&self) -> bool {
        if let Some(GameOrder::AttackMove(_)) = self.first_order(GameOrderType::Order) {
            true
        } else {
            false
        }
    }
    
    fn is_current_order_construct_order(&self, queue_type: GameOrderType) -> bool {
        if let Some(GameOrder::Construct(_)) = self.first_order(queue_type) {
            true
        } else {
            false
        }
    }

    fn is_current_order_extract_order(&self) -> bool {
        if let Some(GameOrder::Extract(_)) = self.first_order(GameOrderType::Order) {
            true
        } else {
            false
        }
    }
}