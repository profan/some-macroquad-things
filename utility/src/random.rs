use macroquad::prelude::*;

trait Modulation {
    fn modulation(&self) -> Color;
}

pub fn nice_modulation(x: f32) -> f32 {
    1.0 + f32::cos(8.0*x)
}

pub fn other_nice_modulation(x: f32) -> f32 {
    ((1.0 + f32::cos(12.0*x)) / 3.0) + 0.75
}