
use macroquad::prelude::*;
use utility::DebugText;

fn ease_in_quad(t: f32) -> f32 {
    t * t
}

fn ease_out_quad(t: f32) -> f32 {
    1.0 - (1.0 - t) * (1.0 - t)
}

fn get_eased_path<EaseInFn: Fn(f32) -> f32, EaseOutFn: Fn(f32) -> f32>(ease_in_fn: EaseInFn, ease_out_fn: EaseOutFn, a: f32, b: f32, t: f32) -> f32 {
    if t < a {
        return ease_in_fn(t * (1.0 / a)) * a
    } else if t < (1.0 - b) {
        return a + (t - b)
    } else {
        return (1.0 - b) + ease_out_fn((t - (1.0 - b)) * (1.0 / b)) * b
    }
}

#[macroquad::main("easy")]
async fn main() {

    let mut debug_text = DebugText::new();

    loop {

        debug_text.new_frame();
        clear_background(WHITE);

        let s = 0.5;
        let t = ((get_time() * s) % 1.0) as f32;

        let a = 0.25;
        let b = 0.25;
        let e_t = get_eased_path(ease_in_quad, ease_out_quad, a, b, t);

        let x = screen_width() * e_t;
        draw_rectangle(x, screen_height() * 0.5, 32.0, 32.0, RED);

        debug_text.draw_text(format!("t: {}", t), utility::TextPosition::TopLeft, BLACK);
        debug_text.draw_text(format!("e_t: {}", e_t), utility::TextPosition::TopLeft, BLACK);
        debug_text.draw_text(format!("ease in: [{}, {}], linear: [{}, {}], ease out: [{}, {}]", 0, a, a, 1.0 - b, 1.0 - b, 1.0), utility::TextPosition::TopLeft, BLACK);

        next_frame().await;

    }

}