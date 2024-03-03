use std::f32::consts::PI;

use macroquad::prelude::*;

/// Draws some text centered over a given position in screen space.
pub fn draw_text_centered(text: &str, x: f32, y: f32, font_size: f32, colour: Color) {
    let TextDimensions { width: t_w, height: t_h, .. } = measure_text(text, None, font_size as u16, 1.0);
    draw_text(text, x - (t_w/2.0), y - (t_h/2.0), font_size, colour);
}

/// Draws a textured quad with the origin offset by half the texture size (so that the center is over x, y).
pub fn draw_texture_centered(texture: Texture2D, x: f32, y: f32, colour: Color) {
    let w = texture.width();
    let h = texture.height();
    draw_texture(texture, x - (w / 2.0), y - (h / 2.0), colour);
}

/// Draws a textured quad with a given rotation.
pub fn draw_texture_with_rotation(texture: Texture2D, x: f32, y: f32, colour: Color, rotation: f32) {
    draw_texture_ex(texture, x, y, colour, DrawTextureParams {
        rotation: rotation,
        ..Default::default()
    });
}

/// Draws a centered textured quad with a given rotation, see also [draw_texture_centered].
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

/// Draws a line with an arrowhead on the end, with a given head size, line thickness and colour.
pub fn draw_arrow(x1: f32, y1: f32, x2: f32, y2: f32, thickness: f32, head_size: f32, colour: Color) {

    let offset = vec2(x2 - x1, y2 - y1).normalize() * head_size;
    let l = vec2(x2, y2) + vec2(offset.y, -offset.x) - offset;
    let r = vec2(x2, y2) - vec2(offset.y, -offset.x) - offset;

    draw_line(x2, y2, l.x, l.y, thickness, colour);
    draw_line(x2, y2, r.x, r.y, thickness, colour);
    draw_line(x1, y1, x2, y2, thickness, colour);

}

/// Returns a random number in \[0.0, 1.0\]
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

/// Draws a rectangle with a given colour where the origin is the center of the rendered shape (given its width, height).
pub fn draw_rectangle_lines_centered(x: f32, y: f32, w: f32, h: f32, thickness: f32, color: Color) {
    draw_rectangle_lines(x - w/2.0, y - h/2.0, w, h, thickness, color);
}

/// Retuns true if the given point is inside the rect.
pub fn is_point_inside_rect(point: &Vec2, rect: &Rect) -> bool {
    let is_outside = point.x < rect.x || point.x > rect.x + rect.w || point.y < rect.y || point.y > rect.y + rect.h;
    !is_outside
}

/// Returns true if the point in screen space is inside the current screen dimensions, including optional padding.
pub fn is_point_inside_screen(point: Vec2, padding: f32) -> bool {
    let w = screen_width();
    let h = screen_height();
    let screen_rect = Rect {
        x: padding,
        y: padding,
        w: w - padding,
        h: h - padding,
    };
    is_point_inside_rect(&point, &screen_rect)
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
pub fn ray_line_intersection(origin: Vec2, direction: Vec2, p1: Vec2, p2: Vec2) -> Option<Vec2> {

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

/// Returns true if the point c is on the line segment between a and b (through abuse of the triangle inequality).
pub fn is_between(a: Vec2, b: Vec2, c: Vec2) -> bool {
    a.distance(c) + c.distance(b) == a.distance(b)
}

/// Returns the intersection point of the ray defined by origin_a and direction_a and the ray defined by origin_b and direction_b, if any.
pub fn ray_ray_intersection(origin_a: Vec2, direction_a: Vec2, origin_b: Vec2, direction_b: Vec2) -> Option<Vec2> {

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

/// Clamps the number to the given range, returning min if less than min, max if greater than max, otherwise returning the original value.
pub fn clamp<T : PartialOrd>(v: T, min: T, max: T) -> T {
    if v < min {
        min
    } else if v > max {
        max
    } else {
        v
    }
}

/// Gets the intersection in the plane defined by the normal and origin, given the ray.
pub fn intersect_ray_with_plane(ray_origin: Vec3, ray_direction: Vec3, plane_normal: Vec3, plane_origin: Vec3) -> Option<Vec3> {

    let denom = plane_normal.dot(ray_direction);
    let intersection = if denom > std::f32::EPSILON {
        let v = plane_origin - ray_origin;
        let d = v.dot(plane_normal) / denom;
        Some(ray_origin + ray_direction * d)
    } else {
        None
    };

    intersection

}

/// Projects a point in the plane defined by the normal and distance from the origin.
pub fn project_point_on_plane(point: Vec3, normal: Vec3, d: f32) -> Vec3 {

    let proj_point = point - (normal.dot(point) + d);

    proj_point

}

/// Projects the point onto a sphere with a specific radius.
pub fn project_point_on_sphere(point: Vec3, sphere_pos: Vec3, r: f32) -> Vec3 {

    let p = point - sphere_pos;
    let p_mag = p.length();
    let q = (r / p_mag) * p;
    let p_s = q + sphere_pos;

    p_s

}

/// Rotates the point using the origin as the pivot.
pub fn rotate_relative_to_origin(origin: Vec3, point: Vec3, rotation: Quat) -> Vec3 {

    let t1 = point - origin;
    let t2 = rotation * t1;
    let t3 = t2 + origin;

    t3

}

/// Returns the sign of the number, 1.0 if x > 0.0, -1.0 if x < 0.0, when x is 0 it returns 0.
pub fn sign(x: f32) -> f32 {
    if x > 0.0 {
        1.0 
    } else if x < 0.0 {
        -1.0
    } else {
        0.0
    }
}

/// Returns the value produced by the equation x - floor(x), should always be in the range \[0.0, 1.0\].
pub fn frac0(x: f32) -> f32 {
    x - x.floor()
}

/// Returns the value produced by the equation 1.0 - x + floor(x), should always be in the range \[0.0, 1.0\].
pub fn frac1(x: f32) -> f32 {
    1.0 - x + x.floor()
}

/// Returns true if the two rectangles intersect.
pub fn intersect_rect(a: &Rect, b: &Rect) -> bool {

    let a_pos = vec2(a.x, a.y);
    let b_pos = vec2(b.x, b.y);
    
    let dx = b_pos.x - a_pos.x;
    let px = (b.w / 2.0) + (a.w / 2.0) - dx.abs();
    if px <= 0.0 {
        return false;
    };

    let dy = b_pos.y - a_pos.y;
    let py = (b.h / 2.0) + (a.h / 2.0) - dy.abs();
    if py <= 0.0 {
        return false;
    };

    // SAT: both axises must be overlapping for an intersection to have occurred, thus we are here

    return true;
    
}

/// Wrap v around the range [lo, hi].
pub fn wrap(v: f32, lo: f32, hi: f32) -> f32 {
    (v - lo) % (hi - lo) + lo
}

/// Normalizes the specific angle in radians into the range [-PI, PI].
pub fn normalize_angle(a: f32) -> f32 {
    wrap(a, -PI, PI)
}

/// Returns the normalized angle difference between the two angles in radians.
pub fn angle_difference(a: f32, b: f32) -> f32 {
    let a = normalize_angle(a);
    let b = normalize_angle(b);
    normalize_angle(b - a)
}

pub fn lerp_angle(a: f32, b: f32, t: f32) -> f32 {
    let difference = angle_difference(a, b);
    normalize_angle(a + difference * t)
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