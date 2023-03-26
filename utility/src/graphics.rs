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

pub fn draw_cube_ex(position: Vec3, size: Vec3, rotation: Quat, texture: Option<Texture2D>, color: Color) {

    unsafe {

        let context = get_internal_gl().quad_gl;
        context.texture(texture.into());

        // because we're applying the rotation and translation here now, use x, y, z as if the cube was at the origin now.
        context.push_model_matrix(Mat4::from_rotation_translation(rotation, position));

    }

    let (x, y, z) = (0.0, 0.0, 0.0);
    let (width, height, length) = (size.x, size.y, size.z);

    // Front face
    let bl_pos = vec3(x - width / 2., y - height / 2., z + length / 2.);
    let bl_uv = vec2(0., 0.);
    let br_pos = vec3(x + width / 2., y - height / 2., z + length / 2.);
    let br_uv = vec2(1., 0.);

    let tr_pos = vec3(x + width / 2., y + height / 2., z + length / 2.);
    let tr_uv = vec2(1., 1.);

    let tl_pos = vec3(x - width / 2., y + height / 2., z + length / 2.);
    let tl_uv = vec2(0., 1.);

    unsafe {
        draw_quad([
            (bl_pos, bl_uv, color),
            (br_pos, br_uv, color),
            (tr_pos, tr_uv, color),
            (tl_pos, tl_uv, color),
        ]);
    }

    // Back face
    let bl_pos = vec3(x - width / 2., y - height / 2., z - length / 2.);
    let bl_uv = vec2(0., 0.);
    let br_pos = vec3(x + width / 2., y - height / 2., z - length / 2.);
    let br_uv = vec2(1., 0.);

    let tr_pos = vec3(x + width / 2., y + height / 2., z - length / 2.);
    let tr_uv = vec2(1., 1.);

    let tl_pos = vec3(x - width / 2., y + height / 2., z - length / 2.);
    let tl_uv = vec2(0., 1.);

    unsafe {
        draw_quad([
            (bl_pos, bl_uv, color),
            (br_pos, br_uv, color),
            (tr_pos, tr_uv, color),
            (tl_pos, tl_uv, color),
        ]);
    }

    // Top face
    let bl_pos = vec3(x - width / 2., y + height / 2., z - length / 2.);
    let bl_uv = vec2(0., 1.);
    let br_pos = vec3(x - width / 2., y + height / 2., z + length / 2.);
    let br_uv = vec2(0., 0.);

    let tr_pos = vec3(x + width / 2., y + height / 2., z + length / 2.);
    let tr_uv = vec2(1., 0.);

    let tl_pos = vec3(x + width / 2., y + height / 2., z - length / 2.);
    let tl_uv = vec2(1., 1.);

    unsafe {
        draw_quad([
            (bl_pos, bl_uv, color),
            (br_pos, br_uv, color),
            (tr_pos, tr_uv, color),
            (tl_pos, tl_uv, color),
        ]);
    }

    // Bottom face
    let bl_pos = vec3(x - width / 2., y - height / 2., z - length / 2.);
    let bl_uv = vec2(0., 1.);
    let br_pos = vec3(x - width / 2., y - height / 2., z + length / 2.);
    let br_uv = vec2(0., 0.);

    let tr_pos = vec3(x + width / 2., y - height / 2., z + length / 2.);
    let tr_uv = vec2(1., 0.);

    let tl_pos = vec3(x + width / 2., y - height / 2., z - length / 2.);
    let tl_uv = vec2(1., 1.);

    unsafe {
        draw_quad([
            (bl_pos, bl_uv, color),
            (br_pos, br_uv, color),
            (tr_pos, tr_uv, color),
            (tl_pos, tl_uv, color),
        ]);
    }

    // Right face
    let bl_pos = vec3(x + width / 2., y - height / 2., z - length / 2.);
    let bl_uv = vec2(0., 1.);
    let br_pos = vec3(x + width / 2., y + height / 2., z - length / 2.);
    let br_uv = vec2(0., 0.);

    let tr_pos = vec3(x + width / 2., y + height / 2., z + length / 2.);
    let tr_uv = vec2(1., 0.);

    let tl_pos = vec3(x + width / 2., y - height / 2., z + length / 2.);
    let tl_uv = vec2(1., 1.);

    unsafe {
        draw_quad([
            (bl_pos, bl_uv, color),
            (br_pos, br_uv, color),
            (tr_pos, tr_uv, color),
            (tl_pos, tl_uv, color),
        ]);
    }

    // Left face
    let bl_pos = vec3(x - width / 2., y - height / 2., z - length / 2.);
    let bl_uv = vec2(0., 1.);
    let br_pos = vec3(x - width / 2., y + height / 2., z - length / 2.);
    let br_uv = vec2(0., 0.);

    let tr_pos = vec3(x - width / 2., y + height / 2., z + length / 2.);
    let tr_uv = vec2(1., 0.);

    let tl_pos = vec3(x - width / 2., y - height / 2., z + length / 2.);
    let tl_uv = vec2(1., 1.);

    unsafe {
        draw_quad([
            (bl_pos, bl_uv, color),
            (br_pos, br_uv, color),
            (tr_pos, tr_uv, color),
            (tl_pos, tl_uv, color),
        ]);
    }

    unsafe {
        let context = get_internal_gl().quad_gl;
        context.pop_model_matrix();
    }

}