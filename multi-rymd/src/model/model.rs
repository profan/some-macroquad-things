use std::collections::HashMap;

use hecs::{CommandBuffer, Entity, World};
use macroquad::{*, math::vec2};
use nanoserde::DeJson;
use utility::AsVector;
use utility::RotatedBy;

use crate::EntityID;
use crate::model::BlueprintID;
use crate::model::GameMessage;
use crate::game::RymdGameParameters;

use super::AnimatedSprite;
use super::{create_simple_bullet, Effect};
use super::get_entity_position;
use super::point_ship_towards_target;

use super::Attackable;
use super::Attacker;
use super::BlueprintIdentity;
use super::Constructor;
use super::Controller;
use super::Energy;
use super::EntityState;
use super::GameOrderType;
use super::Health;
use super::HealthCallback;
use super::Metal;
use super::PhysicsBody;
use super::PhysicsManager;
use super::Player;
use super::Consumer;
use super::Powered;
use super::Producer;
use super::Projectile;
use super::Storage;
use super::consume_energy;
use super::consume_metal;
use super::create_arrowhead_ship_blueprint;
use super::create_commander_ship_blueprint;
use super::create_energy_storage_blueprint;
use super::create_metal_storage_blueprint;
use super::create_player_entity;
use super::create_shipyard_blueprint;
use super::create_solar_collector_blueprint;
use super::current_energy;
use super::current_metal;
use super::provide_energy;
use super::provide_metal;
use super::Weapon;
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
        let metal_storage_blueprint = create_metal_storage_blueprint();
        let energy_storage_blueprint = create_energy_storage_blueprint();
        let solar_collector_blueprint = create_solar_collector_blueprint();
        let shipyard_blueprint = create_shipyard_blueprint();

        blueprints.insert(metal_storage_blueprint.id, metal_storage_blueprint);
        blueprints.insert(energy_storage_blueprint.id, energy_storage_blueprint);
        blueprints.insert(solar_collector_blueprint.id, solar_collector_blueprint);
        blueprints.insert(shipyard_blueprint.id, shipyard_blueprint);

        // units
        let commander_ship_blueprint = create_commander_ship_blueprint();
        let arrowhead_ship_blueprint = create_arrowhead_ship_blueprint();

        blueprints.insert(commander_ship_blueprint.id, commander_ship_blueprint);
        blueprints.insert(arrowhead_ship_blueprint.id, arrowhead_ship_blueprint);

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

            create_player_entity(&mut self.world, player.id);

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

    fn tick_constructing_entities(&mut self) {

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

    fn tick_powered_entities(&mut self) {

        for (e, (state, controller, consumer, _powered)) in self.world.query::<(&mut EntityState, &Controller, &Consumer, &Powered)>().iter() {
            if *state == EntityState::Constructed {
                if consumer.energy >= current_energy(controller.id, &self.world) {
                    *state = EntityState::Inactive
                } else {
                    *state = EntityState::Constructed;
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

    fn tick_attackers(&mut self) {

        let mut attack_targets = HashMap::new();

        // search for targets in range and accumulate

        for (e, (controller, attacker, transform, orderable, &state)) in self.world.query::<(&Controller, &mut Attacker, &Transform, &Orderable, &EntityState)>().iter() {

            attacker.target = None; // reset current target every time we tick the attackers

            for (o, (other_controller, other_attackable, other_transform, &other_state)) in self.world.query::<(&Controller, &Attackable, &Transform, &EntityState)>().iter() {

                let has_same_controller = controller.id == other_controller.id;
                let is_current_order_queue_empty = orderable.is_queue_empty(GameOrderType::Order);
                let is_current_order_attack = if let Some(GameOrder::Attack(_)) = orderable.first_order(GameOrderType::Order) { true } else { false };

                if e == o || state != EntityState::Constructed || (is_current_order_queue_empty == false && is_current_order_attack == false) || has_same_controller {
                    continue
                }

                let is_in_attack_range = transform.world_position.distance(other_transform.world_position) < attacker.range;

                if is_in_attack_range {
                    let entry = attack_targets.entry(e).or_insert(vec![]);
                    entry.push(o);
                }

            }

        }

        // now filter targets and pick one

        for (e, targets) in attack_targets {

            let attacker_position = get_entity_position(&self.world, e).unwrap();

            let mut closest_target_distance = f32::MAX;
            let mut closest_target = Entity::DANGLING;
            let mut closest_target_position = None;

            for t in targets {
                let target_position = get_entity_position(&self.world, t).unwrap();
                let d = attacker_position.distance(target_position);
                if d < closest_target_distance {
                    closest_target_position = Some(target_position);
                    closest_target_distance = d;
                    closest_target = t;
                }
            }

            // #TODO: depending on unit stance, either turn towards the target, or actually pursue it when attacking

            if let Some(position) = closest_target_position {
                point_ship_towards_target(&mut self.world, e, position.x, position.y, Self::TIME_STEP);
                if let Ok(mut attacker) = self.world.get::<&mut Attacker>(e) {
                    attacker.target = Some(closest_target);
                }
            }

        }

    }

    fn tick_attacker_weapons(&mut self) {

        let mut queued_projectile_creations: Vec<Box<dyn Fn(&mut World) -> Entity>> = Vec::new();

        for (e, (controller, transform, attacker, weapon)) in self.world.query::<(&Controller, &Transform, &Attacker, &mut Weapon)>().iter() {

            if let Some(attacker) = attacker.target {

                if weapon.cooldown == 0.0 {

                    let attacker_position = get_entity_position(&self.world, attacker).unwrap();
                    let attack_direction = (attacker_position - transform.world_position).normalize();

                    let id = controller.id;
                    let creation_world_position = transform.world_position + weapon.offset.rotated_by(transform.world_rotation);

                    queued_projectile_creations.push(Box::new(move |w| create_simple_bullet(w, id, creation_world_position, attack_direction)));
                    weapon.cooldown += weapon.fire_rate;

                } else {

                    weapon.cooldown = (weapon.cooldown - Self::TIME_STEP).max(0.0);

                }

            }

        }

        for projectile_fn in queued_projectile_creations {
            projectile_fn(&mut self.world);
        }

    }

    fn tick_projectiles(&mut self) {

        for (e, (body, projectile, health)) in self.world.query_mut::<(&mut DynamicBody, &mut Projectile, &mut Health)>() {

            *body.velocity_mut() = -body.kinematic.orientation.as_vector() * projectile.velocity;
            projectile.lifetime -= Self::TIME_STEP;

            if projectile.lifetime <= 0.0 {
                health.kill();
            }

        }

    }

    fn tick_constructors(&mut self) {

        for (e, (controller, constructor)) in self.world.query::<(&Controller, &mut Constructor)>().iter() {

            // we must have a current entity we're constructing for any of this logic to make sense
            let Some(constructing_entity) = constructor.current_target else { continue; };

            let mut entity_health = self.world.get::<&mut Health>(constructing_entity).expect("entity must have health component to be possible to repair!");
            let entity_blueprint_identity = self.world.get::<&BlueprintIdentity>(constructing_entity).expect("entity must have blueprint identity to be able to identify cost!");
            let entity_blueprint = self.blueprint_manager.get_blueprint(entity_blueprint_identity.blueprint_id).expect("entity must have blueprint!");

            let entity_metal_cost = entity_blueprint.cost.metal;
            let entity_energy_cost = entity_blueprint.cost.energy;
            let entity_total_cost = entity_metal_cost + entity_energy_cost;
            let entity_total_cost_to_full_health = entity_total_cost / entity_health.full_health() as f32;
            let entity_remaining_cost_to_full_health = entity_total_cost_to_full_health * entity_health.current_health_fraction();

            let entity_metal_proportion = entity_metal_cost / entity_total_cost;
            let entity_energy_proportion = entity_energy_cost / entity_total_cost;
            let entity_remaining_metal_cost = entity_remaining_cost_to_full_health * entity_metal_proportion;
            let entity_remaining_energy_cost = entity_remaining_cost_to_full_health * entity_energy_proportion;

            let build_power_metal_cost = constructor.build_speed as f32 * entity_remaining_metal_cost;
            let build_power_energy_cost = constructor.build_speed as f32 * entity_remaining_energy_cost;

            let available_metal = current_metal(controller.id, &self.world);
            let available_energy = current_energy(controller.id, &self.world);

            let available_metal_proportion = (available_metal / build_power_metal_cost).min(1.0);
            let available_energy_proportion = (available_energy / build_power_energy_cost).min(1.0);
            let min_available_proportion = available_energy_proportion.min(available_metal_proportion);
            
            let metal_to_consume = build_power_metal_cost * available_metal_proportion;
            let energy_to_consume = build_power_energy_cost * available_energy_proportion;

            if metal_to_consume > available_metal || energy_to_consume > available_energy {
                continue;
            }

            consume_metal(controller.id, &self.world, metal_to_consume * Self::TIME_STEP);
            consume_energy(controller.id, &self.world, energy_to_consume * Self::TIME_STEP);

            let entity_health_regain_amount = (entity_health.full_health() as f32 * min_available_proportion) / 2.0;
            let entity_repair_health = ((entity_health_regain_amount * Self::TIME_STEP) as i32).min(entity_health.full_health());
            entity_health.heal(entity_repair_health);
            
        }

    }

    fn tick_resource_storage(&mut self) {

        let mut energy_pools_per_player = HashMap::new();

        for (e, (controller, storage, &state)) in self.world.query_mut::<(&Controller, &Storage, &EntityState)>() {

            if state != EntityState::Constructed {
                continue
            }

            let entry = energy_pools_per_player.entry(controller.id).or_insert((0.0, 0.0));
            entry.0 += storage.metal;
            entry.1 += storage.energy;

        }

        for (e, (player, metal, energy)) in self.world.query_mut::<(&Player, &mut Metal, &mut Energy)>() {
            if let Some(&(metal_pool_size, energy_pool_size)) = energy_pools_per_player.get(&player.id) {
                metal.pool_size = metal_pool_size;
                energy.pool_size = energy_pool_size;
            } else {
                metal.pool_size = 0.0;
                energy.pool_size = 0.0;
            }
        }

    }
    
    fn tick_resources(&mut self) {

        let mut energy_incomes_per_player = HashMap::new();

        for (e, (controller, &state, consumer, producer)) in self.world.query::<(&Controller, &EntityState, Option<&Consumer>, Option<&Producer>)>().iter() {

            let mut total_metal_income = 0.0;
            let mut total_energy_income = 0.0;

            if let Some(consumer) = consumer && state == EntityState::Constructed {
                consume_metal(controller.id, &self.world, consumer.metal * Self::TIME_STEP);
                consume_energy(controller.id, &self.world, consumer.energy * Self::TIME_STEP);
                total_metal_income -= consumer.metal;
                total_energy_income -= consumer.energy;
            }

            if let Some(producer) = producer && state == EntityState::Constructed {
                provide_metal(controller.id, &self.world, producer.metal * Self::TIME_STEP);
                provide_energy(controller.id, &self.world, producer.energy * Self::TIME_STEP);
                total_metal_income += producer.metal;
                total_energy_income += producer.energy;
            }

            let entry = energy_incomes_per_player.entry(controller.id).or_insert((0.0, 0.0));
            entry.0 += total_metal_income;
            entry.1 += total_energy_income;

        }

        for (e, (player, metal, energy)) in self.world.query_mut::<(&Player, &mut Metal, &mut Energy)>() {
            if let Some((metal_income, energy_income)) = energy_incomes_per_player.get(&player.id) {
                metal.income = *metal_income;
                energy.income = *energy_income;
            } else {
                metal.income = 0.0;
                energy.income = 0.0;
            }
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

    fn tick_effects(&mut self) {

        for (e, (sprite, effect)) in self.world.query_mut::<(Option<&mut AnimatedSprite>, &mut Effect)>() {

            if let Some(sprite) = sprite {
                sprite.current_frame = (sprite.h_frames as f32 * (1.0 - (effect.lifetime / effect.total_lifetime))) as i32;
            }

            effect.lifetime -= Self::TIME_STEP;

        }

    }

    fn tick_lifetimes(&mut self) {

        let mut destroyed_entities = Vec::new();

        for (e, health) in self.world.query_mut::<&Health>() {      
            if health.is_at_or_below_zero_health() {
                destroyed_entities.push(e);
            }
        }

        for (e, effect) in self.world.query_mut::<&Effect>() {
            if effect.lifetime <= 0.0 {
                destroyed_entities.push(e);
            }
        }

        let mut command_buffer = CommandBuffer::new();

        for e in destroyed_entities {

            if let Ok(health_callback) = self.world.get::<&HealthCallback>(e) {
                (health_callback.on_death)(&self.world, &mut command_buffer, e);
            }

            let result = self.world.despawn(e);
            
            if let Err(error) = result {
                println!("[RymdGameModel] tried to despawn non-existing entity: {:?}, should never happen!", e);
            }
        }

        command_buffer.run_on(&mut self.world);

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
        
        self.tick_constructing_entities();
        self.tick_powered_entities();
        self.tick_resource_storage();
        self.tick_orderables();
        self.tick_transforms();
        self.tick_resources();
        self.tick_attackers();
        self.tick_attacker_weapons();
        self.tick_projectiles();
        self.tick_effects();
        self.tick_constructors();
        self.tick_physics_engine();
        self.tick_transform_updates();
        self.tick_lifetimes();

    }

}