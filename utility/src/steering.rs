use macroquad::prelude::*;
use std::f32::consts::*;
use std::ops::Add;

use crate::extensions::*;

#[derive(Debug, Clone, Default)]
pub struct Kinematic {
    pub position: Vec2,
    pub orientation: f32,
    pub velocity: Vec2,
    pub angular_velocity: f32,
    pub friction_value: f32,
    pub mass: f32
}

impl Kinematic {

    pub fn integrate(&mut self, dt: f32) {

        if self.velocity.is_nan() {
            self.velocity = Vec2::ZERO;
        }

        if self.angular_velocity.is_nan() {
            self.angular_velocity = 0.0;
        }

        self.position += self.velocity * dt;
        self.orientation += self.angular_velocity * dt;

    }
    
    pub fn apply_friction(&mut self, dt: f32) {

        let original_fixed_rate = 1.0 / 60.0;
        let friction_rate = (self.friction_value).log2() / original_fixed_rate;

        self.velocity *= (friction_rate * dt).exp2();
        self.angular_velocity *= (friction_rate * dt).exp2();
        if self.orientation.is_nan() {
            self.orientation = 0.0;
        }

    }

    /// Returns the given Kinematic's predicted position a given number of seconds into the future given it's current position and velocity.
    pub fn predicted_position(&self, dt: f32) -> Vec2 {
        let mut mock_entity = self.clone();
        mock_entity.integrate(dt);
        mock_entity.apply_friction(dt);
        mock_entity.position
    }

}

pub trait SteeringOutputFilteredExt {
    fn only_linear(&self) -> SteeringOutput;
    fn only_angular(&self) -> SteeringOutput;
}

impl SteeringOutputFilteredExt for SteeringOutput {
    fn only_linear(&self) -> SteeringOutput {
        SteeringOutput { linear: self.linear, angular: 0.0 }
    }

    fn only_angular(&self) -> SteeringOutput {
        SteeringOutput { linear: Vec2::ZERO, angular: self.angular }
    }
}

#[derive(Copy, Clone, Default)]
pub struct SteeringOutput {
    pub linear: Vec2,
    pub angular: f32
}

impl Add for SteeringOutput {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            linear: self.linear + other.linear,
            angular: self.angular + other.angular
        }
    }
}

#[derive(Clone, Copy)]
pub struct SteeringParameters {

    pub acceleration: f32,

    pub max_speed: f32,
    pub max_acceleration: f32,
    pub arrive_radius: f32,
    pub slow_radius: f32,

    pub align_max_rotation: f32,
    pub align_max_angular_acceleration: f32,
    pub align_radius: f32,
    pub align_slow_radius: f32,

    pub separation_threshold: f32,
    pub separation_decay_coefficient: f32,

}

type SteeringBehaviour = dyn FnMut(&Kinematic, &Kinematic) -> SteeringOutput;

pub fn seek(character: &Kinematic, target: &Kinematic) -> SteeringOutput {

    let vector_to_target = target.position - character.position;
    let result_velocity = vector_to_target.normalize();

    SteeringOutput {
        linear: result_velocity,
        angular: 0.0
    }
    
}

pub fn pursue(character: &Kinematic, target: &Kinematic) -> SteeringOutput {

    let target_lead = target.velocity;
    let adjusted_target = Kinematic {
        position: target.position + target_lead,
        ..*target
    };

    seek(character, &adjusted_target)

}

/// The flee behaviour is the inverse of [`seek()`].
pub fn flee(character: &Kinematic, target: &Kinematic) -> SteeringOutput {
    seek(target, character) // flee is the inverse of seek, so it's as simple as this
}

pub fn arrive_with_lead_ex(character: &Kinematic, target: &Kinematic, parameters: SteeringParameters, time_to_target: f32) -> Option<SteeringOutput> {
    arrive(character, target, parameters.max_speed, parameters.max_acceleration, parameters.arrive_radius, parameters.slow_radius, time_to_target)
}

pub fn arrive_with_lead(character: &Kinematic, target: &Kinematic, max_speed: f32, max_acceleration: f32, target_radius: f32, slow_radius: f32, time_to_target: f32) -> Option<SteeringOutput> {

    let lead_target = Kinematic {
        position: target.position + target.velocity,
        ..*target
    };

    arrive(character, &lead_target, max_speed, max_acceleration, target_radius, slow_radius, time_to_target)

}

