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

fn draw_arrow(origin: Vec2, ray: Vec2, arrow_length: f32, thickness: f32) {

    let line_end = origin + ray;

    draw_line(
        origin.x(),
        origin.y(),
        line_end.x(),
        line_end.y(),
        thickness,
        WHITE
    );

    let p1 = vec2(-ray.y(), ray.x());
    let p2 = vec2(ray.y(), -ray.x());
    let left = -ray.lerp(p1, 0.5);
    let right = -ray.lerp(p2, 0.5);
    let left_end = line_end + left.normalize() * arrow_length;
    let right_end = line_end + right.normalize() * arrow_length;

    draw_line(
        line_end.x(),
        line_end.y(),
        left_end.x(),
        left_end.y(),
        thickness,
        WHITE
    );

    draw_line(
        line_end.x(),
        line_end.y(),
        right_end.x(),
        right_end.y(),
        thickness,
        WHITE
    );

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

        top_set = is_key_down(KeyCode::W);
        bottom_set = is_key_down(KeyCode::S);
        left_set = is_key_down(KeyCode::A);
        right_set = is_key_down(KeyCode::D);

        let screen_center = vec2(screen_width() / 2.0,screen_height() / 2.0);

        let corners = [
            top_set.then_some(vec2(0.0, -1.0)),
            bottom_set.then_some(vec2(0.0, 1.0)),
            left_set.then_some(vec2(-1.0, 0.0)),
            right_set.then_some(vec2(1.0, 0.0))
        ];

        let current_corners = corners.iter().filter_map(|v| *v);
        let current_corner_vector = corner_vector(current_corners);
        let vector_length = 32.0;

        if let Some(current_vector) = current_corner_vector {

            let screen_vector = current_vector * vector_length;

            draw_arrow(
                screen_center,
                screen_vector,
                16.0, // arrow length
                4.0
            );
            
        }

        draw_debug_text(format!("top set: {} (W)", top_set), 32.0, 32.0);
        draw_debug_text(format!("bottom set: {} (S)", bottom_set), 32.0, 48.0);
        draw_debug_text(format!("left set: {} (A)", left_set), 32.0, 64.0);
        draw_debug_text(format!("right set: {} (D)", right_set), 32.0, 80.0);

        next_frame().await

    }

}
