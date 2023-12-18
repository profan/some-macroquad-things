use std::f32::consts::PI;

use hecs::*;
use macroquad::prelude::*;
use utility::AsAngle;

use crate::PlayerID;
use crate::model::{Transform, Orderable, AnimatedSprite, Thruster, DynamicBody, Ship, ThrusterKind};
use super::{Constructor, Controller, Health, DEFAULT_STEERING_PARAMETERS, Steering, create_default_kinematic_body, Blueprint, EntityState, BlueprintIdentity};

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
        id: 2,
        shortcut: KeyCode::Key3,
        name: String::from("Commander Ship"),
        texture: String::from("PLAYER_SHIP"),
        constructor: build_commander_ship,
        is_building: false
    }
}

pub fn build_commander_ship(world: &mut World, owner: PlayerID, position: Vec2) -> Entity {

    let commander_ship_size = 32.0;
    let bounds = Rect { x: 0.0, y: 0.0, w: commander_ship_size, h: commander_ship_size };
    let is_enabled = false;
    let is_static = false;

    let initial_commander_health = 100;
    let full_commander_health = 1000;

    let commander_build_speed = 250;
    let commander_build_range = 100;
    let commander_build_offset = -vec2(bounds.w / 8.0, 0.0);

    let commander_thruster_power = 64.0;
    let commander_turn_thruster_power = 16.0;

    let steering_parameters = DEFAULT_STEERING_PARAMETERS;
    let kinematic = create_default_kinematic_body(position, 0.0);
    let ship_parameters = ShipParameters {
        turn_rate: 4.0
    };

    // assemble the ship
    let controller = Controller { id: owner };
    let blueprint_identity = BlueprintIdentity { blueprint_id: 2 };
    let constructor = Constructor { is_constructing: false, constructibles: vec![0, 1], build_speed: commander_build_speed, build_range: commander_build_range, beam_offset: commander_build_offset };
    let health = Health::new_with_current_health(full_commander_health, initial_commander_health);
    let transform = Transform::new(position, 0.0, None);
    let dynamic_body = DynamicBody { is_enabled, is_static, bounds, kinematic };
    let sprite = AnimatedSprite { texture: "PLAYER_SHIP".to_string(), current_frame: 0, h_frames: 3 };
    let steering = Steering { parameters: steering_parameters };
    let ship = Ship::new(ship_parameters.turn_rate);
    let orderable = Orderable::new();
    let state = EntityState::Ghost;

    let commander_ship_body = world.spawn((health, transform, dynamic_body, sprite, steering, ship, orderable, controller, constructor, blueprint_identity, state));

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