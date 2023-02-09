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

pub fn lerp(a: f32, b: f32, v: f32) -> f32 {
    a * (1.0 - v) + b * v
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

/// Returns the current screen dimensions as a Vec2.
pub fn screen_dimensions() -> Vec2 {
    return vec2(screen_width(), screen_height())
}

/// Returns the intersection point of the ray defined by origin and direction and the line which is defined as passing through p1 and p2, if any.
fn ray_line_intersection(origin: Vec2, direction: Vec2, p1: Vec2, p2: Vec2) -> Option<Vec2> {

    let dp = direction.dot((p2 - p1).normalize());
    if dp < 0.0 {
        return None;
    }

    let (x1, y1) = origin.into();
    let (x2, y2) = (origin + direction).into();
    let (x3, y3) = p1.into();
    let (x4, y4) = p2.into();

    let d = (x1 - x2) * (y3 - y4) - (y1 - y2) * (x3 - x4);

    if d != 0.0 {
        let p_x = ((x1 * y2 - y1 * x2) * (x3 - x4) - (x1 - x2) * (x3 * y4 - y3 * x4)) / d;
        let p_y = ((x1 * y2 - y1 * x2) * (y3 - y4) - (y1 - y2) * (x3 * y4 - y3 * x4)) / d;
        return Some(vec2(p_x, p_y));
    }

    return None;

}

// Returns true if the point c is on the line segment between a and b.
fn is_between(a: Vec2, b: Vec2, c: Vec2) -> bool {
    a.distance(c) + c.distance(b) == a.distance(b)
}

/// Returns the intersection point of the ray defined by origin_a and direction_a and the ray defined by origin_b and direction_b, if any.
fn ray_ray_intersection(origin_a: Vec2, direction_a: Vec2, origin_b: Vec2, direction_b: Vec2) -> Option<Vec2> {

    if origin_a == origin_b
    {
        return Some(origin_a);
    }

    let d = origin_b - origin_a;
    let determinant = direction_b.perp_dot(direction_a);
    if determinant != 0.0 {
        let u = (d.y * direction_b.x - d.x * direction_b.y) / determinant;
        let v = (d.y * direction_a.x - d.x * direction_a.y) / determinant;
        if u >= 0.0 && v >= 0.0 {
            let result = origin_a + direction_a * u;
            return Some(result);
        }
    }

    return None;

}


#[cfg(test)]
mod tests {

    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_is_between_simple() {

        let a = vec2(0.0, 0.0);
        let b = vec2(1.0, 0.0);
        let c = vec2(0.5, 0.0);
        
        assert!(is_between(a, b, c));

    }

    #[test]
    fn test_is_between_simple_false() {

        let a = vec2(0.0, 0.0);
        let b = vec2(1.0, 0.0);
        let c = vec2(-0.5, 0.0);
        
        assert!(is_between(a, b, c) == false);

    }

    #[test]
    fn test_ray_ray_intersection_simple() {

        let ray_origin_a = vec2(0.0, 0.0);
        let ray_dir_a = vec2(1.0, 0.0);

        let ray_origin_b = vec2(0.5, 0.5);
        let ray_dir_b = vec2(0.0, -1.0); // ray_origin_a + ray_dir_b = [0.5, -0.5]

        let expected_intersection = Some(vec2(0.5, 0.0));

        assert_eq!(expected_intersection, ray_ray_intersection(ray_origin_a, ray_dir_a, ray_origin_b, ray_dir_b));

    }

    #[test]
    fn test_ray_ray_intersection_non_overlap() {

        let ray_origin_a = vec2(0.0, 0.0);
        let ray_dir_a = vec2(1.0, 0.0);

        let ray_origin_b = vec2(0.5, 0.5);
        let ray_dir_b = vec2(0.0, 1.0);

        assert_eq!(None, ray_ray_intersection(ray_origin_a, ray_dir_a, ray_origin_b, ray_dir_b));

    }

}