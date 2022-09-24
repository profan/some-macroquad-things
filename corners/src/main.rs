#![feature(let_chains)]
use std::f32::consts::PI;

use macroquad::prelude::*;
use utility::{RotatedBy, draw_arrow};

fn discretize(direction: Vec2, mut sides: impl Iterator::<Item=Vec2>) -> Option<Vec2> {

    let first_side = sides.nth(0)?;
    
    let mut min_side = first_side;
    let mut min_d = first_side.dot(direction);

    for s in sides {
        let d = s.dot(direction);
        if d > min_d {
            min_side = s;
            min_d = d;
        }
    }

    Some(min_side)

}

#[macroquad::main("corners")]
async fn main() {

    fn draw_debug_text(text: String, x: f32, y: f32) {
        draw_text(
            text.as_str(),
            x, y,
            16.0, WHITE
        );
    }

    loop {

        let _dt = get_frame_time();

        clear_background(BLUE);

        let mouse_btn_down = is_mouse_button_down(MouseButton::Left);
        let mouse_pos_vec : Vec2 = mouse_position().into();

        let top = if is_key_down(KeyCode::W) { vec2(0.0, -1.0) } else { vec2(0.0, 0.0) };
        let bottom = if is_key_down(KeyCode::S) { vec2(0.0, 1.0) } else { vec2(0.0, 0.0) };
        let left = if is_key_down(KeyCode::A) { vec2(-1.0, 0.0) } else { vec2(0.0, 0.0) };
        let right = if is_key_down(KeyCode::D) { vec2(1.0, 0.0) } else { vec2(0.0, 0.0) };

        let screen_center: Vec2 = vec2(screen_width() / 2.0, screen_height() / 2.0);
        let vector_to_mouse: Vec2 = (mouse_pos_vec - screen_center).normalize();
        let combined_inputs = top + bottom + left + right
            + if mouse_btn_down { vector_to_mouse } else { vec2(0.0, 0.0) };

        let has_any_input = combined_inputs.x.abs() > 0.0 || combined_inputs.y.abs() > 0.0;
         
        let current_corner_vector = discretize(
            combined_inputs,
            [
                vec2(0.0, -1.0), // n
                vec2(0.0, -1.0).rotated_by(PI / 4.0), // ne
                vec2(0.0, 1.0), // s
                vec2(0.0, 1.0).rotated_by(PI / 4.0), // sw
                vec2(-1.0, 0.0), // w
                vec2(-1.0, 0.0).rotated_by(PI / 4.0), // nw
                vec2(1.0, 0.0), // e
                vec2(1.0, 0.0).rotated_by(PI / 4.0) // se
            ].into_iter()
        );

        if let Some(current_vector) = current_corner_vector && has_any_input {

            let vector_length = 32.0;
            let screen_vector = current_vector * vector_length;

            draw_arrow(
                screen_center.x, screen_center.y,
                screen_center.x + screen_vector.x,
                screen_center.y + screen_vector.y,
                4.0, // thickness
                16.0, // head size
                WHITE
            );
            
        }

        draw_debug_text(format!("left mouse: {}", mouse_btn_down), 32.0, 32.0);
        draw_debug_text(format!("up pressed: {} (W)", is_key_down(KeyCode::W)), 32.0, 48.0);
        draw_debug_text(format!("down pressed: {} (S)", is_key_down(KeyCode::S)), 32.0, 64.0);
        draw_debug_text(format!("left pressed: {} (A)", is_key_down(KeyCode::A)), 32.0, 80.0);
        draw_debug_text(format!("right pressed: {} (D)", is_key_down(KeyCode::D)), 32.0, 96.0);

        next_frame().await

    }

}
