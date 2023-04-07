use macroquad::prelude::*;
use std::f32::consts::*;

use crate::lerp;

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

pub trait AdjustHue {
    fn darken(&self, v: f32) -> Self;
    fn lighten(&self, v: f32) -> Self;
}

impl AdjustHue for Color {
    fn darken(&self, v: f32) -> Self {
        Color {
            r: self.r - v,
            g: self.g - v,
            b: self.b - v,
            a: self.a
        }
    }

    fn lighten(&self, v: f32) -> Self {
        Color {
            r: self.r + v,
            g: self.g + v,
            b: self.b + v,
            a: self.a
        }
    }
}

pub trait LerpColor {
    fn lerp(&self, other: &Self, v: f32) -> Self;
}

impl LerpColor for Color {
    fn lerp(&self, other: &Color, v: f32) -> Color {
        Color {
            r: lerp(self.r, other.r, v),
            g: lerp(self.g, other.g, v),
            b: lerp(self.b, other.b, v),
            a: lerp(self.a, other.a, v)
        }
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
    fn rotated_by_around_origin(&self, angle: f32, origin: Vec2) -> Vec2;
}

impl RotatedBy for Vec2 {

    /// Returns the vector rotated by the specified angle in radians.
    fn rotated_by(&self, angle: f32) -> Vec2 {
        vec2(
            self.x * angle.cos() - self.y * angle.sin(),
            self.x * angle.sin() + self.y * angle.cos()
        )
    }

    /// Returns the vector rotated by the specific angle in radians, around a specific pivot point.
    fn rotated_by_around_origin(&self, angle: f32, origin: Vec2) -> Vec2 {

        // translate to origin
        let o_p = *self - origin;

        // rotate
        let r_p = o_p.rotated_by(angle);

        // translate back
        let f_p = r_p + origin;

        return f_p;
        
    }

}

pub trait Step {
    fn step(&self, step_size: Self) -> Self;
}

impl Step for f32 {
    fn step(&self, step_size: Self) -> Self {
        (self / step_size).floor() * step_size
    }
}

pub trait AsPolar {
    fn as_polar(&self) -> (f32, f32);
}

impl AsPolar for Vec2 {
    fn as_polar(&self) -> (f32, f32) {
        cartesian_to_polar(*self).into()
    }
}

pub trait AsCartesian {
    fn as_cartesian(&self) -> Vec2;
}

impl AsCartesian for (f32, f32) {
    fn as_cartesian(&self) -> Vec2 {
        polar_to_cartesian(self.0, self.1)
    }
}

pub trait WithY {
    fn with_y(&self, y: f32) -> Vec3;
}

impl WithY for Vec2 {
    fn with_y(&self, y: f32) -> Vec3 {
        vec3(self.x, y, self.y)
    }
}

pub trait FromRotationArcAround {
    fn from_rotation_arc_2d_around_y(from: Vec2, to: Vec2) -> Quat;
}

impl FromRotationArcAround for Quat {

    /// Given two 2D vectors in the XZ plane, return a rotation around the Y axis.
    fn from_rotation_arc_2d_around_y(from: Vec2, to: Vec2) -> Quat {
        assert!(from.is_normalized());
        assert!(to.is_normalized());

        const ONE_MINUS_EPSILON: f32 = 1.0 - 2.0 * core::f32::EPSILON;
        let dot = from.dot(to);
        if dot > ONE_MINUS_EPSILON {
            // 0° singulary: from ≈ to
            Quat::IDENTITY
        } else if dot < -ONE_MINUS_EPSILON {
            // 180° singulary: from ≈ -to
            const COS_FRAC_PI_2: f32 = 0.0;
            const SIN_FRAC_PI_2: f32 = 1.0;
            // rotation around y by PI radians
            Quat::from_xyzw(0.0, SIN_FRAC_PI_2, 0.0, COS_FRAC_PI_2)
        } else {
            // vector3 cross where z=0
            let y = from.x * to.y - to.x * from.y;
            let w = 1.0 + dot;
            // calculate length with x=0 and z=0 to normalize
            let len_rcp = 1.0 / (y * y + w * w).sqrt();
            Quat::from_xyzw(0.0, y * len_rcp, 0.0, w * len_rcp)
        }
    }
    
}