pub fn arrive_ex(character: &Kinematic, target: &Kinematic, parameters: SteeringParameters, time_to_target: f32) -> Option<SteeringOutput> {
    arrive(
        character,
        target,
        parameters.max_speed,
        parameters.max_acceleration,
        parameters.arrive_radius,
        parameters.slow_radius,
        time_to_target
    )
}

pub fn arrive(character: &Kinematic, target: &Kinematic, max_speed: f32, max_acceleration: f32, target_radius: f32, slow_radius: f32, time_to_target: f32) -> Option<SteeringOutput> {

    let vector_to_target = target.position - character.position;
    let distance = vector_to_target.length();

    if distance < target_radius {
        return None
    }

    let target_speed = if distance < slow_radius {
        max_speed * distance / slow_radius
    } else {
        max_speed
    };

    let target_velocity = vector_to_target.normalize() * target_speed;
    let result_velocity = (target_velocity - character.velocity) / time_to_target;

    let adjusted_velocity = if result_velocity.length() > max_acceleration {
        result_velocity.normalize() * max_acceleration  
    } else {
        result_velocity
    };

    Some(SteeringOutput {
        linear: adjusted_velocity,
        angular: 0.0
    })

}

pub fn map_to_range_new(r: f32) -> f32 {
    let two_pi = 2.0 * PI;
    let wrapped_angle = ((r - PI) % two_pi + two_pi) % two_pi;
    if wrapped_angle >= PI {
        wrapped_angle - two_pi
    } else {
        wrapped_angle
    }
}

pub fn map_to_range_fmod(r: f32) -> f32 {
    ((r - PI) % PI + PI) % PI
}

pub fn map_to_range_basic(r: f32) -> f32 {
    r % PI
}

pub fn map_to_range(r: f32) -> f32 {
    (r + PI) % (PI*2.0) - PI
}

pub fn map_to_range_old(r: f32) -> f32 {
    // note: ARGHHHH
    r % (2.0*PI)
    // r.rem_euclid(2.0*PI) - PI
}

pub fn shortest_arc_old(r: f32, t: f32) -> f32 {
    if (t - r).abs() < PI {
        t - r
    } else if t > r {
        t - r - PI*2.0
    } else {
        t - r + PI*2.0
    }
}

pub fn shortest_arc(r: f32, t: f32) -> f32 {
    let max = PI*2.0;
    let delta = (t - r) % max;
    2.0 * delta % (max - delta)
}

pub fn align_ex(character: &Kinematic, target: &Kinematic, parameters: SteeringParameters, time_to_target: f32) -> Option<SteeringOutput> {
    align(character, target, parameters.align_max_rotation, parameters.align_max_angular_acceleration, parameters.align_radius, parameters.align_slow_radius, time_to_target)
}

pub fn align(character: &Kinematic, target: &Kinematic, max_rotation: f32, max_angular_acceleration: f32, target_radius: f32, slow_radius: f32, time_to_target: f32) -> Option<SteeringOutput> {

    // let rotation_to_target = target.orientation
    let rotation_to_target = map_to_range(target.orientation - character.orientation);
    let rotation_size = rotation_to_target.abs();

    if rotation_size < target_radius {
        return None
    }

    let target_rotation = if rotation_to_target < slow_radius {
        max_rotation * rotation_size / slow_radius
    } else {
        max_rotation
    } * rotation_to_target / rotation_size;

    let result_rotation = target_rotation - character.orientation;
    let result_rotation_diff = result_rotation / time_to_target;
    let result_angular_acceleration = result_rotation_diff.abs();

    let scaled_angular_acceleration = if result_angular_acceleration > max_angular_acceleration {
        (result_rotation_diff / result_angular_acceleration) * max_angular_acceleration
    } else {
        result_rotation_diff
    };

    Some(SteeringOutput {
        linear: vec2(0.0, 0.0),
        angular: scaled_angular_acceleration
    })

}

pub fn face_with_lead_ex(character: &Kinematic, target: &Kinematic, parameters: SteeringParameters, time_to_target: f32) -> Option<SteeringOutput> {
    face_with_lead(character, target, parameters.align_max_rotation, parameters.align_max_angular_acceleration, parameters.align_radius, parameters.align_slow_radius, time_to_target)
}

