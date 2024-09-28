use std::f32::consts::PI;

use macroquad::prelude::*;

use crate::{RotatedBy, WithY};

unsafe fn draw_quad(vertices: [(Vec3, Vec2, Color); 4]) {
    let context = get_internal_gl().quad_gl;
    let indices = [0, 1, 2, 0, 2, 3];
    let ab = vertices[0].0 - vertices[1].0;
    let ac = vertices[0].0 - vertices[3].0;
    let d = ab.cross(ac);
    let n = (d / d.length()).extend(0.0);
    let quad = [
        macroquad::models::Vertex {
            position: vertices[0].0,
            uv: vertices[0].1,
            color: vertices[0].2.into(),
            normal: n
        },
        macroquad::models::Vertex {
            position: vertices[1].0,
            uv: vertices[1].1,
            color: vertices[1].2.into(),
            normal: n
        },
        macroquad::models::Vertex {
            position: vertices[2].0,
            uv: vertices[2].1,
            color: vertices[2].2.into(),
            normal: n
        },
        macroquad::models::Vertex {
            position: vertices[3].0,
            uv: vertices[3].1,
            color: vertices[3].2.into(),
            normal: n
        }
    ];

    context.draw_mode(DrawMode::Triangles);
    context.geometry(&quad[..], &indices);
}

/// Draws a single quad with an optional texture, color, and uv scale.
pub fn draw_quad_3d_ex(v1: Vec3, v2: Vec3, v3: Vec3, v4: Vec3, texture: Option<&Texture2D>, color: Color, uv_scale: f32) {

    unsafe {

        {
            let context = get_internal_gl().quad_gl;
            context.texture(texture);
        }

        draw_quad(
            [
                (v1, vec2(0.0, 0.0), color),
                (v2, vec2(uv_scale, 0.0), color),
                (v3, vec2(uv_scale, uv_scale), color),
                (v4, vec2(0.0, uv_scale), color)
            ]
        );

    }

}

/// Draws a single quad with an optional texture, color.
pub fn draw_quad_3d(v1: Vec3, v2: Vec3, v3: Vec3, v4: Vec3, texture: Option<&Texture2D>, color: Color) {

    let default_uv_scale = 1.0;
    draw_quad_3d_ex(v1, v2, v3, v4, texture, color, default_uv_scale);

}

/// Draws quads using the vertices array passed in (in chunks of 4), the number of vertices must be divisible by 4 or else will panic with an assert.
pub fn draw_quads_3d(vertices: &[Vec3], texture: Option<&Texture2D>, color: Color) {

    assert!(vertices.len() % 4 == 0);

    for vtx in vertices.chunks(4) {
        draw_quad_3d(vtx[0], vtx[1], vtx[2], vtx[3], texture, color);
    }

}

/// Draws a cube with rotation.
pub fn draw_cube_ex(position: Vec3, rotation: Quat, size: Vec3, texture: Option<&Texture2D>, color: Color) {

    unsafe {

        let context = get_internal_gl().quad_gl;

        // because we're applying the rotation and translation here now, use x, y, z as if the cube was at the origin now.
        context.push_model_matrix(Mat4::from_rotation_translation(rotation, position));

    }
    
    draw_cube(vec3(0.0, 0.0, 0.0), size, texture, color);

    unsafe {
        let context = get_internal_gl().quad_gl;
        context.pop_model_matrix();
    }

}

/// Draws a wireframe cube with rotation.
pub fn draw_cube_wires_ex(position: Vec3, rotation: Quat, size: Vec3, color: Color) {

    unsafe {

        let context = get_internal_gl().quad_gl;

        // because we're applying the rotation and translation here now, use x, y, z as if the cube was at the origin now.
        context.push_model_matrix(Mat4::from_rotation_translation(rotation, position));

    }
    
    draw_cube_wires(vec3(0.0, 0.0, 0.0), size, color);

    unsafe {
        let context = get_internal_gl().quad_gl;
        context.pop_model_matrix();
    }

}

