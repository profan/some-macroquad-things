use std::f32::consts::PI;

use hecs::*;
use macroquad::prelude::*;
use utility::AsAngle;

use crate::PlayerID;
use crate::model::{Transform, Orderable, AnimatedSprite, Thruster, DynamicBody, Ship, ThrusterKind};
use super::{create_default_kinematic_body, create_explosion_effect_in_buffer, get_entity_position, Attackable, Attacker, Blueprint, BlueprintIdentity, Blueprints, Constructor, Controller, Cost, EntityState, Health, HealthCallback, Producer, Steering, Weapon, DEFAULT_STEERING_PARAMETERS};

pub struct ShipParameters {
    turn_rate: f32
}

#[derive(Bundle)]
pub struct ShipThruster {
    transform: Transform,
    thruster: Thruster
}

impl ShipThruster {
    pub fn new(position: Vec2, direction: Vec2, angle: f32, power: f32, kind: ThrusterKind, parent: Entity) -> ShipThruster {
        ShipThruster {
            transform: Transform::new(position, direction.as_angle(), Some(parent)),
            thruster: Thruster { kind, direction, angle, power },
        }
    }
}

pub fn create_commander_ship_blueprint() -> Blueprint {
    Blueprint {
        id: Blueprints::Commander as i32,
        shortcut: KeyCode::Key3,
        name: String::from("Commander Ship"),
        texture: String::from("PLAYER_SHIP"),
        constructor: build_commander_ship,
        cost: Cost { metal: 25.0, energy: 25.0 },
        is_building: false
    }
}

pub fn create_arrowhead_ship_blueprint() -> Blueprint {
    Blueprint {
        id: Blueprints::Arrowhead as i32,
        shortcut: KeyCode::Key6,
        name: String::from("Arrowhead (Fighter)"),
        texture: String::from("ARROWHEAD"),
        constructor: build_arrowhead_ship,
        cost: Cost { metal: 25.0, energy: 25.0 },
        is_building: false
    }
}

fn on_ship_death(world: &World, buffer: &mut CommandBuffer, entity: Entity) {

    if let Ok(ship) = world.get::<&Ship>(entity) {
        for &t in &ship.thrusters {
            if let Ok(mut h) = world.get::<&mut Health>(t) {
                h.kill();
            }
        }
    }
    
    let ship_position = get_entity_position(world, entity).unwrap();
    create_explosion_effect_in_buffer(buffer, ship_position);
    
}

pub fn build_commander_ship(world: &mut World, owner: PlayerID, position: Vec2) -> Entity {

    let commander_ship_size = 32.0;
    let bounds = Rect { x: 0.0, y: 0.0, w: commander_ship_size, h: commander_ship_size };
    let is_enabled = false;
    let is_static = false;
    let mask = 1 << owner;

    let initial_commander_health = 100;
    let full_commander_health = 1000;

    let commander_build_speed = 100;
    let commander_build_range = 100;
    let commander_build_offset = -vec2(bounds.w / 8.0, 0.0);
    let commander_blueprints = vec![Blueprints::Shipyard as i32, Blueprints::SolarCollector as i32, Blueprints::EnergyStorage as i32, Blueprints::MetalStorage as i32];

    let commander_thruster_power = 64.0;
    let commander_turn_thruster_power = 16.0;

    let commander_metal_income = 10.0;
    let commander_energy_income = 10.0;

    let steering_parameters = DEFAULT_STEERING_PARAMETERS;
    let kinematic = create_default_kinematic_body(position, 0.0);
    let ship_parameters = ShipParameters {
        turn_rate: 4.0
    };

    // assemble the ship
    let controller = Controller { id: owner };
    let blueprint_identity = BlueprintIdentity::new(Blueprints::Commander);
    let constructor = Constructor { current_target: None, constructibles: commander_blueprints, build_speed: commander_build_speed, build_range: commander_build_range, beam_offset: commander_build_offset, can_assist: true };
    let health = Health::new_with_current_health(full_commander_health, initial_commander_health);
    let health_callback = HealthCallback { on_death: on_ship_death };
    let transform = Transform::new(position, 0.0, None);
    let dynamic_body = DynamicBody { is_enabled, is_static, bounds, kinematic, mask };
    let sprite = AnimatedSprite { texture: "PLAYER_SHIP".to_string(), current_frame: 0, h_frames: 3 };
    let producer = Producer { metal: commander_metal_income, energy: commander_energy_income };
    let steering = Steering { parameters: steering_parameters };
    let ship = Ship::new(ship_parameters.turn_rate);
    let orderable = Orderable::new();
    let state = EntityState::Ghost;
    let attackable = Attackable;

    let commander_ship_body = world.spawn((health, health_callback, transform, dynamic_body, sprite, producer, steering, ship, orderable, controller, constructor, blueprint_identity, state, attackable));

    // add ship thrusters
    let commander_ship_thruster_left_top = world.spawn(ShipThruster::new(vec2(-14.0, 4.0), -Vec2::X, -(PI / 2.0), commander_turn_thruster_power, ThrusterKind::Attitude, commander_ship_body));
    let commander_ship_thuster_right_top = world.spawn(ShipThruster::new(vec2(14.0, 4.0), Vec2::X, PI / 2.0, commander_turn_thruster_power, ThrusterKind::Attitude, commander_ship_body));

    let commander_ship_thruster_left_bottom = world.spawn(ShipThruster::new(vec2(-14.0, 4.0), Vec2::X, -(PI / 2.0), commander_turn_thruster_power, ThrusterKind::Attitude, commander_ship_body));
    let commander_ship_thuster_right_bottom = world.spawn(ShipThruster::new(vec2(14.0, 4.0), -Vec2::X, PI / 2.0, commander_turn_thruster_power, ThrusterKind::Attitude, commander_ship_body));

    let commander_ship_thruster_main = world.spawn(ShipThruster::new(vec2(0.0, 10.0), Vec2::Y, 0.0, commander_thruster_power, ThrusterKind::Main, commander_ship_body));

    let mut commander_ship = world.get::<&mut Ship>(commander_ship_body).unwrap();
    commander_ship.thrusters.push(commander_ship_thruster_left_top);
    commander_ship.thrusters.push(commander_ship_thuster_right_top);
    commander_ship.thrusters.push(commander_ship_thruster_left_bottom);
    commander_ship.thrusters.push(commander_ship_thuster_right_bottom);
    commander_ship.thrusters.push(commander_ship_thruster_main);

    commander_ship_body

}

