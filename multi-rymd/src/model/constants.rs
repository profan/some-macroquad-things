use macroquad::{math::Rect, prelude::Vec2};
use utility::{SteeringParameters, Kinematic};

use super::{BeamParameters, BulletParameters};

pub enum Blueprints {

    Shipyard = 1,
    SolarCollector = 2,
    EnergyStorage = 3,
    MetalStorage = 4,

    // blue units
    Commander = 5,
    Arrowhead = 6,
    Extractor = 7,

    // green units
    Commissar = 8,
    Grunt = 9

}

pub const DEFAULT_STEERING_PARAMETERS: SteeringParameters = SteeringParameters {

    acceleration: 256.0,

    max_speed: 384.0,
    max_acceleration: 128.0,
    arrive_radius: 64.0,
    slow_radius: 200.0,

    align_max_rotation: 2.0,
    align_max_angular_acceleration: 2.0,
    align_radius: 0.0125 / 4.0,
    align_slow_radius: 0.05 / 4.0,

    separation_threshold: 512.0,
    separation_decay_coefficient: 2048.0

};

pub const COMMANDER_STEERING_PARAMETERS: SteeringParameters = SteeringParameters {
    align_max_angular_acceleration: 8.0,
    ..DEFAULT_STEERING_PARAMETERS
};

pub const ARROWHEAD_STEERING_PARAMETERS: SteeringParameters = SteeringParameters {
    align_max_angular_acceleration: 8.0,
    ..DEFAULT_STEERING_PARAMETERS
};

pub const EXTRACTOR_STEERING_PARAMETERS: SteeringParameters = SteeringParameters {
    max_speed: 256.0,
    align_max_angular_acceleration: 8.0,
    ..DEFAULT_STEERING_PARAMETERS
};

pub const SIMPLE_BULLET_PARAMETERS: BulletParameters = BulletParameters {
    health: 10.0,
    lifetime: 4.0,
    velocity: 256.0,
    damage: 25.0,

    bounds: Rect { x: 0.0, y: 0.0, w: 2.0, h: 2.0 },
    texture: "SIMPLE_BULLET"
};

pub const SIMPLE_BEAM_PARAMETERS: BeamParameters = BeamParameters {
    damage: 25.0,
    lifetime: 1.0 / 60.0,
    range: 256.0,
    color: 0xfed452
};

pub fn create_default_kinematic_body(position: Vec2, orientation: f32) -> Kinematic {
    Kinematic {
        position,
        orientation,
        velocity: Vec2::ZERO,
        angular_velocity: 0.0,
        friction_value: 0.975f32,
        mass: 1.0
    }
}