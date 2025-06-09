#![feature(let_chains)]
use std::f32::consts::PI;

use macroquad::prelude::*;
use utility::{draw_arrow, DebugText};

fn window_conf() -> Conf {
    Conf {
        window_title: "product".to_owned(),
        sample_count: 4,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {

    let mut debug_text = DebugText::new();

    loop {

        debug_text.new_frame();
        clear_background(WHITE);

        let (c_x, c_y) = (screen_width() / 2.0, screen_height() / 2.0);
        let (m_x, m_y) = mouse_position();
        let (v_x, v_y) = (m_x - c_x, m_y - c_y);

        draw_arrow(c_x, c_y, m_x, m_y, 2.0, 8.0, BLACK);

        let (ts_x, ts_y) = (c_x, c_y + c_y / 2.0);
        draw_arrow(c_x, c_y, ts_x, ts_y, 2.0, 8.0, RED);

        let t = vec2(ts_x - c_x, ts_y - c_y);
        debug_text.draw_text(format!("center x: {}, center y: {}", c_x, c_y), utility::TextPosition::TopLeft, BLACK);
        debug_text.draw_text(format!("mouse x: {}, mouse y: {}", m_x, m_y), utility::TextPosition::TopLeft, BLACK);

        debug_text.skip_line(utility::TextPosition::TopLeft);

        debug_text.draw_text(format!("vector x: {}, vector y: {} (black)", v_x, v_y), utility::TextPosition::TopLeft, BLACK);
        debug_text.draw_text(format!("target x: {}, target y: {} (red)", t.x, t.y), utility::TextPosition::TopLeft, BLACK);

        debug_text.skip_line(utility::TextPosition::TopLeft);

        let d = vec2(v_x, v_y).dot(t);
        let p = vec2(v_x, v_y).perp_dot(t);

        debug_text.draw_text("raw vector values", utility::TextPosition::TopLeft, BLACK);
        debug_text.draw_text(format!("vector dot target = {}", d), utility::TextPosition::TopLeft, BLACK);
        debug_text.draw_text(format!("vector perp_dot target = {}", p), utility::TextPosition::TopLeft, BLACK);

        debug_text.skip_line(utility::TextPosition::TopLeft);

        let n_d = vec2(v_x, v_y).normalize().dot(t.normalize());
        let n_p = vec2(v_x, v_y).normalize().perp_dot(t.normalize());
        debug_text.draw_text("both normalized", utility::TextPosition::TopLeft, BLACK);
        debug_text.draw_text(format!("vector dot target = {:.2}", n_d), utility::TextPosition::TopLeft, BLACK);
        debug_text.draw_text(format!("vector perp_dot target = {:.2}", n_p), utility::TextPosition::TopLeft, BLACK);

        debug_text.skip_line(utility::TextPosition::TopLeft);

        let a = if n_p < 0.0 { n_d.acos() } else { -n_d.acos() };
        debug_text.draw_text(format!("angle between (normalized) vector and target = {:.2} degrees ({:.2} radians, {:.2} pi)", a.to_degrees(), a, a / PI), utility::TextPosition::TopLeft, BLACK);

        next_frame().await;

    }

}