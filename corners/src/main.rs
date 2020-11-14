#![feature(bool_to_option)]
use macroquad::prelude::*;

fn corner_vector(sides: impl Iterator::<Item=Vec2>) -> Option<Vec2> {

    let (result_vector, num_sides) = sides.fold((Vec2::zero(), 0.0), |(acc_v, acc_i), v| (acc_v + v, acc_i + 1.0));
    let result = (result_vector / num_sides).normalize();

    if !result.is_nan().any() {
        Some(result)
    } else {
        None
    }

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
    
    fn draw_box(x: f32, y: f32, size: f32) {
        draw_rectangle(
            x - (size/2.0), y - (size/2.0),
            size, size,
            WHITE
        )
    }

    let mut top_set = false;
    let mut bottom_set = false;
    let mut left_set = false;
    let mut right_set = false;

    loop {

        let _dt = get_frame_time();

        clear_background(BLUE);

        if is_key_pressed(KeyCode::W) { top_set = !top_set; };
        if is_key_pressed(KeyCode::S) { bottom_set = !bottom_set; };
        if is_key_pressed(KeyCode::A) { left_set = !left_set; };
        if is_key_pressed(KeyCode::D) { right_set = !right_set; };

        let screen_center = vec2(screen_width() / 2.0,screen_height() / 2.0);

        let corners = [
            top_set.then_some(vec2(0.0, 1.0)),
            bottom_set.then_some(vec2(0.0, -1.0)),
            left_set.then_some(vec2(1.0, 0.0)),
            right_set.then_some(vec2(-1.0, 0.0))
        ];

        let current_corners = corners.iter().filter_map(|v| *v);
        let current_corner_vector = corner_vector(current_corners);
        let vector_length = 16.0;

        if let Some(current_vector) = current_corner_vector {
            let screen_vector_end = screen_center + current_vector * vector_length;
            draw_line(
                screen_center.x(),
                screen_center.y(),
                screen_vector_end.x(),
                screen_vector_end.y(),
                4.0,
                WHITE
            );
            
        }

        let box_size = vector_length;

        if top_set {
            draw_box(screen_center.x(), screen_center.y() - box_size, box_size);
        }

        if bottom_set {
            draw_box(screen_center.x(), screen_center.y() + box_size, box_size);
        }

        if left_set {
            draw_box(screen_center.x() - box_size, screen_center.y(), box_size);
        }

        if right_set {
            draw_box(screen_center.x() + box_size, screen_center.y(), box_size);
        }

        draw_debug_text(format!("top set: {} (W)", top_set), 32.0, 32.0);
        draw_debug_text(format!("bottom set: {} (S)", bottom_set), 32.0, 48.0);
        draw_debug_text(format!("left set: {} (A)", left_set), 32.0, 64.0);
        draw_debug_text(format!("right set: {} (D)", top_set), 32.0, 80.0);

        next_frame().await

    }

}
