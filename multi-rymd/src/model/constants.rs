use utility::SteeringParameters;

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