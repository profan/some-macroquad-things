#![feature(let_chains)]
#![feature(map_try_insert)]

use std::{path::PathBuf, fs::read_to_string, collections::HashMap};
use macroquad::{prelude::*, rand::{gen_range, ChooseRandom}};
use utility::*;

const TILE_SIZE: f32 = 32.0;

pub trait RandomChoice {
    type Item;
    fn choose(&self) -> Self::Item;
}

pub trait AsWorldPosition {
    fn as_world_position(&self) -> Vec2;
}

impl AsWorldPosition for IVec2 {
    fn as_world_position(&self) -> Vec2 {
        vec2(
            (self.x as f32) * TILE_SIZE,
            (self.y as f32) * TILE_SIZE
        )
    }
}

pub trait AsTilePosition {
    fn as_tile_position(&self) -> IVec2;
}

impl AsTilePosition for Vec2 {
    fn as_tile_position(&self) -> IVec2 {
        ivec2(
            (self.x / TILE_SIZE) as i32,
            (self.y / TILE_SIZE) as i32
        )
    }
}

#[derive(Clone, Copy, PartialEq)]
enum TileKind {
    Air,
    Wall,
    Floor,
    Any
}

struct Tile {
    tile_position: IVec2,
    kind: TileKind
}

impl Tile {

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

struct TileMap {
    tiles: HashMap<IVec2, TileKind>
}

impl TileMap {

    pub fn new() -> TileMap {
        TileMap {
            tiles: HashMap::new()
        }
    }

    pub fn vertical_neighbours(&self, position: IVec2) -> impl Iterator<Item=(IVec2, TileKind)> {

        let above_pos = position + ivec2(0, -1);
        let below_pos = position + ivec2(0, 1);

        let above = self.tiles.get_key_value(&above_pos)
            .or(Some((&above_pos, &TileKind::Air)));

        let below = self.tiles.get_key_value(&below_pos)
            .or(Some((&below_pos, &TileKind::Air)));
        
        [
            above.map(|(p, t)| (*p, *t)),
            below.map(|(p, t)| (*p, *t))
        ].into_iter().flatten()

    }

    pub fn horizontal_neighbours(&self, position: IVec2) -> impl Iterator<Item=(IVec2, TileKind)> {

        let left_pos = position + ivec2(-1, 0);
        let right_pos = position + ivec2(1, 0);

        let left = self.tiles.get_key_value(&left_pos)
            .or(Some((&left_pos, &TileKind::Air)));

        let right = self.tiles.get_key_value(&right_pos)
            .or(Some((&right_pos, &TileKind::Air)));
    
        [
            left.map(|(p, t)| (*p, *t)),
            right.map(|(p, t)| (*p, *t))
        ].into_iter().flatten()

    }

    pub fn neighbours(&self, position: IVec2) -> impl Iterator<Item=(IVec2, TileKind)> {
        
        let above_pos = position + ivec2(0, -1);
        let below_pos = position + ivec2(0, 1);
        let left_pos = position + ivec2(-1, 0);
        let right_pos = position + ivec2(1, 0);

        let above = self.tiles.get_key_value(&above_pos)
            .or(Some((&above_pos, &TileKind::Air)));

        let below = self.tiles.get_key_value(&below_pos)
            .or(Some((&below_pos, &TileKind::Air)));

        let left = self.tiles.get_key_value(&left_pos)
            .or(Some((&left_pos, &TileKind::Air)));

        let right = self.tiles.get_key_value(&right_pos)
            .or(Some((&right_pos, &TileKind::Air)));

        [
            above.map(|(p, t)| (*p, *t)),
            below.map(|(p, t)| (*p, *t)),
            left.map(|(p, t)| (*p, *t)),
            right.map(|(p, t)| (*p, *t))
        ].into_iter().flatten()

    }

    /// Checks if the given tile can be added at the given position.
    pub fn can_add_tile(&self, position: IVec2, _tile: TileKind) -> bool {
        self.tiles.get(&position).is_none()
    }

    /// Tries to add a tile at a position, returning true if the tile was not already present,
    ///  returns false and does not modify the tile if a tile was already present.
    pub fn try_add_tile(&mut self, position: IVec2, tile: TileKind) -> bool {
        self.tiles.try_insert(position, tile).is_ok()
    }

    /// Adds a tile at a position, returning true if the tile was not already present.
    pub fn add_tile(&mut self, position: IVec2, tile: TileKind) -> bool {
        self.tiles.insert(position, tile).is_none()
    }

    /// Gets the tile at a given position, if present.
    pub fn get_tile(&self, position: IVec2) -> Option<TileKind> {
        self.tiles.get(&position).cloned()
    }

}

struct Dungeon {
    map: TileMap
}

impl Dungeon {

    pub fn new() -> Dungeon {
        Dungeon {
            map: TileMap::new()
        }
    }

    pub fn bounding_box(&self) -> Rect {
        Rect {
            x: 0.0,
            y: 0.0,
            w: 0.0,
            h: 0.0
        }
    }

