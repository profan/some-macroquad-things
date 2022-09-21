#![feature(let_chains)]
use std::{path::PathBuf, fs::read_to_string};
use macroquad::prelude::*;

const TILE_SIZE: f32 = 32.0;

enum TileKind {
    Air,
    Wall,
    Floor,
    Any
}

struct Tile {
    tile_position: Vec2,
    kind: TileKind
}

impl Tile {

    fn local_position(&self) -> Vec2 {
        self.tile_position * TILE_SIZE
    }

    fn world_position(&self, parent_position: Vec2) -> Vec2 {
        parent_position + self.local_position()
    }

    fn overlaps_with(&self, tile: &Tile, parent: &Room) -> bool {
        false
    }

}

struct Room {
    // tile coordinates are local offsets relative to where the room instance is
    tiles: Vec<Tile>,
    bounding_box: Rect,
    tile_position: Vec2
}

impl Room {

    fn world_position(&self) -> Vec2 {
        self.tile_position * TILE_SIZE
    }

    fn overlaps_with(&self, room: &Room) -> bool {

        for tile in &self.tiles {
            for other_tile in &room.tiles {
                if tile.overlaps_with(other_tile, room) {
                    return true;
                }
            }
        }

        false
    }

}

fn is_ignored_path(path: &PathBuf) -> bool {
    path.ends_with("readme.txt")
}

fn load_all_rooms_from_file(path: &str) -> Vec<Room> {

    let mut found_rooms = Vec::new();

    for path in std::fs::read_dir(path).unwrap() {
        let path_string = path.unwrap().path();
        if let Some(room) = load_room_from_file(&path_string) && is_ignored_path(&path_string) == false {
            found_rooms.push(room);
        }
    }

    found_rooms

}

fn char_to_tile_kind(c: char) -> TileKind {
    // * = air
    // # = wall
    // . = floor
    // ? = random object
    match c {
        '*' => TileKind::Air,
        '#' => TileKind::Wall,
        '.' => TileKind::Floor,
        '?' => TileKind::Any,
        _ => TileKind::Air
    }
}

fn tile_with_kind(c: char, position: Vec2) -> Tile {
    let tile_kind = char_to_tile_kind(c);
    Tile {
        tile_position: position,
        kind: tile_kind
    }
}

fn load_room_from_file(path: &PathBuf) -> Option<Room> {
    
    let contents = read_to_string(path).ok()?;

    let mut new_room = Room {
        tiles: Vec::new(),
        bounding_box: Rect::default(),
        tile_position: Vec2::ZERO
    };

    let mut current_pos = Vec2::ZERO;

    for line in contents.lines() {
        current_pos.x = 0.0;
        for c in line.chars() {
            let new_tile = tile_with_kind(c, current_pos);
            new_room.tiles.push(new_tile);
            current_pos += vec2(1.0, 0.0);
        }
        current_pos += vec2(0.0, 1.0);
    }

    Some(new_room)

}

fn draw_wireframe_cross(position: Vec2, line_thickness: f32, line_colour: Color) {

    let top_right = position + vec2(TILE_SIZE, 0.0);
    let bottom_right = position + vec2(TILE_SIZE, TILE_SIZE);
    let bottom_left = position + vec2(0.0, TILE_SIZE);

    draw_line(
        position.x, position.y,
        bottom_right.x, bottom_right.y,
        line_thickness, line_colour
    );

    draw_line(
        top_right.x, top_right.y,
        bottom_left.x, bottom_left.y,
        line_thickness, line_colour
    );

}

fn draw_wireframe_box(position: Vec2, line_thickness: f32, line_colour: Color) {

    let top_right = position + vec2(TILE_SIZE, 0.0);
    let bottom_right = position + vec2(TILE_SIZE, TILE_SIZE);
    let bottom_left = position + vec2(0.0, TILE_SIZE);

    draw_line(position.x, position.y, top_right.x, top_right.y, line_thickness, line_colour);
    draw_line(top_right.x, top_right.y, bottom_right.x, bottom_right.y, line_thickness, line_colour);
    draw_line(bottom_right.x, bottom_right.y, bottom_left.x, bottom_left.y, line_thickness, line_colour);
    draw_line(bottom_left.x, bottom_left.y, position.x, position.y, line_thickness, line_colour);

}

fn draw_wireframe_tile(position: Vec2, line_thickness: f32, line_colour: Color) {
    draw_wireframe_box(position, line_thickness, line_colour);
}

fn draw_wall_tile(position: Vec2) {

    let line_thickness = 2.0;
    draw_wireframe_tile(position, line_thickness, BLACK)

}

fn draw_floor_tile(position: Vec2) {

    let line_thickness = 1.0;
    draw_wireframe_cross(position, line_thickness, BLACK)

}

fn draw_tile(position: Vec2, tile: &Tile) {

    let current_tile_position = position + tile.local_position();

    match tile.kind {
        TileKind::Wall => draw_wall_tile(current_tile_position),
        TileKind::Floor => draw_floor_tile(current_tile_position),
        _ => {}
    };

}

fn draw_room(room: &Room) {
    for tile in &room.tiles {
        draw_tile(room.world_position(), tile);
    }
}

fn draw_all_rooms(rooms: &Vec<&Room>) {
    for room in rooms {
        draw_room(room);
    }
}

/// Tries to generate a dungeon, returning any generated rooms.
fn generate_dungeon(goal_room_count: i32) -> Vec<Room> {

    let max_number_of_tries = 1000;

    let mut rooms = Vec::new();
    let mut room_count = 0;
    let mut num_tries = 0;

    while goal_room_count < room_count && num_tries < max_number_of_tries {



    }

    rooms

}

/// Gets the position of all wall tiles with two adjacent wall tiles on the same axis.
fn get_candidate_walls(rooms: &Vec<Room>) -> Vec<Vec2> {

    let mut candidate_walls = Vec::new();

    for room in rooms {
        for tile in &room.tiles {

        }
    }

    candidate_walls

}

fn handle_camera_input(camera: &mut Camera2D, dt: f32) -> bool {

    let camera_movement_speed = 1.0;

    let is_up_pressed = is_key_down(KeyCode::Up) || is_key_down(KeyCode::W);
    let is_down_pressed = is_key_down(KeyCode::Down) || is_key_down(KeyCode::S);
    let is_left_pressed = is_key_down(KeyCode::Left) || is_key_down(KeyCode::A);
    let is_right_pressed = is_key_down(KeyCode::Right) || is_key_down(KeyCode::D);

    let camera_v_delta = ((is_down_pressed as i32) - (is_up_pressed as i32)) as f32;
    let camera_h_delta = ((is_left_pressed as i32) - (is_right_pressed as i32)) as f32;
    let camera_delta = vec2(camera_h_delta, camera_v_delta) * camera_movement_speed * dt;

    camera.offset += camera_delta;

    // if the camera moved, return this information
    let camera_moved = camera_delta.x != 0.0 || camera_delta.y != 0.0;
    camera_moved

}

#[macroquad::main("dungeons")]
async fn main() {

    let mut camera = Camera2D::from_display_rect(
        Rect {
            x: 0.0, y: 0.0,
            w: screen_width(),
            h: screen_height()
        }
    );

    let available_rooms = load_all_rooms_from_file("data/rooms");
    let first_room = available_rooms.last().unwrap();
    let current_rooms = vec![first_room];

    loop {

        let dt = get_frame_time();

        set_camera(&camera);
        clear_background(WHITE);

        handle_camera_input(&mut camera, dt);
        draw_all_rooms(&current_rooms);

        next_frame().await;

    }

}