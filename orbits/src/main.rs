use core::num;
use std::f32::consts::PI;

use macroquad::prelude::*;
use utility::{GameCamera, create_camera_from_game_camera, AsAngle, DebugText, draw_circle_lines_3d, draw_with_transformation, TextPosition, WithAlpha, WithY};

const WORLD_UP: Vec3 = Vec3::Y;
const PLANET_RADIUS: f32 = 4.0;
const MOON_SIZE: f32 = 0.5;

struct Moon {
    position: Vec3,
    rotation: Vec3
}

struct Planet {
    position: Vec3,
    moons: Vec<Moon>
}

fn create_moon(position: Vec3) -> Moon {
    let rotation = infer_orbital_plane(position);
    Moon { position, rotation }
}

fn random_position_in_sphere(radius: f32) -> Vec3 {

    let allow_negative = true;
    let min = if allow_negative { -1.0 } else { 0.1 };
    let max = 1.0;

    let x = rand::gen_range(min, max);
    let y = rand::gen_range(min, max);
    let z = rand::gen_range(min, max);

    Vec3 { x, y, z }.normalize() * radius
}

fn create_planet(position: Vec3, min_number_of_moons: i32, max_number_of_moons: i32) -> Planet {

    let mut moons = Vec::new();
    let number_of_moons = rand::gen_range(min_number_of_moons, max_number_of_moons);

    for _ in 0..number_of_moons {
        let position = random_position_in_sphere(PLANET_RADIUS * 2.0);
        moons.push(create_moon(position));
    }

    Planet { position, moons }

}

struct Game {
    camera: GameCamera,
    debug: DebugText,
    planets: Vec<Planet>
}

impl Game {

    fn new() -> Game {
        Game {
            camera: GameCamera::new(),
            debug: DebugText::new(),
            planets: Vec::new()
        }
    }

}

fn setup_camera_for_game(game: &mut Game) {

    // set initial camera target and position
    let initial_camera_position = vec3(0.0, 32.0, 32.0);

    game.camera.position += vec3(0.0, 8.0, 16.0) + initial_camera_position;
    game.camera.target += vec3(0.0, 0.0, 8.0) + initial_camera_position;

    // fix up movement speed
    game.camera.parameters.movement_speed = 16.0;
    game.camera.parameters.rotation_speed = PI / 2.0;

}

fn create_game() -> Game {

    let mut game = Game::new();
    setup_camera_for_game(&mut game);

    let min_moons = 4;
    let max_moons = 4;

    let main_planet = create_planet(vec3(1.0, 0.0, 0.0), min_moons, max_moons);

    // add some planets
    game.planets.push(main_planet);

    game

}

fn draw_orbital_plane(position: Vec3, angles: Vec3, radius: f32) {

    let line_thickness = 1.0;
    let orbit_rotation = Quat::from_euler(EulerRot::ZXY, angles.x, angles.y, angles.z);

    draw_with_transformation(Vec3::ZERO, orbit_rotation.inverse(), || {
        draw_circle_lines_3d(position, radius, line_thickness, RED);
    });

}

fn update_game(game: &mut Game, dt: f32) {

    for planet in &mut game.planets {
        update_planet(planet, dt);
    }

}

fn update_planet(planet: &mut Planet, dt: f32) {

    for moon in &mut planet.moons {
        update_moon(moon, dt);
    }

}

fn update_moon(moon: &mut Moon, dt: f32) {

}

fn draw_planet(planet: &Planet) {

    draw_sphere_wires(planet.position, PLANET_RADIUS, None, BLACK);

    for moon in &planet.moons {
        draw_moon(planet, moon);
    }

}

fn draw_moon(planet: &Planet, moon: &Moon) {

    let world_position_of_moon = planet.position + moon.position;
    draw_cube_wires(world_position_of_moon, vec3(MOON_SIZE, MOON_SIZE, MOON_SIZE), BLACK);

    let orbital_plane_radius = world_position_of_moon.length();
    draw_orbital_plane(planet.position, moon.rotation, orbital_plane_radius);

}

fn draw_game(game: &Game) {

    draw_grid(16, 4.0, RED.with_alpha(0.75), BLACK.with_alpha(0.75));

    for planet in &game.planets {
        draw_planet(planet);
    }

}

fn draw_debug_ui(game: &mut Game) {

    for planet in &game.planets {
        
        game.debug.draw_text(format!("planet - {}", planet.position), TextPosition::TopLeft, BLACK);

        for moon in &planet.moons {
            game.debug.draw_text(format!(" - moon - {} - {}", moon.position, moon.rotation), TextPosition::TopLeft, BLACK);
        }

    }

}

/// Given a position in local space of another body, returns the a set of angles in radians representing the orbital plane the body is in, it assumes no roll.
fn infer_orbital_plane(position: Vec3) -> Vec3 {

    let hyp = position.length();
    let opp = position.y.abs();

    let sin_theta = opp / hyp;

    let dir_to_plane_intersection = position.with_y(0.0).normalize();
    let dir_to_position = position.normalize();

    let pitch = -(position.normalize().y * sin_theta.asin());

    let yaw = position.xz().as_angle() + PI / 2.0;
    let roll = 0.0;

    vec3(pitch, yaw, roll)

}

fn handle_camera_input(active: &mut GameCamera, dt: f32) {

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


#[macroquad::main("orbits")]
async fn main() {

    let mut game = create_game();

    loop {

        if is_key_released(KeyCode::R) {
            let old_game = std::mem::replace(&mut game, create_game());
            game.camera = old_game.camera;
        }

        let dt = get_frame_time();
        update_game(&mut game, dt);

        game.debug.new_frame();
        clear_background(WHITE);

        handle_camera_input(&mut game.camera, dt);

        set_camera(&create_camera_from_game_camera(&game.camera));
        draw_game(&game);

        set_default_camera();
        draw_debug_ui(&mut game);

        next_frame().await;

    }
   
}