    /// Tries to place a given room at a position, returning true if successful.
    pub fn try_place_room(&mut self, position: IVec2, room: &Room) -> bool {

        let are_all_tiles_placeable = (&room.tiles).into_iter()
            .all(|t| self.map.can_add_tile(t.tile_position + position, t.kind));

        if are_all_tiles_placeable == false {
            return false;
        }

        for tile in &room.tiles {
            let current_tile_position = tile.tile_position + position;
            self.map.add_tile(current_tile_position, tile.kind);
        }

        true
        
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

fn tile_with_kind(c: char, position: IVec2) -> Tile {
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

    let mut current_pos = IVec2::ZERO;

    for line in contents.lines() {
        current_pos.x = 0;
        for c in line.chars() {
            let new_tile = tile_with_kind(c, current_pos);
            new_room.tiles.push(new_tile);
            current_pos += ivec2(1, 0);
        }
        current_pos += ivec2(0, 1);
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

fn draw_vertical_door_tile(position: Vec2) {

}

fn draw_horizontal_door_tile(position: Vec2) {

}

fn draw_tile(position: Vec2, tile_kind: &TileKind) {

    match tile_kind {
        TileKind::Wall => draw_wall_tile(position),
        TileKind::Floor => draw_floor_tile(position),
        _ => {}
    };

}

fn draw_tilemap(tilemap: &TileMap) {

    for (tile_pos, tile) in &tilemap.tiles {
        let world_position = tile_pos.as_world_position();
        draw_tile(world_position, tile)
    }

}

fn draw_dungeon(dungeon: &Dungeon) {
    draw_tilemap(&dungeon.map);
}

fn add_walls_to_dungeon(dungeon: &mut Dungeon) {

    let mut tiles_to_create_walls_in = Vec::new();

    for (&tile_pos, &tile) in &dungeon.map.tiles {

        let neighbours = dungeon.map.neighbours(tile_pos);

        if tile != TileKind::Floor || tile == TileKind::Air {
            continue;
        }

        for (n_pos, n) in neighbours {
            if n == TileKind::Air {
                tiles_to_create_walls_in.push(n_pos);
            }
        }

    }

    for tile_pos in tiles_to_create_walls_in {
        dungeon.map.add_tile(tile_pos, TileKind::Wall);
    }

}

/// Tries to generate a dungeon, returning any generated rooms.
fn generate_dungeon(goal_room_count: i32) -> Dungeon {

    let available_rooms = load_all_rooms_from_file("data/rooms");

    let max_number_of_tries = 1000;
    let mut dungeon = Dungeon::new();
    let mut room_count = 0;
    let mut num_tries = 0;

    // place the first room, this should not be able to fail really
    dungeon.try_place_room(ivec2(0, 0), available_rooms.last().unwrap());

    while room_count < goal_room_count && num_tries < max_number_of_tries {

        num_tries += 1;

        let candidates = get_candidate_walls(&dungeon.map);
        if candidates.len() == 0 {
            println!("generate_dungeon: aborting as no candidate walls? should be impossible!");
            break;
        }

        let random_candidate = *candidates.choose().unwrap();
        let random_room = available_rooms.choose().unwrap();

        println!("generate_dungeon: trying to place room at: {}, try: {}", random_candidate, num_tries);

        // try to place the room, maybe we'll succeed
        if dungeon.try_place_room(random_candidate, random_room) {
            println!("generate_dungeon: placed room at: {}, try: {}", random_candidate, num_tries);
            room_count += 1;
        }

    }

    add_walls_to_dungeon(&mut dungeon);

    dungeon

}

/// Gets the position of all wall tiles with two adjacent wall tiles on the same axis,
///  where one side of the wall is also exposed to unused space.
fn get_candidate_walls(map: &TileMap) -> Vec<IVec2> {

    let mut candidate_walls = Vec::new();

    for (&tile_pos, _) in &map.tiles {

        let mut neighbours = map.neighbours(tile_pos);
        let mut vertical_neighbours = map.vertical_neighbours(tile_pos);
        let mut horizontal_neighbours = map.horizontal_neighbours(tile_pos);

        let has_two_adjacent_wall_tiles_h = vertical_neighbours.any(|(_p, t)| t != TileKind::Air);
        let has_two_adjacent_wall_tiles_v = horizontal_neighbours.any(|(_p, t)| t != TileKind::Air);
        let is_any_neighbour_exposed_to_unused_space = neighbours.any(|(_p, t)| t == TileKind::Air);

        if (has_two_adjacent_wall_tiles_h || has_two_adjacent_wall_tiles_v) && is_any_neighbour_exposed_to_unused_space {

            if has_two_adjacent_wall_tiles_h {
                candidate_walls.extend(
                    horizontal_neighbours.filter(|(_p, t)| *t == TileKind::Air).map(|(p, _t)| p)
                );
            }

            if has_two_adjacent_wall_tiles_v {
                candidate_walls.extend(
                    vertical_neighbours.filter(|(_p, t)| *t == TileKind::Air).map(|(p, _t)| p)
                );
            }

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

fn draw_debug_text(camera: &Camera2D) {

    let world_position_under_mouse = camera.screen_to_world(mouse_position().into());
    let tile_under_mouse = world_position_under_mouse.as_tile_position();

    draw_text(format!("tile pos: {}", tile_under_mouse).as_str(), screen_width() - 128.0, 32.0, 16.0, BLACK);

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

    let dungeon = generate_dungeon(12);

    loop {

        let dt = get_frame_time();

        clear_background(WHITE);
        handle_camera_input(&mut camera, dt);

        set_camera(&camera);
        draw_dungeon(&dungeon);

        set_default_camera();
        draw_debug_text(&camera);

        next_frame().await;

    }

}