pub fn face_with_lead(character: &Kinematic, target: &Kinematic, max_rotation: f32, max_angular_acceleration: f32, target_radius: f32, slow_radius: f32, time_to_target: f32) -> Option<SteeringOutput> {

    let predicted_target_position = target.predicted_position(1.0 / 60.0);

    let lead_target = Kinematic {
        position: predicted_target_position,
        ..*target
    };

    face(character, &lead_target, max_rotation, max_angular_acceleration, target_radius, slow_radius, time_to_target)

}

pub fn face_ex(character: &Kinematic, target: &Kinematic, parameters: SteeringParameters, time_to_target: f32) -> Option<SteeringOutput> {
    face(character, target, parameters.align_max_rotation, parameters.align_max_angular_acceleration, parameters.align_radius, parameters.align_slow_radius, time_to_target)
}

pub fn face(character: &Kinematic, target: &Kinematic, max_rotation: f32, max_angular_acceleration: f32, target_radius: f32, slow_radius: f32, time_to_target: f32) -> Option<SteeringOutput> {

    let vector_to_target = target.position - character.position;

    if vector_to_target.length() < 0.01 {
        return Some(SteeringOutput::default())
    }

    let adjusted_target = Kinematic {
        orientation: -vector_to_target.as_angle(),
        ..*target
    };

    align(character, &adjusted_target, max_rotation, max_angular_acceleration, target_radius, slow_radius, time_to_target)

}

pub fn velocity_match_ex(character: &Kinematic, target: &Kinematic, parameters: SteeringParameters, time_to_target: f32) -> SteeringOutput {
    velocity_match(character, target, parameters.max_acceleration, time_to_target)
}

pub fn velocity_match(character: &Kinematic, target: &Kinematic, max_acceleration: f32, time_to_target: f32) -> SteeringOutput {

    let target_velocity = (target.velocity - character.velocity) / time_to_target;

    let scaled_velocity = if target_velocity.length() > max_acceleration {
        target_velocity.normalize() * max_acceleration
    } else {
        target_velocity
    };

    SteeringOutput {
        linear: scaled_velocity,
        angular: 0.0
    }
    
}

pub fn look_where_you_are_going_ex(character: &Kinematic, target: &Kinematic, parameters: SteeringParameters, time_to_target: f32) -> Option<SteeringOutput> {
    look_where_you_are_going(character, target, parameters.align_max_rotation, parameters.align_max_angular_acceleration, parameters.align_radius, parameters.align_slow_radius, time_to_target)
}

pub fn look_where_you_are_going(character: &Kinematic, target: &Kinematic, max_rotation: f32, max_angular_acceleration: f32, target_radius: f32, slow_radius: f32, time_to_target: f32) -> Option<SteeringOutput> {

    let vector_to_target = character.velocity;

    let velocity_based_target = Kinematic {
        position: character.position + vector_to_target,
        ..*target
    };

    face(character, &velocity_based_target, max_rotation, max_angular_acceleration, target_radius, slow_radius, time_to_target)

}

pub fn wander(_character: &Kinematic, _target: &Kinematic, _max_acceleration: f32, _wander_offset: f32, _wander_radius: f32, _wander_rate: f32, _wander_orientation: f32) -> SteeringOutput {

    SteeringOutput {
        linear: vec2(0.0, 0.0),
        angular: 0.0
    }
}

pub fn separation<'a>(character: &Kinematic, targets: impl Iterator<Item=Kinematic>, max_acceleration: f32, threshold: f32, decay_coefficient: f32) -> SteeringOutput {

    let mut repulsion: Vec2 = Vec2::ZERO;

    for target in targets {
        let vector_to_target = target.position - character.position;
        let distance = vector_to_target.length();

        if distance < threshold {
            let strength = (decay_coefficient / (distance * distance)).min(max_acceleration);
            repulsion -= strength * vector_to_target.normalize();
        }
    }

    SteeringOutput {
        linear: repulsion,
        angular: 0.0
    }
}

pub fn blend_steering_behaviours(_behaviours: &[&SteeringBehaviour]) -> SteeringOutput {

    SteeringOutput {
        ..Default::default()
    }

}