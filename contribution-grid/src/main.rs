use std::f32::consts::PI;

use macroquad::{prelude::*, rand::gen_range};
use utility::{DebugText, TextPosition, RotatedBy, GameCamera, GameCameraParameters, create_camera_from_game_camera};

const WORLD_UP: Vec3 = vec3(0.0, 1.0, 0.0);

/// world size in units
const WORLD_SIZE: i32 = 100;

/// the size of each square in the world grid
const WORLD_GRID_SIZE: i32 = 5;

const ORB_SIZE: f32 = 2.0;

#[derive(Clone, Copy)]
struct Triangle {
    a: Vec3,
    b: Vec3,
    c: Vec3
}

#[derive(Clone, Copy)]
struct Orb {
    position: Vec3,
    normal: Vec3
}

struct Game {
    camera: GameCamera,
    debug_text: DebugText,
    world_bounds: (Vec3, Vec3),
    orbs: Vec<Orb>
}

impl Game {
    pub fn new() -> Game {
        Game {
            camera: GameCamera {

                parameters: GameCameraParameters {
                    movement_speed: 16.0,
                    rotation_speed: PI/2.0,
                    zoom_speed: 1.0
                },

                position: Vec3::ZERO,
                up: WORLD_UP,
                target: Vec3::ZERO,

            },
            world_bounds: (Vec3::ZERO, Vec3::ZERO),
            debug_text: DebugText::new(),
            orbs: Vec::new()
        }
    }
}

fn calculate_contribution(p: Vec3, t: Triangle) -> Vec3 {

    let d_a = p.distance(t.a);
    let d_b = p.distance(t.b);
    let d_c = p.distance(t.c);

    let total = d_a + d_b + d_c;

    let a = d_a / total;
    let b = d_b / total;
    let c = d_c / total;

    vec3(
        ((b + c) - a).max(0.0),
        ((a + c) - b).max(0.0),
        ((a + b) - c).max(0.0)
    )

}

fn handle_camera_input(active: &mut GameCamera, last_mouse_position: Vec2, dt: f32) {

    let is_forwards_pressed = is_key_down(KeyCode::W);
    let is_backwards_pressed = is_key_down(KeyCode::S);
    let is_left_pressed = is_key_down(KeyCode::A);
    let is_right_pressed = is_key_down(KeyCode::D);

    let is_up_pressed = is_key_down(KeyCode::Space);
    let is_down_pressed = is_key_down(KeyCode::LeftControl);

    let mut camera_movement_delta = Vec3::ZERO;

    let forward_in_plane = vec3(active.forward().x, 0.0, active.forward().z);
    let left_in_plane = vec3(active.left().x, 0.0, active.left().z);
    let up_in_plane = WORLD_UP;

    if is_forwards_pressed {
        camera_movement_delta += forward_in_plane * active.parameters.movement_speed * dt;
    }

    if is_backwards_pressed {
        camera_movement_delta -= forward_in_plane * active.parameters.movement_speed * dt;
    }

    if is_left_pressed {
        camera_movement_delta += left_in_plane * active.parameters.movement_speed * dt;
    }

    if is_right_pressed {
        camera_movement_delta -= left_in_plane * active.parameters.movement_speed * dt;
    }

    if is_up_pressed {
        camera_movement_delta += up_in_plane * active.parameters.movement_speed * dt;
    }

    if is_down_pressed {
        camera_movement_delta -= up_in_plane * active.parameters.movement_speed * dt;
    }

    active.position += camera_movement_delta;
    active.target += camera_movement_delta;

}

fn draw_orbs(game: &Game) {

    for orb in &game.orbs {

        draw_cube_wires(
            orb.position,
            vec3(ORB_SIZE, ORB_SIZE, ORB_SIZE),
            RED
        );

    }

}