pub fn build_arrowhead_ship(world: &mut World, owner: PlayerID, position: Vec2) -> Entity {

    let arrowhead_ship_size = 32.0;
    let bounds = Rect { x: 0.0, y: 0.0, w: arrowhead_ship_size, h: arrowhead_ship_size };
    let is_enabled = false;
    let is_static = false;
    let mask = 1 << owner;

    let initial_arrowhead_health = 100;
    let full_arrowhead_health = 250;

    let arrowhead_thruster_power = 64.0;
    let arrowhead_turn_thruster_power = 16.0;
    let arrowhead_fire_rate = 0.25;
    let arrowhead_range = 256.0;

    let steering_parameters = DEFAULT_STEERING_PARAMETERS;
    let kinematic = create_default_kinematic_body(position, 0.0);
    let ship_parameters = ShipParameters {
        turn_rate: 4.0
    };

    // assemble the ship
    let controller = Controller { id: owner };
    let blueprint_identity = BlueprintIdentity::new(Blueprints::Arrowhead);
    let health = Health::new_with_current_health(full_arrowhead_health, initial_arrowhead_health);
    let health_callback = HealthCallback { on_death: on_ship_death };
    let transform = Transform::new(position, 0.0, None);
    let dynamic_body = DynamicBody { is_enabled, is_static, bounds, kinematic, mask };
    let sprite = AnimatedSprite { texture: "ARROWHEAD".to_string(), current_frame: 0, h_frames: 1 };
    let steering = Steering { parameters: steering_parameters };
    let ship = Ship::new(ship_parameters.turn_rate);
    let orderable = Orderable::new();
    let state = EntityState::Ghost;

    let weapon = Weapon { offset: vec2(0.0, -(arrowhead_ship_size / 2.0)), fire_rate: arrowhead_fire_rate, cooldown: 0.0 };
    let attackable = Attackable;
    let attacker = Attacker {
        range: arrowhead_range,
        target: None
    };

    let arrowhead_ship_body = world.spawn((health, health_callback, transform, dynamic_body, sprite, steering, ship, orderable, controller, blueprint_identity, state, weapon, attackable, attacker));

    // add ship thrusters
    let arrowhead_ship_thruster_left_top = world.spawn(ShipThruster::new(vec2(-14.0, 4.0), -Vec2::X, -(PI / 2.0), arrowhead_turn_thruster_power, ThrusterKind::Attitude, arrowhead_ship_body));
    let arrowhead_ship_thuster_right_top = world.spawn(ShipThruster::new(vec2(14.0, 4.0), Vec2::X, PI / 2.0, arrowhead_turn_thruster_power, ThrusterKind::Attitude, arrowhead_ship_body));

    let arrowhead_ship_thruster_left_bottom = world.spawn(ShipThruster::new(vec2(-14.0, 4.0), Vec2::X, -(PI / 2.0), arrowhead_turn_thruster_power, ThrusterKind::Attitude, arrowhead_ship_body));
    let arrowhead_ship_thuster_right_bottom = world.spawn(ShipThruster::new(vec2(14.0, 4.0), -Vec2::X, PI / 2.0, arrowhead_turn_thruster_power, ThrusterKind::Attitude, arrowhead_ship_body));

    let arrowhead_ship_thruster_main = world.spawn(ShipThruster::new(vec2(0.0, 10.0), Vec2::Y, 0.0, arrowhead_thruster_power, ThrusterKind::Main, arrowhead_ship_body));

    let mut arrowhead_ship = world.get::<&mut Ship>(arrowhead_ship_body).unwrap();
    arrowhead_ship.thrusters.push(arrowhead_ship_thruster_left_top);
    arrowhead_ship.thrusters.push(arrowhead_ship_thuster_right_top);
    arrowhead_ship.thrusters.push(arrowhead_ship_thruster_left_bottom);
    arrowhead_ship.thrusters.push(arrowhead_ship_thuster_right_bottom);
    arrowhead_ship.thrusters.push(arrowhead_ship_thruster_main);

    arrowhead_ship_body

}