/// Draws a sphere with a specific rotation, so that you can nicely render rolling balls and similar things.
pub fn draw_sphere_ex_with_rotation(center: Vec3, rotation: Quat, radius: f32, texture: Option<Texture2D>, color: Color, params: DrawSphereParams) {

    unsafe {

        let context = get_internal_gl().quad_gl;
        context.texture(None);

        // because we're applying the rotation and translation here now, use x, y, z as if the sphere was at the origin now.
        context.push_model_matrix(Mat4::from_rotation_translation(rotation, center));

    }

    draw_sphere_ex(vec3(0.0, 0.0, 0.0), radius, texture.as_ref(), color, params);

    unsafe {
        let context = get_internal_gl().quad_gl;
        context.pop_model_matrix();
    }

}

/// Draws a wireframe sphere with a specific rotation, so that you can nicely render rolling balls and similar things.
pub fn draw_sphere_wires_ex(center: Vec3, rotation: Quat, radius: f32, color: Color, params: DrawSphereParams) {

    unsafe {

        let context = get_internal_gl().quad_gl;
        context.texture(None);

        // because we're applying the rotation and translation here now, use x, y, z as if the sphere was at the origin now.
        context.push_model_matrix(Mat4::from_rotation_translation(rotation, center));

    }

    draw_sphere_ex(vec3(0.0, 0.0, 0.0), radius, None, color, DrawSphereParams { rings: params.rings, slices: params.slices, draw_mode: DrawMode::Lines });

    unsafe {
        let context = get_internal_gl().quad_gl;
        context.pop_model_matrix();
    }

}

/// Draws a textured plane at a given position with a given size and desired (uniform) uv scale.
pub fn draw_plane_ex(center: Vec3, size: Vec2, texture: Option<&Texture2D>, color: Color, uv_scale: f32) {

    let half_x = size.x / 2.0;
    let half_y = size.y / 2.0;

    let a = center + vec3(-half_x, 0.0, -half_y);
    let b = center + vec3(half_x, 0.0, -half_y);
    let c = center + vec3(half_x, 0.0, half_y);
    let d = center + vec3(-half_x, 0.0, half_y);

    draw_quad_3d_ex(a, b, c, d, texture, color, uv_scale);

}

/// Draws a grid at a given position, with the given rotation.
pub fn draw_grid_ex(center: Vec3, rotation: Quat, slices: u32, spacing: f32, axes_color: Color, other_color: Color) {

    unsafe {

        let context = get_internal_gl().quad_gl;
        context.texture(None);

        // because we're applying the rotation and translation here now, use x, y, z as if the sphere was at the origin now.
        context.push_model_matrix(Mat4::from_rotation_translation(rotation, center));

    }

    draw_grid(slices, spacing, axes_color, other_color);

    unsafe {
        let context = get_internal_gl().quad_gl;
        context.pop_model_matrix();
    }

}

/// Pushes a model matrix with the given translation and rotation before calling the drawing function, then pops the model matrix.
pub fn draw_with_transformation<F>(position: Vec3, rotation: Quat, drawing_fn: F)
    where F: Fn()
{

    unsafe {
        let context = get_internal_gl().quad_gl;
        context.push_model_matrix(Mat4::from_rotation_translation(rotation, position));
    }

    drawing_fn();

    unsafe {
        let context = get_internal_gl().quad_gl;
        context.pop_model_matrix();
    }
    
}

/// Draws a circle in the XZ plane, assuming Y+ up.
pub fn draw_circle_lines_3d(position: Vec3, r: f32, _thickness: f32, color: Color) {

    let sides = 20;

    for i in 0..sides {

        let current_angle = ((2.0*PI) / sides as f32) * i as f32;
        let next_angle = (((2.0*PI) / sides as f32) * (i + 1) as f32) % (2.0 * PI);

        let current_start = position.xz() + position.xz().normalize_or_zero().rotated_by(current_angle) * r;
        let current_end = position.xz() + position.xz().normalize_or_zero().rotated_by(next_angle) * r;
        
        draw_line_3d(WithY::with_y(&current_start, position.y), WithY::with_y(&current_end, position.y), color);

    }
    
}