fn find_three_closest_orbs(game: &Game, position: Vec3) -> [Orb; 3] {

    let mut result: [Orb; 3] = [game.orbs[0], game.orbs[1], game.orbs[2]];
    let mut values: [f32; 3] = [f32::MAX, f32::MAX, f32::MAX];

    for &orb in &game.orbs {

        let d = orb.position.distance(position);

        if d < values[0] {

            values[2] = values[1];
            result[2] = result[1];

            values[1] = values[0];
            result[1] = result[0];

            values[0] = d;
            result[0] = orb;

        } else if d < values[1] {

            result[2] = result[1];
            values[2] = values[1];

            result[1] = orb;
            values[1] = d;

        } else if d < values[2] {

            result[2] = orb;
            values[2] = d;

        }

    }

    result

}

fn find_current_average_for_position(game: &Game, position: Vec3) -> Vec3 {

    let should_draw_debug_triangles = false;
    let orbs = find_three_closest_orbs(game, position);

    let orb_a_position = orbs[0].position;
    let orb_b_position = orbs[1].position;
    let orb_c_position = orbs[2].position;

    if should_draw_debug_triangles {

        draw_line_3d(orb_a_position, orb_b_position, PURPLE);
        draw_line_3d(orb_b_position, orb_c_position, PURPLE);
        draw_line_3d(orb_c_position, orb_a_position, PURPLE);
        
    }

    let triangle = Triangle { a: orb_a_position, b: orb_b_position, c: orb_c_position };
    let contribution = calculate_contribution(position, triangle);

    let result =
        orb_a_position * contribution.x +
        orb_b_position * contribution.y +
        orb_c_position * contribution.z;

    result

}

fn draw_world_grid(game: &Game) {

    let world_offset = -vec3(WORLD_SIZE as f32 / 2.0, 0.0, WORLD_SIZE as f32 / 2.0);

    for x_i in 0 .. (WORLD_SIZE / WORLD_GRID_SIZE) {
        for z_i in 0 .. (WORLD_SIZE / WORLD_GRID_SIZE) {

            let x_a = (x_i * WORLD_GRID_SIZE) as f32;
            let z_a = (z_i * WORLD_GRID_SIZE) as f32;
            let y_a = find_current_average_for_position(game, vec3(x_a, 0.0, z_a) + world_offset).y;

            let x_b = (x_i * WORLD_GRID_SIZE) as f32;
            let z_b = ((z_i + 1) * WORLD_GRID_SIZE) as f32;
            let y_b = find_current_average_for_position(game, vec3(x_b, 0.0, z_b) + world_offset).y;

            let x_c = ((x_i + 1) * WORLD_GRID_SIZE) as f32;
            let z_c = ((z_i + 1) * WORLD_GRID_SIZE) as f32;
            let y_c = find_current_average_for_position(game, vec3(x_c, 0.0, z_c) + world_offset).y;

            let x_d = ((x_i + 1) * WORLD_GRID_SIZE ) as f32;
            let z_d = (z_i * WORLD_GRID_SIZE) as f32;
            let y_d = find_current_average_for_position(game, vec3(x_d, 0.0, z_d) + world_offset).y;

            draw_line_3d(
                vec3(x_a, y_a, z_a) + world_offset,
                vec3(x_b, y_b, z_b) + world_offset,
                GREEN
            );

            draw_line_3d(
                vec3(x_b, y_b, z_b) + world_offset,
                vec3(x_c, y_c, z_c) + world_offset,
                GREEN
            );

            draw_line_3d(
                vec3(x_c, y_c, z_c) + world_offset,
                vec3(x_d, y_d, z_d) + world_offset,
                GREEN
            );

            draw_line_3d(
                vec3(x_d, y_d, z_d) + world_offset,
                vec3(x_a, y_a, z_a) + world_offset,
                GREEN
            );
            
        }
    }

}

fn draw_debug_ui(game: &mut Game) {

    game.debug_text.draw_text(format!("camera position: {}", game.camera.position), TextPosition::TopLeft, BLACK);
    game.debug_text.draw_text("w/a/s/d to move the camera in the plane", TextPosition::TopLeft, BLACK);
    game.debug_text.draw_text("space/ctrl to move the camera up/down", TextPosition::TopLeft, BLACK);
    game.debug_text.draw_text("alt to toggle rotation", TextPosition::TopLeft, BLACK);
    game.debug_text.draw_text("press r to regenerate", TextPosition::TopLeft, BLACK);

}

