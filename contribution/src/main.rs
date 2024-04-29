use macroquad::prelude::*;
use utility::{AdjustHue};

#[derive(Clone, Copy)]
struct Triangle {
    a: Vec2,
    b: Vec2,
    c: Vec2
}

fn is_point_inside_triangle(p: Vec2, t: Triangle) -> bool {

    let ax = t.a.x;
    let ay = t.a.y;

    let bx = t.b.x;
    let by = t.b.y;

    let cx = t.c.x;
    let cy = t.c.y;

    // Segment A to B
    let side_1 = (p.x - bx) * (ay - by) - (ax - bx) * (p.y - by);

    // Segment B to C
    let side_2 = (p.x - cx) * (by - cy) - (bx - cx) * (p.y - cy);

    // Segment C to A
    let side_3 = (p.x - ax) * (cy - ay) - (cx - ax) * (p.y - ay);
    
    // All the signs must be positive or all negative
    return (side_1 < 0.0) == (side_2 < 0.0)
        && (side_2 < 0.0) == (side_3 < 0.0)
        && (side_1 < 0.0) == (side_3 < 0.0)

}

fn barycentric_coordinate(p: Vec2, t: Triangle) -> Vec3 {

    let ax = t.a.x;
    let ay = t.a.y;

    let bx = t.b.x;
    let by = t.b.y;

    let cx = t.c.x;
    let cy = t.c.y;

    // Segment A to B
    let side_1 = (p.x - bx) * (ay - by) - (ax - bx) * (p.y - by);

    // Segment B to C
    let side_2 = (p.x - cx) * (by - cy) - (bx - cx) * (p.y - cy);

    // Segment C to A
    let side_3 = (p.x - ax) * (cy - ay) - (cx - ax) * (p.y - ay);

    let v1 = side_1 / (t.b - t.a).length();
    let v2 = side_2 / (t.c - t.b).length();
    let v3 = side_3 / (t.a - t.c).length();
    let t = v1 + v2 + v3;

    return vec3(
        v1 / t,
        v2 / t,
        v3 / t
    )

}

fn calculate_contribution(p: Vec2, t: Triangle) -> Vec3 {

    let d_a = p.distance(t.a);
    let d_b = p.distance(t.b);
    let d_c = p.distance(t.c);

    let total = d_a + d_b + d_c;

    let a = d_a / total;
    let b = d_b / total;
    let c = d_c / total;

    // vec3(
    //     (b + c) - a,
    //     (a + c) - b,
    //     (a + b) - c
    // ).normalize()

    vec3(
        ((b + c) - a).max(0.0),
        ((a + c) - b).max(0.0),
        ((a + b) - c).max(0.0)
    )

}

#[macroquad::main("contribution")]
async fn main() {

    let should_draw_distances = false;
    
    loop {

        let font_size = 32.0;
        let _dt = get_frame_time();

        clear_background(WHITE);

        let padding = 100.0;
        let w = screen_width();
        let h = screen_height();

        let a = vec2(w / 2.0, padding);
        let b = vec2(padding, h - padding);
        let c = vec2(w - padding, h - padding);
        let t = 4.0;

        let mouse_pos: Vec2 = mouse_position().into();
        let our_triangle = Triangle { a, b, c };

        if is_point_inside_triangle(mouse_pos, our_triangle) {

            // let b_c = barycentric_coordinate(mouse_pos, our_triangle);
            let v_c = calculate_contribution(mouse_pos, our_triangle);

            // draw_text(format!("{} ({})", b_c, b_c.x + b_c.y + b_c.z).as_str(), mouse_pos.x, mouse_pos.y, font_size, BLACK);
            draw_text(format!("{}", v_c).as_str(), mouse_pos.x, mouse_pos.y + font_size, font_size, BLACK);
            draw_text(format!("- total: {}", v_c.x + v_c.y + v_c.z).as_str(), mouse_pos.x, mouse_pos.y + font_size * 2.0, font_size, BLACK);

            draw_line(a.x, a.y, mouse_pos.x, mouse_pos.y, t, BLACK.lighten(0.25));
            draw_line(b.x, b.y, mouse_pos.x, mouse_pos.y, t, BLACK.lighten(0.25));
            draw_line(c.x, c.y, mouse_pos.x, mouse_pos.y, t, BLACK.lighten(0.25));

            // contributions?
            draw_text(format!("{} %", v_c.x * 100.0).as_str(), a.x - font_size / 2.0, a.y - font_size / 2.0, font_size, BLACK);
            draw_text(format!("{} %", v_c.y * 100.0).as_str(), b.x - font_size / 2.0, b.y + font_size, font_size, BLACK);
            draw_text(format!("{} %", v_c.z * 100.0).as_str(), c.x, c.y + font_size, font_size, BLACK);

            // distances ?

            if should_draw_distances {

                let d1 = mouse_pos.distance(our_triangle.a);
                let d2 = mouse_pos.distance(our_triangle.b);
                let d3 = mouse_pos.distance(our_triangle.c);

                draw_text(format!("{} px", d1).as_str(), a.x - font_size / 2.0, (a.y - font_size / 2.0) + font_size, font_size, BLACK);
                draw_text(format!("{} px", d2).as_str(), b.x - font_size / 2.0, (b.y + font_size) + font_size, font_size, BLACK);
                draw_text(format!("{} px", d3).as_str(), c.x, (c.y + font_size) + font_size, font_size, BLACK);

            }
            
        } else {

            draw_text("0 %", a.x - font_size / 2.0, a.y - font_size / 2.0, font_size, BLACK);
            draw_text("0 %", b.x - font_size / 2.0, b.y + font_size, font_size, BLACK);
            draw_text("0 %", c.x, c.y + font_size, font_size, BLACK);
            
        }

        draw_triangle_lines(a, b, c, t, BLACK);

        next_frame().await;

    }

}
