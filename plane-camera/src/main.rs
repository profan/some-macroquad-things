use std::cmp;

use macroquad::prelude::*;
use utility::WithAlpha;

const ORIGIN: Vec3 = Vec3::ZERO;

const UP: Vec3 = vec3(0.0, 1.0, 0.0);
const FORWARD: Vec3 = vec3(0.0, 0.0, -1.0);
const LEFT: Vec3 = vec3(-1.0, 0.0, 0.0);

struct Plane {
    position: Vec3,
    normal: Vec3
}

fn draw_plane_wires_3d(position: Vec3, normal: Vec3, size: Vec2, color: Color) {

    let normal_in_plane = normal.any_orthonormal_vector();
    let fwd_in_plane = normal.cross(normal_in_plane);

    // now build this matrix manually because we know de wae
    let transform = Affine3A::from_cols(
        normal_in_plane.into(),
        normal.into(),
        fwd_in_plane.into(),
        position.into()
    );

    let top_left = transform.transform_point3(vec3(0.0, 0.0, 0.0));
    let top_right = transform.transform_point3(vec3(size.x, 0.0, 0.0));
    let bottom_left = transform.transform_point3(vec3(0.0, 0.0, size.y));
    let bottom_right = transform.transform_point3(vec3(size.x, 0.0, size.y));

    // draw the lines of the edges of the plane
    draw_line_3d(top_left, top_right, color);
    draw_line_3d(top_right, bottom_right, color);
    draw_line_3d(bottom_right, bottom_left, color);
    draw_line_3d(bottom_left, top_left, color);

    // draw normal
    draw_line_3d(position, position + normal, RED);

    // draw the orthogonal vector in green
    draw_line_3d(position, position + normal_in_plane, GREEN);

}

fn handle_camera_input(camera: &mut Camera3D, dt: f32) {

    let is_up_down = is_key_down(KeyCode::W) || is_key_down(KeyCode::Up);
    let is_down_down = is_key_down(KeyCode::S) || is_key_down(KeyCode::Down);
    let is_left_down = is_key_down(KeyCode::A) || is_key_down(KeyCode::Left);
    let is_right_down = is_key_down(KeyCode::D) || is_key_down(KeyCode::Right);
    let is_sprint_down = is_key_down(KeyCode::LeftShift);

    let mut camera_delta = Vec3::ZERO;
    let camera_movement_speed = 4.0;

    let camera_fwd = (camera.target - camera.position).normalize();
    let camera_left = camera.up.cross(camera_fwd);
    
    if is_up_down {
        camera_delta += camera_fwd;
    }
    
    if is_down_down {
        camera_delta -= camera_fwd;
    }

    if is_left_down {
        camera_delta += camera_left;
    }

    if is_right_down {
        camera_delta -= camera_left;
    }

    if is_sprint_down {
        camera_delta *= 2.0;
    }

    camera.position += camera_delta * camera_movement_speed * dt;
    camera.target += camera_delta * camera_movement_speed * dt;

}

fn calculate_current_camera_up_vector(camera: &Camera3D, planes: &[Plane], min_distance: f32) -> Vec3 {

    let mut total_planes = 0.0;
    let mut total_sum = Vec3::ZERO;

    // sort the planes by distance
    // planes.sort_by(|p1, p2| p1.position.distance(camera.position).total_cmp(&p2.position.distance(camera.position)));

    // for p in planes {

    //     let distance_to_plane = camera.position.distance(p.position);
    //     let contribution_factor = 1.0 - utility::normalize(distance_to_plane, 0.0, 1.0, min_distance);

    //     if distance_to_plane < min_distance {
    //         total_sum += p.normal * contribution_factor;
    //         total_planes += 1.0;
    //     }

    // }

    // if total_planes > 0.0 {
    //     let average_normal = total_sum / total_planes;
    //     average_normal
    // } else {
    //     camera.up
    // }

    // find min plane
    let min_plane = planes.into_iter().min_by(|p1, p2| p1.position.distance(camera.position).total_cmp(&p2.position.distance(camera.position)));

    min_plane.up


}

fn draw_debug_text(camera: &Camera3D) {

    draw_text(
        format!("camera up: {}", camera.up).as_str(),
        32.0, 32.0,
        16.0, BLACK
    );

}

#[macroquad::main("plane-camera")]
async fn main() {

    let mut camera = Camera3D {
        position: vec3(-20.0, 15.0, 0.0),
        up: vec3(0.0, 1.0, 0.0),
        target: vec3(0.0, 0.0, 0.0),
        ..Default::default()
    };

    let planes = [
        Plane {
            position: ORIGIN,
            normal: UP
        },
        Plane {
            position: ORIGIN + LEFT * 5.0,
            normal: FORWARD.lerp(LEFT, 0.5)
        },
    ];
    
    loop {
         
        let dt = get_frame_time();

        clear_background(WHITE);

        handle_camera_input(&mut camera, dt);

        if is_key_pressed(KeyCode::Escape) {
            break;
        }
    
        // recalculate camera up if necessary
        let last_camera_up = camera.up;
        camera.up = calculate_current_camera_up_vector(&camera, &planes, 30.0);
        let arc_from_last_camera_up = Quat::from_rotation_arc(last_camera_up, camera.up);
        camera.target = arc_from_last_camera_up.mul_vec3(camera.target);

        // use 3d camera
        set_camera(&camera);

        for p in &planes {

            let plane_size = vec2(5.0, 5.0);

            draw_plane_wires_3d(
                p.position,
                p.normal,
                plane_size,
                BLACK.with_alpha(0.5)
            );

        }

        // draw_grid(20, 1., BLACK, GRAY);

        // draw_cube_wires(vec3(0., 1., -6.), vec3(2., 2., 2.), DARKGREEN);
        // draw_cube_wires(vec3(0., 1., 6.), vec3(2., 2., 2.), DARKBLUE);
        // draw_cube_wires(vec3(2., 1., 2.), vec3(2., 2., 2.), YELLOW);

        // draw_plane(vec3(-8., 0., -8.), vec2(5., 5.), None, WHITE);

        // draw_cube(vec3(-5., 1., -2.), vec3(2., 2., 2.), None, WHITE);
        // draw_cube(vec3(-5., 1., 2.), vec3(2., 2., 2.), None, WHITE);
        // draw_cube(vec3(2., 0., -2.), vec3(0.4, 0.4, 0.4), None, BLACK);

        // screen space
        set_default_camera();

        draw_debug_text(&camera);

        next_frame().await;

    }

}