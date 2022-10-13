use macroquad::prelude::*;

pub fn draw_text_centered(text: &str, x: f32, y: f32, font_size: f32, colour: Color) {
    let TextDimensions { width: t_w, height: t_h, .. } = measure_text(text, None, font_size as u16, 1.0);
    draw_text(text, x - (t_w/2.0), y - (t_h/2.0), font_size, colour);
}

pub fn draw_texture_centered(texture: Texture2D, x: f32, y: f32, colour: Color) {
    let w = texture.width();
    let h = texture.height();
    draw_texture(texture, x - (w / 2.0), y - (h / 2.0), colour);
}

pub fn draw_texture_with_rotation(texture: Texture2D, x: f32, y: f32, colour: Color, rotation: f32) {
    draw_texture_ex(texture, x, y, colour, DrawTextureParams {
        rotation: rotation,
        ..Default::default()
    });
}

pub fn draw_texture_centered_with_rotation(texture: Texture2D, x: f32, y: f32, colour: Color, rotation: f32) {
    let w = texture.width();
    let h = texture.height();
    draw_texture_ex(texture, x - (w / 2.0), y - (h / 2.0), colour, DrawTextureParams {
        rotation: rotation,
        ..Default::default()
    });
}

pub fn draw_texture_centered_with_rotation_frame(texture: Texture2D, x: f32, y: f32, colour: Color, rotation: f32, frame: i32, num_v_frames: i32, flip: bool) {
    let w = texture.width() / num_v_frames as f32;
    let h = texture.height();
    let atlas_rect = Rect {
        x: w * frame as f32, y: 0.0,
        w: w, h: h
    };
    let flip_mul = if flip { 1 } else { -1 };
    draw_texture_ex(texture, x - (w / 2.0) * flip_mul as f32, y - (h / 2.0), colour, DrawTextureParams {
        source: Some(atlas_rect),
        dest_size: Some(vec2(w * flip_mul as f32, h)),
        rotation: rotation,
        ..Default::default()
    });
}

pub fn draw_arrow(x1: f32, y1: f32, x2: f32, y2: f32, thickness: f32, head_size: f32, colour: Color) {

    let offset = vec2(x2 - x1, y2 - y1).normalize() * head_size;
    let l = vec2(x2, y2) + vec2(offset.y, -offset.x) - offset;
    let r = vec2(x2, y2) - vec2(offset.y, -offset.x) - offset;

    draw_line(x2, y2, l.x, l.y, thickness, colour);
    draw_line(x2, y2, r.x, r.y, thickness, colour);
    draw_line(x1, y1, x2, y2, thickness, colour);

}

pub fn random_binomial() -> f32 {
    rand::gen_range(0.0, 1.0) - rand::gen_range(0.0, 1.0)
}

/// Can be used to map the input value v in the range [[0.0, value_max]] to a range [[min, max]].
pub fn normalize(v: f32, min: f32, max: f32, value_max: f32) -> f32 {
    (min + v) / (value_max / (max - min))
}

pub fn draw_rectangle_lines_centered(x: f32, y: f32, w: f32, h: f32, thickness: f32, color: Color) {
    draw_rectangle_lines(x - w/2.0, y - h/2.0, w, h, thickness, color);
}

pub fn is_point_inside_rect(p: &Vec2, r: &Rect) -> bool {
    let is_outside = p.x < r.x || p.x > r.x + r.w || p.y < r.y || p.y > r.y + r.h;
    !is_outside
}

pub fn is_point_inside_screen(p: Vec2, padding: f32) -> bool {
    let w = screen_width();
    let h = screen_height();
    let screen_rect = Rect {
        x: padding,
        y: padding,
        w: w - padding,
        h: h - padding,
    };
    is_point_inside_rect(&p, &screen_rect)
}

/// Returns the vector in the set of vectors that is most similar to the vector v.
pub fn most_aligned(v: Vec2, mut vectors: impl Iterator::<Item=Vec2>) -> Option<Vec2> {

    let first_vector = vectors.nth(0)?;

    let mut max_vector = first_vector;
    let mut max_d = first_vector.dot(v);

    for s in vectors {
        let d = s.dot(v);
        if d > max_d {
            max_vector = s;
            max_d = d;
        }
    }

    Some(max_vector)

}