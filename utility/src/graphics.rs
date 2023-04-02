use macroquad::prelude::*;

unsafe fn draw_quad(vertices: [(Vec3, Vec2, Color); 4]) {
    let context = get_internal_gl().quad_gl;
    let indices = [0, 1, 2, 0, 2, 3];
    let quad = [
        (
            [vertices[0].0.x, vertices[0].0.y, vertices[0].0.z],
            [vertices[0].1.x, vertices[0].1.y],
            vertices[0].2.into(),
        ),
        (
            [vertices[1].0.x, vertices[1].0.y, vertices[1].0.z],
            [vertices[1].1.x, vertices[1].1.y],
            vertices[1].2.into(),
        ),
        (
            [vertices[2].0.x, vertices[2].0.y, vertices[2].0.z],
            [vertices[2].1.x, vertices[2].1.y],
            vertices[2].2.into(),
        ),
        (
            [vertices[3].0.x, vertices[3].0.y, vertices[3].0.z],
            [vertices[3].1.x, vertices[3].1.y],
            vertices[3].2.into(),
        ),
    ];

    context.draw_mode(DrawMode::Triangles);
    context.geometry(&quad[..], &indices);
}

pub fn draw_quad_3d_ex(v1: Vec3, v2: Vec3, v3: Vec3, v4: Vec3, texture: Option<Texture2D>, color: Color, uv_scale: f32) {

    unsafe {

        {
            let context = get_internal_gl().quad_gl;
            context.texture(texture.into());
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

pub fn draw_quad_3d(v1: Vec3, v2: Vec3, v3: Vec3, v4: Vec3, texture: Option<Texture2D>, color: Color) {

    let default_uv_scale = 1.0;
    draw_quad_3d_ex(v1, v2, v3, v4, texture, color, default_uv_scale);

}

pub fn draw_quads_3d(vertices: &[Vec3], texture: Option<Texture2D>, color: Color) {

    assert!(vertices.len() % 4 == 0);

    for vtx in vertices.chunks(4) {
        draw_quad_3d(vtx[0], vtx[1], vtx[2], vtx[3], texture, color);
    }

}

pub fn draw_cube_ex(position: Vec3, rotation: Quat, size: Vec3, texture: Option<Texture2D>, color: Color) {

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

pub fn draw_sphere_ex_with_rotation(center: Vec3, rotation: Quat, radius: f32, texture: Option<Texture2D>, color: Color, params: DrawSphereParams) {

    unsafe {

        let context = get_internal_gl().quad_gl;
        context.texture(None);

        // because we're applying the rotation and translation here now, use x, y, z as if the sphere was at the origin now.
        context.push_model_matrix(Mat4::from_rotation_translation(rotation, center));

    }

    draw_sphere_ex(vec3(0.0, 0.0, 0.0), radius, texture, color, params);

    unsafe {
        let context = get_internal_gl().quad_gl;
        context.pop_model_matrix();
    }

}

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

pub fn draw_plane_ex(center: Vec3, size: Vec2, texture: Option<Texture2D>, color: Color, uv_scale: f32) {

    let half_x = size.x / 2.0;
    let half_y = size.y / 2.0;

    let a = center + vec3(-half_x, 0.0, -half_y);
    let b = center + vec3(half_x, 0.0, -half_y);
    let c = center + vec3(half_x, 0.0, half_y);
    let d = center + vec3(-half_x, 0.0, half_y);

    draw_quad_3d_ex(a, b, c, d, texture, color, uv_scale);

}

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