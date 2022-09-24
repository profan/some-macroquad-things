use macroquad::prelude::*;
use std::f32::consts::*;

pub trait AsAngle {
    fn as_angle(&self) -> f32;
}

pub trait AsVector {
    fn as_vector(&self) -> Vec2;
}

pub trait WithAlpha {
    fn with_alpha(&self, alpha: f32) -> Self;
}

impl WithAlpha for Color {
    fn with_alpha(&self, alpha: f32) -> Self {
        Color::new(self.r, self.g, self.b, alpha)
    }
}

impl AsAngle for Vec2 {
    fn as_angle(&self) -> f32 {
        self.angle_between(vec2(0.0, -1.0))
    }
}

impl AsVector for f32 {      
    fn as_vector(&self) -> Vec2 {
        vec2(
            (self + (PI/2.0)).cos(),
            (self + (PI/2.0)).sin()
        )
    }
}

pub trait AsPerpendicular {
    fn perpendicular(&self) -> Self;
    fn perpendicular_ccw(&self) -> Self;
}

impl AsPerpendicular for Vec2 {
    fn perpendicular(&self) -> Self {
        vec2(-self.y, self.x)
    }
    fn perpendicular_ccw(&self) -> Self {
        vec2(self.y, -self.x)
    }
}

pub trait DistanceBetween {
    fn distance_to(&self, other: Self) -> f32;
}

impl DistanceBetween for Vec2 {
    fn distance_to(&self, other: Vec2) -> f32 {
        (other - *self).length()
    }
}

pub trait RotatedBy {
    fn rotated_by(&self, angle: f32) -> Vec2;
}

impl RotatedBy for Vec2 {
    fn rotated_by(&self, angle: f32) -> Vec2 {
        vec2(
            self.x * angle.cos() - self.y * angle.sin(),
            self.x * angle.sin() + self.y * angle.cos()
        )
    }
}