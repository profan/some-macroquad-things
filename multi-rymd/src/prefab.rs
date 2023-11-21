use std::{collections::VecDeque, f32::consts::PI};

use hecs::*;
use macroquad::prelude::*;
use utility::{Kinematic, AsAngle};

use crate::{Transform, Orderable, AnimatedSprite, Thruster, DynamicBody, Ship, ThrusterKind};

#[derive(Bundle)]
pub struct ShipBody {
    transform: Transform,
    dynamic_body: DynamicBody,
    orderable: Orderable,
    sprite: AnimatedSprite,
    ship: Ship
}

pub struct ShipParameters {
    turn_rate: f32
}

impl ShipBody {

    pub fn new(position: Vec2, kinematic: Kinematic, parameters: ShipParameters, texture: &str, v_frames: i32) -> ShipBody {
        ShipBody {
            transform: Transform::new(position, 0.0, None),
            dynamic_body: DynamicBody { kinematic },
            orderable: Orderable::new(),
            sprite: AnimatedSprite { texture: texture.to_string(), current_frame: 0, v_frames },
            ship: Ship::new(parameters.turn_rate)
        }
    }

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

pub fn create_player_ship(world: &mut World, position: Vec2) -> Entity {

    let player_thruster_power = 64.0;
    let player_turn_thruster_power = 16.0;
    
    let player_kinematic_body = Kinematic {
        position: position,
        orientation: 0.0,
        velocity: Vec2::ZERO,
        angular_velocity: 0.0,
        friction_value: 0.975f32,
        mass: 1.0
    };

    let player_ship_parameters = ShipParameters {
        turn_rate: 4.0
    };

    let player_ship_body = world.spawn(ShipBody::new(position, player_kinematic_body, player_ship_parameters, "PLAYER_SHIP", 3));

    let player_ship_thruster_left_top = world.spawn(ShipThruster::new(vec2(-14.0, 4.0), -Vec2::X, -(PI / 2.0), player_turn_thruster_power, ThrusterKind::Attitude, player_ship_body));
    let player_ship_thuster_right_top = world.spawn(ShipThruster::new(vec2(14.0, 4.0), Vec2::X, (PI / 2.0), player_turn_thruster_power, ThrusterKind::Attitude, player_ship_body));

    let player_ship_thruster_left_bottom = world.spawn(ShipThruster::new(vec2(-14.0, 4.0), Vec2::X, -(PI / 2.0), player_turn_thruster_power, ThrusterKind::Attitude, player_ship_body));
    let player_ship_thuster_right_bottom = world.spawn(ShipThruster::new(vec2(14.0, 4.0), -Vec2::X, (PI / 2.0), player_turn_thruster_power, ThrusterKind::Attitude, player_ship_body));

    let player_ship_thruster_main = world.spawn(ShipThruster::new(vec2(0.0, 10.0), Vec2::Y, 0.0, player_thruster_power, ThrusterKind::Main, player_ship_body));

    let mut player_ship = world.get::<&mut Ship>(player_ship_body).unwrap();
    player_ship.thrusters.push(player_ship_thruster_left_top);
    player_ship.thrusters.push(player_ship_thuster_right_top);
    player_ship.thrusters.push(player_ship_thruster_left_bottom);
    player_ship.thrusters.push(player_ship_thuster_right_bottom);
    player_ship.thrusters.push(player_ship_thruster_main);

    player_ship_body

}