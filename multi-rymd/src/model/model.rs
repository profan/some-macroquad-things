use core::f32;
use std::collections::BTreeMap;

use hecs::{CommandBuffer, Entity, World};
use macroquad::*;
use math::Vec2;
use rand::RandomRange;
use utility::random_binomial;
use utility::separation;
use utility::AsVector;
use utility::RotatedBy;

use crate::model::create_simple_beam;
use crate::model::BeamParameters;
use crate::model::BeamWeapon;
use crate::model::Steering;
use crate::EntityID;
use crate::model::BlueprintID;
use crate::model::GameMessage;
use crate::game::RymdGameParameters;
use crate::PlayerID;

use super::are_players_allied;
use super::are_players_hostile;
use super::create_commissar_ship_blueprint;
use super::create_extractor_ship_blueprint;
use super::create_grunt_ship_blueprint;
use super::create_impact_effect_in_world;
use super::entity_apply_raw_steering;
use super::is_within_extractor_range_with_extractor;
use super::spatial::entity_distance_sort_function;
use super::spatial::SpatialQueryManager;
use super::steer_entity_towards_target;
use super::AnimatedSprite;
use super::Beam;
use super::Building;
use super::BulletParameters;
use super::ExtractOrder;
use super::Extractor;
use super::MovementTarget;
use super::OrdersExt;
use super::PreviousTransform;
use super::ResourceSource;
use super::RotationTarget;
use super::UnitState;
use super::{create_simple_bullet, Effect};
use super::get_entity_position;
use super::point_entity_towards_target;

use super::Attackable;
use super::Attacker;
use super::BlueprintIdentity;
use super::Constructor;
use super::Controller;
use super::Energy;
use super::EntityState;
use super::GameOrderType;
use super::Health;
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
use super::create_shipyard_blueprint;
use super::create_solar_collector_blueprint;
use super::current_energy;
use super::current_metal;
use super::provide_energy;
use super::provide_metal;
use super::ProjectileWeapon;
use super::{GameOrder, Orderable, Transform, DynamicBody, Blueprint};

pub struct RymdGameModel {
    pub physics_manager: PhysicsManager,
    pub spatial_manager: SpatialQueryManager,
    pub blueprint_manager: BlueprintManager,
    pub player_mapping: BTreeMap<PlayerID, Player>,
    pub world: World,
    pub current_tick: u64
}

pub struct BlueprintManager {
    blueprints: BTreeMap<BlueprintID, Blueprint>
}

fn create_blue_side_blueprints() -> BTreeMap<i32, Blueprint> {

    let mut blueprints = BTreeMap::new();

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
    let extractor_ship_blueprint = create_extractor_ship_blueprint();

    blueprints.insert(commander_ship_blueprint.id, commander_ship_blueprint);
    blueprints.insert(arrowhead_ship_blueprint.id, arrowhead_ship_blueprint);
    blueprints.insert(extractor_ship_blueprint.id, extractor_ship_blueprint);

    blueprints

}

fn create_green_side_blueprints() -> BTreeMap<i32, Blueprint> {

    let mut blueprints = BTreeMap::new();

    // units
    let commissar_ship_blueprint = create_commissar_ship_blueprint();
    let grunt_ship_blueprint = create_grunt_ship_blueprint();

    blueprints.insert(commissar_ship_blueprint.id, commissar_ship_blueprint);
    blueprints.insert(grunt_ship_blueprint.id, grunt_ship_blueprint);

    blueprints

}

impl BlueprintManager {

