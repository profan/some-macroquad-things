use std::f32::consts::PI;

use hecs::*;
use macroquad::prelude::*;
use utility::{Kinematic, AsAngle, SteeringParameters};

use crate::PlayerID;
use crate::model::{Transform, Orderable, AnimatedSprite, Thruster, DynamicBody, Ship, ThrusterKind};
use super::{Constructor, Controller, Health, DEFAULT_STEERING_PARAMETERS, Steering};

#[derive(Bundle)]
pub struct AnimatedShipBody {
    health: Health,
    transform: Transform,
    dynamic_body: DynamicBody,
    orderable: Orderable,
    sprite: AnimatedSprite,
    steering: Steering,
    ship: Ship
}

pub struct ShipParameters {
    turn_rate: f32
}

impl AnimatedShipBody {

    pub fn new(health: i32, position: Vec2, kinematic: Kinematic, parameters: ShipParameters, steering: SteeringParameters, texture: &str, v_frames: i32) -> AnimatedShipBody {

        let standard_size = 32.0;

        // #FIXME: we need to compute the bounds somehow without requiring graphics... maybe that is a fools errand? we'll figure it out i guess, json file maybe?
        let bounds = Rect { x: -(standard_size / 2.0), y: -(standard_size / 2.0), w: standard_size, h: standard_size };
        let is_static = false;

        AnimatedShipBody {
            health: Health::new(health),
            transform: Transform::new(position, 0.0, None),
            dynamic_body: DynamicBody { is_static, bounds, kinematic },
            orderable: Orderable::new(),
            sprite: AnimatedSprite { texture: texture.to_string(), current_frame: 0, h_frames: v_frames },
            steering: Steering { parameters: steering },
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

pub fn create_commander_ship(world: &mut World, owner: PlayerID, position: Vec2) -> Entity {

    let commander_health = 1000;

    let commander_build_speed = 100;
    let commander_build_range = 100;

    let commander_thruster_power = 64.0;
    let commander_turn_thruster_power = 16.0;

    let commander_steering_parameters = DEFAULT_STEERING_PARAMETERS;
    
    let commander_kinematic_body = Kinematic {
        position: position,
        orientation: 0.0,
        velocity: Vec2::ZERO,
        angular_velocity: 0.0,
        friction_value: 0.975f32,
        mass: 1.0
    };

    let commander_ship_parameters = ShipParameters {
        turn_rate: 4.0
    };

    let commander_ship_controller = Controller { id: owner };
    let commander_ship_constructor = Constructor { constructibles: vec![0], build_speed: commander_build_range, build_range: commander_build_speed };
    let commander_ship_body = world.spawn(AnimatedShipBody::new(commander_health, position, commander_kinematic_body, commander_ship_parameters, commander_steering_parameters, "PLAYER_SHIP", 3));

    let _ = world.insert_one(commander_ship_body, commander_ship_controller);
    let _ = world.insert_one(commander_ship_body, commander_ship_constructor);

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