use hoxx_shared::{HEX_IS_VERTICAL, HEX_SIZE};
use macroquad::{color::Color, shapes::draw_hexagon};

const HEX_BORDER_SIZE: f32 = 1.0;

pub fn draw_hex(x: f32, y: f32, fill_color: Color, border_color: Color) {
    draw_hexagon(
        x, y,
        HEX_SIZE,
        HEX_BORDER_SIZE,
        HEX_IS_VERTICAL,
        border_color,
        fill_color
    );
}