    pub fn new() -> BlueprintManager {

        let mut blueprints = BTreeMap::new();
        
        for (k, v) in create_blue_side_blueprints() {
            blueprints.insert(k, v);
        }
        
        for (k, v) in create_green_side_blueprints() {
            blueprints.insert(k, v);
        }

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
    pub const SPATIAL_BUCKET_SIZE: i32 = 256;

    pub fn new() -> RymdGameModel {
        RymdGameModel {
            physics_manager: PhysicsManager::new(Self::TIME_STEP),
            spatial_manager: SpatialQueryManager::new(Self::SPATIAL_BUCKET_SIZE),
            blueprint_manager: BlueprintManager::new(),
            player_mapping: BTreeMap::new(),
            world: World::new(),
            current_tick: 0
        }
    }

    pub fn get_player_by_id(&self, id: PlayerID) -> &Player {
        &self.player_mapping[&id]
    }

    fn populate_player_mapping(&mut self) {
        for (e, player) in self.world.query_mut::<&Player>() {
            self.player_mapping.insert(player.id, player.clone());
        }
    }

    pub fn start(&mut self, parameters: RymdGameParameters) {
        self.populate_player_mapping();
        self.current_tick = 0;
    }

    pub fn stop(&mut self) {
        self.physics_manager.clear();
        self.world.clear();
    }

    fn handle_order(&mut self, entities: &Vec<EntityID>, order: GameOrder, should_add: bool) {

        for &entity_id in entities {
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

    pub fn handle_message(&mut self, message: &GameMessage) {

        // println!("[RymdGameModel] got message: {:?}", message);

        match message {
            GameMessage::Order { entities, order, add } => self.handle_order(entities, *order, *add),
        }

    }

    //#[profiling::function]
    fn tick_constructing_entities(&mut self) {

        for (e, (state, health, body, building)) in self.world.query_mut::<(&mut EntityState, &Health, Option<&mut DynamicBody>, Option<&Building>)>() {

            let is_entity_ghost = *state == EntityState::Ghost;
            if is_entity_ghost == false { continue; }

            let is_entity_at_full_health = health.is_at_full_health();
            if is_entity_at_full_health == false { continue; };

            // all entities that are ghosts which reach full health are then considered constructed
            *state = EntityState::Constructed;

            let Some(body) = body else { continue; };

            // only things that aren't buildings and have bodies should be made non-static after construction
            if building.is_none() {
                body.is_static = false;                            
            }

        }

    }

    //#[profiling::function]
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

    //#[profiling::function]
    fn tick_orderables(&mut self) {

        self.tick_order_queue(GameOrderType::Order);
        self.tick_order_queue(GameOrderType::Construct);

    }

    //#[profiling::function]
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

    //#[profiling::function]
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

    //#[profiling::function]
    fn tick_rotation_targets(&mut self) {

        let mut rotation_targets = Vec::new();

        for (e, rotation_target) in self.world.query::<&RotationTarget>().iter() {
            let Some(target_position) = rotation_target.target else { continue };
            rotation_targets.push((e, target_position));
        }

        for (e, target_position) in rotation_targets {
            point_entity_towards_target(&mut self.world, e, target_position.x, target_position.y, Self::TIME_STEP);
        }

    }

    //#[profiling::function]
    fn tick_movement_targets(&mut self) {

        let mut move_targets = Vec::new();

        for (e, movement_target) in self.world.query::<&MovementTarget>().iter() {
            let Some(target_position) = movement_target.target else { continue };
            move_targets.push((e, target_position));
        }

        for (e, target_position) in move_targets {
            steer_entity_towards_target(&mut self.world, e, target_position.x, target_position.y, Self::TIME_STEP);
        }

    }

    fn tick_separation(&mut self) {

        let mut steering_output_to_apply = Vec::new();

        for (e, (dynamic_body, steering)) in self.world.query::<(&DynamicBody, &Steering)>().iter() {

            if dynamic_body.is_enabled == false || dynamic_body.is_static {
                continue;
            }

            let steering_parameters = steering.parameters;
            let nearby_entities = self.spatial_manager.entities_within_radius(dynamic_body.position(), steering_parameters.separation_threshold);
            let nearby_entities_with_dynamic_body = nearby_entities.filter(|o| e != *o).filter_map(|e| self.world.get::<&DynamicBody>(e).and_then(|b| Ok(b.kinematic.clone())).ok());

            let steering_output = separation(
                &dynamic_body.kinematic,
                nearby_entities_with_dynamic_body,
                steering_parameters.max_acceleration,
                steering_parameters.separation_threshold,
                steering_parameters.separation_decay_coefficient
            );

            if steering_output.linear.length() > f32::EPSILON {
                steering_output_to_apply.push((e, steering_output));
            }

        }

        for (e, steering_output) in steering_output_to_apply {
            let mut dynamic_body = self.world.get::<&mut DynamicBody>(e).expect("must have DynamicBody to get here!");
            entity_apply_raw_steering(&mut dynamic_body.kinematic, Some(steering_output), Self::TIME_STEP);
        }

    }

    //#[profiling::function]
    fn tick_attackers(&mut self) {

        let mut attack_targets = BTreeMap::new();

        // search for targets in range and accumulate

        for (e, (controller, attacker, transform, orderable, &state)) in self.world.query::<(&Controller, &mut Attacker, &Transform, &Orderable, &EntityState)>().iter() {

            if orderable.is_current_order_attack_order() {
                continue;
            }

            attacker.target = None; // reset current target every time we tick the attackers

            for o in self.spatial_manager.entities_within_radius(transform.world_position, attacker.range) {

                if self.world.contains(o) == false {
                    continue;
                }

                let mut other_query = self.world.query_one::<(&Controller, &Attackable, &Transform, &EntityState)>(o).unwrap();
                let Some((other_controller, other_attackable, other_transform, other_state)) = other_query.get() else { continue };

                let can_attack = self.is_controller_attackable_by(controller.id, other_controller);
                let is_current_order_queue_empty = orderable.is_queue_empty(GameOrderType::Order);
                let is_current_order_attack_move = orderable.is_current_order_attack_move_order();

                if e == o || state != EntityState::Constructed || (is_current_order_queue_empty == false && is_current_order_attack_move == false) || can_attack == false {
                    continue
                }

                let is_in_attack_range = transform.world_position.distance(other_transform.world_position) <= attacker.range;

                if is_in_attack_range {
                    let entry = attack_targets.entry(e).or_insert(vec![]);
                    entry.push(o);
                }

            }

        }

        // now filter targets and pick one unless we have a direct attack order

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
                point_entity_towards_target(&mut self.world, e, position.x, position.y, Self::TIME_STEP);
                if let Ok(mut attacker) = self.world.get::<&mut Attacker>(e) {
                    attacker.target = Some(closest_target);
                }
            }

        }

    }

    //#[profiling::function]
    fn tick_projectile_weapons(&mut self) {

        struct Bullet {
            owner: PlayerID,
            position: Vec2,
            direction: Vec2,
            parameters: BulletParameters
        }

        let mut queued_projectile_creations: Vec<Bullet> = Vec::new();

        for (e, (controller, transform, attacker, projectile_weapon)) in self.world.query::<(&Controller, &Transform, &Attacker, &mut ProjectileWeapon)>().iter() {

            if let Some(attacker) = attacker.target {

                if projectile_weapon.cooldown == 0.0 {

                    let attacker_position = get_entity_position(&self.world, attacker).unwrap();

                    let attack_direction_deviation = random_binomial() * projectile_weapon.deviation;
                    let attack_direction = (attacker_position - transform.world_position).normalize();
                    let attack_direction_with_deviation = attack_direction.rotated_by(attack_direction_deviation);

                    let id = controller.id;
                    let creation_world_position = transform.world_position + projectile_weapon.offset.rotated_by(transform.world_rotation);

                    queued_projectile_creations.push(Bullet { owner: controller.id, position: creation_world_position, direction: attack_direction_with_deviation, parameters: projectile_weapon.projectile });
                    projectile_weapon.cooldown += projectile_weapon.fire_rate;

                } else {

                    projectile_weapon.cooldown = (projectile_weapon.cooldown - Self::TIME_STEP).max(0.0);

                }

            }

        }

        for projectile in queued_projectile_creations {
            create_simple_bullet(&mut self.world, projectile.owner, projectile.position, projectile.direction);
        }

    }
    
    fn tick_beam_weapons(&mut self) {

        struct Beam {
            owner: PlayerID,
            position: Vec2,
            direction: Vec2,
            parameters: BeamParameters
        }

        let mut queued_beam_creations: Vec<Beam> = Vec::new();

        for (e, (controller, transform, attacker, beam_weapon)) in self.world.query::<(&Controller, &Transform, &Attacker, &mut BeamWeapon)>().iter() {

            if let Some(attacker) = attacker.target {

                if beam_weapon.cooldown == 0.0 {

                    let attacker_position = get_entity_position(&self.world, attacker).unwrap();

                    let attack_direction_deviation = random_binomial() * beam_weapon.deviation;
                    let attack_direction = (attacker_position - transform.world_position).normalize();
                    let attack_direction_with_deviation = attack_direction.rotated_by(attack_direction_deviation);

                    let id = controller.id;
                    let creation_world_position = transform.world_position + beam_weapon.offset.rotated_by(transform.world_rotation);

                    queued_beam_creations.push(Beam { owner: controller.id, position: creation_world_position, direction: attack_direction_with_deviation, parameters: beam_weapon.beam });
                    beam_weapon.cooldown += beam_weapon.fire_rate;

                } else {

                    beam_weapon.cooldown = (beam_weapon.cooldown - Self::TIME_STEP).max(0.0);

                }

            }

        }

        for projectile in queued_beam_creations {
            create_simple_beam(&mut self.world, projectile.owner, projectile.position, projectile.direction);
        }  

    }

    fn tick_beams(&mut self) {

        let mut hit_entities = Vec::new();

        for (e, (controller, beam)) in self.world.query::<(&Controller, &mut Beam)>().iter() {

            if beam.fired {
                continue
            }
            
            let beam_range = 1000.0;
            if let Some((entity, intersection)) = self.physics_manager.ray_cast(beam.position, beam.target, &self.world, &self.spatial_manager, 1 << controller.id) {
                hit_entities.push((entity, intersection, (intersection - beam.position).normalize(), beam.damage));
                beam.target = intersection;
            }

            beam.fired = true;

        }

        for (e, position, hit_normal, damage) in hit_entities {

            {
                let Ok(health) = self.world.query_one_mut::<&mut Health>(e) else { continue; };
                health.damage(damage);

                if let Ok(body) = self.world.query_one_mut::<&mut DynamicBody>(e) {
                    body.apply_impulse(hit_normal * damage / 2.0, Vec2::ZERO);
                }
            }

            create_impact_effect_in_world(&mut self.world, position, hit_normal);

        }

    }

    //#[profiling::function]
    fn tick_projectiles(&mut self) {

        for (e, (body, projectile, health)) in self.world.query_mut::<(&mut DynamicBody, &mut Projectile, &mut Health)>() {

            *body.velocity_mut() = -body.kinematic.orientation.as_vector() * projectile.velocity;
            projectile.lifetime -= Self::TIME_STEP;

            if projectile.lifetime <= 0.0 {
                health.kill();
            }

        }

    }

    //#[profiling::function]
    fn tick_constructors(&mut self) {

        for (e, (controller, constructor)) in self.world.query::<(&Controller, &mut Constructor)>().iter() {

            // we must have a current entity we're constructing for any of this logic to make sense
            let Some(constructing_entity) = constructor.current_target else { continue; };

            let mut entity_health = self.world.get::<&mut Health>(constructing_entity).expect("entity must have the Health component");
            let entity_blueprint_identity = self.world.get::<&BlueprintIdentity>(constructing_entity).expect("entity must have blueprint identity to be able to identify cost!");
            let entity_blueprint = self.blueprint_manager.get_blueprint(entity_blueprint_identity.blueprint_id).expect("entity must have blueprint!");

            let entity_metal_cost = entity_blueprint.cost.metal;
            let entity_energy_cost = entity_blueprint.cost.energy;

            // #HACK: this is a little hackety sax
            let entity_remaining_health_fraction = 1.0 - entity_health.current_health_fraction();
            
            let entity_remaining_metal_cost = entity_metal_cost * entity_remaining_health_fraction;
            let entity_remaining_energy_cost = entity_energy_cost * entity_remaining_health_fraction;

            let available_metal = current_metal(controller.id, &self.world);
            let available_energy = current_energy(controller.id, &self.world);

            // build power/metal cost should actually be the max of the build speed and remaining cost, we clamp this later (this way build speed actually matters)
            let build_power_metal_cost = (constructor.build_speed as f32).max(entity_remaining_metal_cost);
            let build_power_energy_cost = (constructor.build_speed as f32).max(entity_remaining_energy_cost);
            
            let available_metal_proportion = available_metal / entity_remaining_metal_cost;
            let available_energy_proportion = available_energy / entity_remaining_energy_cost;
            let min_available_proportion = available_metal_proportion.min(available_energy_proportion);

            let build_power_metal_proportion = build_power_metal_cost / entity_remaining_metal_cost;
            let build_power_energy_proportion = build_power_energy_cost / entity_remaining_energy_cost;
            let min_available_proportion_with_build_power = build_power_metal_proportion.min(build_power_energy_proportion);

            // here we make sure that if our build speed exceeds our actually available resources, we take the minimum so we can actually use as much as is actually available
            let clamped_min_available_proportion_with_build_power = min_available_proportion_with_build_power.min(min_available_proportion);

            let metal_to_consume = entity_remaining_metal_cost * clamped_min_available_proportion_with_build_power;
            let energy_to_consume = entity_remaining_energy_cost * clamped_min_available_proportion_with_build_power;

            let metal_to_consume_this_tick = metal_to_consume * Self::TIME_STEP;
            let energy_to_consume_this_tick = energy_to_consume * Self::TIME_STEP;

            consume_metal(controller.id, &self.world, metal_to_consume_this_tick, Self::TIME_STEP);
            consume_energy(controller.id, &self.world, energy_to_consume_this_tick, Self::TIME_STEP);

            let entity_health_proportion = entity_remaining_health_fraction * entity_health.full_health();
            let entity_health_regain_amount = entity_health_proportion * clamped_min_available_proportion_with_build_power;
            let entity_repair_health = entity_health_regain_amount * Self::TIME_STEP;
            entity_health.heal(entity_repair_health);
            
        }

    }

    fn entities_within_radius_sorted_by_distance(&self, world_position: Vec2, radius: f32) -> Vec<Entity> {
        self.spatial_manager.entities_within_radius_sorted_by(
            world_position,
            radius,
            |a, b| entity_distance_sort_function(&self.world, world_position, a, b)
        )
    }

    fn tick_extractors(&mut self) {

        for (e, (controller, transform, orderable, extractor, movement_target, rotation_target)) in self.world.query::<(&Controller, &Transform, &mut Orderable, &mut Extractor, &mut MovementTarget, &mut RotationTarget)>().iter() {

            if let Some(last_extractor_target) = extractor.last_target {
                movement_target.target = None;
                rotation_target.target = None;
                extractor.last_target = None;
                extractor.is_active = false;
                extractor.is_searching = false;
            }

            if extractor.is_searching && extractor.current_target.is_none() {

                for o in self.entities_within_radius_sorted_by_distance(transform.world_position, extractor.extraction_range as f32) {

                    if e == o {
                        continue;
                    }

                    if self.world.contains(o) == false {
                        continue;
                    }

                    let Ok(source) = self.world.get::<&ResourceSource>(o) else { continue; };

                    if source.is_occupied() == false {
                        orderable.push_order(GameOrder::Extract(ExtractOrder { entity_id: o.to_bits().into() }));
                        extractor.is_searching = false;
                        continue;
                    }

                }

            }

            let Some(current_extractor_target) = extractor.current_target else { continue; };
            let Some(current_extractor_target_position) = get_entity_position(&self.world, current_extractor_target) else { continue; };

            if is_within_extractor_range_with_extractor(e, &self.world, extractor, current_extractor_target_position) {

                let mut source = self.world.get::<&mut ResourceSource>(current_extractor_target).expect("extractor target must have resource source!");
                
                let actual_source_metal_provided = source.current_metal.min(extractor.extraction_speed as f32);
                let actual_source_energy_provided = source.current_energy.min(extractor.extraction_speed as f32);

                let source_metal_provided = actual_source_metal_provided * Self::TIME_STEP;
                let source_energy_provided = actual_source_energy_provided * Self::TIME_STEP;

                provide_metal(controller.id, &self.world, source_metal_provided, Self::TIME_STEP);
                provide_energy(controller.id, &self.world, source_energy_provided, Self::TIME_STEP);

                if source.is_finite == false {
                    source.current_metal = (source.current_metal - actual_source_metal_provided).max(0.0);
                    source.current_energy = (source.current_energy - actual_source_energy_provided).max(0.0);
                } else {
                    source.current_metal = (source.current_metal - source_metal_provided).max(0.0);
                    source.current_energy = (source.current_energy - source_energy_provided).max(0.0);
                }

                movement_target.target = None;
                rotation_target.target = Some(current_extractor_target_position);
                extractor.is_active = true;

            } else {

                extractor.is_active = false;
                movement_target.target = Some(current_extractor_target_position);
                rotation_target.target = None;
                
            }

            extractor.last_target = extractor.current_target;

        }

    }

    //#[profiling::function]
    fn tick_resource_storage(&mut self) {

        let mut energy_pools_per_player = BTreeMap::new();

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
    
    //#[profiling::function]
    fn tick_resources(&mut self) {

        let mut energy_incomes_per_player = BTreeMap::new();

        for (e, (controller, &state, consumer, producer)) in self.world.query::<(&Controller, &EntityState, Option<&Consumer>, Option<&Producer>)>().iter() {

            let mut total_metal_income = 0.0;
            let mut total_energy_income = 0.0;

            if let Some(consumer) = consumer && state == EntityState::Constructed {
                consume_metal(controller.id, &self.world, consumer.metal * Self::TIME_STEP, 0.0);
                consume_energy(controller.id, &self.world, consumer.energy * Self::TIME_STEP, 0.0);
                total_metal_income -= consumer.metal;
                total_energy_income -= consumer.energy;
            }

            if let Some(producer) = producer && state == EntityState::Constructed {
                provide_metal(controller.id, &self.world, producer.metal * Self::TIME_STEP, 0.0);
                provide_energy(controller.id, &self.world, producer.energy * Self::TIME_STEP, 0.0);
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

    fn tick_resource_sources(&mut self) {

        for (e, (resource_source, health)) in self.world.query::<(&mut ResourceSource, &mut Health)>().iter() {

            if resource_source.is_finite {
                if resource_source.is_exhausted() {
                    health.kill();
                }
                continue;
            }

            resource_source.current_metal = resource_source.total_metal;
            resource_source.current_energy = resource_source.total_energy;

        }

    }

    //#[profiling::function]
    fn tick_transform_updates(&mut self) {

        for (e, (transform, body, previous_transform)) in self.world.query_mut::<(&mut Transform, &mut DynamicBody, Option<&mut PreviousTransform>)>() {

            transform.local_position = body.kinematic.position;
            transform.local_rotation = body.kinematic.orientation;

            if let Some(previous_transform) = previous_transform {
                previous_transform.transform = *transform;
            }

        }

    }

    //#[profiling::function]
    fn tick_spatial_engine(&mut self) {
        
        let mut created_entity_positions = Vec::new();
        let mut updated_entity_positions = Vec::new();
        let mut deleted_entity_positions = Vec::new();
        
        for (e, (transform, _dynamic_body, health, effect, previous_transform)) in self.world.query_mut::<(&mut Transform, &DynamicBody, Option<&Health>, Option<&Effect>, Option<&mut PreviousTransform>)>() {

            // handle entities being created/updated
            if let Some(previous_transform) = previous_transform {        
                if previous_transform.transform.world_position != transform.world_position {
                    updated_entity_positions.push((e, previous_transform.transform.world_position, transform.world_position));
                }
            } else {
                created_entity_positions.push((e, *transform));
            }
            
            // handle entities being destroyed
            if let Some(health) = health && health.is_at_or_below_zero_health() {
                deleted_entity_positions.push((e, *transform));
            } else if let Some(effect) = effect && effect.lifetime <= 0.0 {
                deleted_entity_positions.push((e, *transform));
            }

        }

        for (e, transform) in created_entity_positions {
            let _ = self.world.insert(e, (PreviousTransform { transform }, ));
            self.spatial_manager.add_entity(e, transform.world_position);
        }

        for (e, previous_world_position, new_world_position) in updated_entity_positions {
            self.spatial_manager.update_entity_position(e, previous_world_position, new_world_position);
        }

        for (e, transform) in deleted_entity_positions {
            self.spatial_manager.remove_entity(e, transform.world_position);
        }

    }

    //#[profiling::function]
    fn tick_physics_engine(&mut self) {
        self.physics_manager.integrate(&mut self.world);
        self.physics_manager.handle_overlaps(&mut self.world, &self.spatial_manager);
        self.physics_manager.handle_collisions(&mut self.world);
    }

    //#[profiling::function]
    fn tick_effects(&mut self) {

        for (e, (sprite, effect)) in self.world.query_mut::<(Option<&mut AnimatedSprite>, &mut Effect)>() {

            if let Some(sprite) = sprite {
                sprite.current_frame = (sprite.h_frames as f32 * (1.0 - (effect.lifetime / effect.total_lifetime))) as i32;
            }

            effect.lifetime -= Self::TIME_STEP;

        }

    }

    //#[profiling::function]
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

            if let Ok(health) = self.world.get::<&Health>(e) && let Some(on_death_fn) = health.on_death {
                (on_death_fn)(&self.world, &mut command_buffer, e);
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

    pub fn is_controller_attackable_by(&self, controller_id: PlayerID, target_controller: &Controller) -> bool {
        are_players_hostile(self.get_player_by_id(controller_id), self.get_player_by_id(target_controller.id))
    }

    pub fn is_controller_friendly_to(&self, controller_id: PlayerID, target_controller: &Controller) -> bool {
        are_players_allied(self.get_player_by_id(controller_id), self.get_player_by_id(target_controller.id))
    }

    pub fn is_controller_controllable_by(&self, controller_id: PlayerID, target_controller: &Controller) -> bool {
        controller_id == target_controller.id // #TODO: alliances, teams?
    }

    pub fn is_entity_attackable_by(&self, attacking_controller_id: PlayerID, entity: Entity) -> bool {
        let controller = self.world.get::<&Controller>(entity).expect("must have controller!");
        self.is_controller_attackable_by(attacking_controller_id, &controller) && self.world.get::<&Attackable>(entity).is_ok()
    }

    pub fn tick(&mut self) {

        macroquad::rand::srand(42);
        
        self.tick_constructing_entities();
        self.tick_powered_entities();
        self.tick_resource_storage();
        self.tick_orderables();
        self.tick_transforms();
        self.tick_resources();
        self.tick_resource_sources();
        self.tick_rotation_targets();
        self.tick_movement_targets();
        self.tick_separation();
        self.tick_attackers();
        self.tick_projectile_weapons();
        self.tick_beam_weapons();
        self.tick_projectiles();
        self.tick_beams();
        self.tick_effects();
        self.tick_constructors();
        self.tick_extractors();
        self.tick_physics_engine();
        self.tick_spatial_engine();
        self.tick_transform_updates();
        self.tick_lifetimes();

        self.current_tick += 1;

    }

}