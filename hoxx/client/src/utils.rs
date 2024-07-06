use hoxx_shared::ClientColor;
use macroquad::color::Color;

pub fn to_macroquad_color(client_color: ClientColor) -> Color {
    Color {
        r: client_color.r,
        g: client_color.g,
        b: client_color.b,
        a: 1.0
    }
}