fn add_initial_orbs(game: &mut Game, number_of_orbs: i32) {

    let (start, end) = game.world_bounds;

    for i in 0..number_of_orbs {
        
        let r_x = gen_range(start.x, end.x);
        let r_z = gen_range(start.z, end.z);

        // height!!!
        let r_y = gen_range(0.0, 24.0);
    
        game.orbs.push(
            Orb {
                position: vec3(r_x, r_y, r_z),
                normal: WORLD_UP
            }
        )
    }

}

fn add_initial_orbs_for_triangle_scene(game: &mut Game) {

    // center

    game.orbs.push(
        Orb {
            position: vec3(4.0, -20.0, 0.0),
            normal: WORLD_UP
        },
    );

    game.orbs.push(
        Orb {
            position: vec3(-4.0, -20.0, 0.0),
            normal: WORLD_UP
        },
    );

    game.orbs.push(
        Orb {
            position: vec3(0.0, -20.0, 4.0),
            normal: WORLD_UP
        },
    );

    // corners

    game.orbs.push(
        Orb {
            position: vec3(50.0, 0.0, -50.0),
            normal: WORLD_UP
        },
    );

    game.orbs.push(
        Orb {
            position: vec3(-50.0, 0.0, -50.0),
            normal: WORLD_UP
        },
    );

    game.orbs.push(
        Orb {
            position: vec3(0.0, 0.0, 50.0),
            normal: WORLD_UP
        },
    );

}

fn rotate_orbs_around_center_of_grid(game: &mut Game, dt: f32) {

    let (grid_start, grid_end) = game.world_bounds;
    let center_of_grid = (grid_start + grid_end) / 2.0;
    let radians_per_second = PI/4.0;

    for orb in &mut game.orbs {

        let rotation_delta = radians_per_second * dt;
        let rotated_position_in_plane = orb.position.xz().rotated_by_around_origin(rotation_delta, center_of_grid.xz());
        
        orb.position.x = rotated_position_in_plane.x;
        orb.position.z = rotated_position_in_plane.y;

    }

}

#[macroquad::main("contribution-grid")]
async fn main() {

    let mut game = Game::new();
    let mut last_mouse_position: Vec2 = mouse_position().into();

    let mut should_generate_the_world = true;
    let mut should_be_rotating_the_grid = false;

    // set initial world bounds
    let world_bounds_half = vec3(WORLD_SIZE as f32 / 2.0, 0.0, WORLD_SIZE as f32 / 2.0);
    game.world_bounds = (-world_bounds_half, world_bounds_half);

    // set initial camera target and position
    let initial_camera_position = vec3(0.0, 56.0, 56.0);

    game.camera.position += vec3(0.0, 8.0, 16.0) + initial_camera_position;
    game.camera.target += vec3(0.0, 0.0, 8.0) + initial_camera_position;
    
    loop {

        let was_reset_key_pressed = is_key_pressed(KeyCode::R);
        if was_reset_key_pressed {
            should_generate_the_world = true;
        }

        let was_toggle_grid_rotation_pressed = is_key_pressed(KeyCode::LeftAlt);
        if was_toggle_grid_rotation_pressed {
            should_be_rotating_the_grid = !should_be_rotating_the_grid;
        }

        if should_generate_the_world {

            // clear existing worlds
            game.orbs.clear();

            // cweate the initial set of owbs to pwace
            let number_of_orbs = 12;

            add_initial_orbs(&mut game, number_of_orbs);
            // add_initial_orbs_for_triangle_scene(&mut game);

            // unset flag
            should_generate_the_world = false;

        }

        let dt = get_frame_time();
        game.debug_text.new_frame();

        set_camera(&create_camera_from_game_camera(&game.camera));
        clear_background(WHITE);

        handle_camera_input(&mut game.camera, last_mouse_position, dt);
        last_mouse_position = mouse_position().into();

        // update scene
        if should_be_rotating_the_grid {
            rotate_orbs_around_center_of_grid(&mut game, dt);
        }

        // draw scene, etc
        draw_world_grid(&game);
        draw_orbs(&game);

        // draw ui
        set_default_camera();
        draw_debug_ui(&mut game);

        next_frame().await;

    }

}