use std::vec;

use macroquad::{prelude::{*, camera::mouse}, rand::gen_range};
use utility::{DebugText, draw_arrow, WithAlpha};

const PHYSICS_TIMESTEP: f32 = 1.0 / 60.0;

struct Game {
    camera: Camera2D,
    debug_text: DebugText,
    world: World
}

impl Game {

    pub fn new() -> Game {

        let camera = Camera2D::from_display_rect(
            Rect {
                x: 0.0, y: 0.0,
                w: screen_width(),
                h: screen_height()
            }
        );
        
        let mut debug_text = DebugText::new();

        let world = World::new();

        Game {
            camera,
            debug_text,
            world
        }

    }

    pub fn mouse_world_position(&self) -> Vec2 {
        self.camera.screen_to_world(mouse_position().into())
    }

    pub fn world_to_screen(&self, position: Vec2) -> Vec2 {
        self.camera.world_to_screen(position)
    }

    pub fn screen_to_world(&self, position: Vec2) -> Vec2 {
        self.camera.screen_to_world(position)
    }

}

struct World {
    entities: Vec<Entity>,
    timestep: f32,
    damping: f32,
}

impl World {

    pub fn new() -> World {
        World {
            entities: Vec::new(),
            timestep: PHYSICS_TIMESTEP,
            damping: 0.985,
        }
    }

    pub fn add_entity(&mut self, entity: Entity) {
        self.entities.push(entity);
    }

}

#[derive(PartialEq)]
struct Entity {
    position: Vec2,
    velocity: Vec2
}

fn spawn_some_entities(world: &mut World, number_of_entities: i32) {

    let buffer = 100.0;

    let start_x = 100.0;
    let end_x = screen_width() - buffer;

    let start_y = 100.0;
    let end_y = screen_height() - buffer;

    for _i in 0..number_of_entities {

        let rand_x = gen_range(start_x, end_x);
        let rand_y = gen_range(start_y, end_y);
        
        let rand_vx = gen_range(-32.0, 32.0);
        let rand_vy = gen_range(-32.0, 32.0);

        world.add_entity(Entity {
            position: vec2(rand_x, rand_y),
            velocity: vec2(rand_vx, rand_vy)
        });

    }

}

fn push_entities_near_mouse(game: &mut Game) {

    let mouse_push_threshold = 64.0;

    let mouse_force_threshold = 32.0;
    let mouse_world_position = game.mouse_world_position();

    for e in &mut game.world.entities {

        let vector_to_mouse = e.position - mouse_world_position;
        let clamped_vector_to_mouse = vector_to_mouse.clamp_length_max(mouse_force_threshold);

        if e.position.distance(mouse_world_position) < mouse_push_threshold {
            e.velocity += (clamped_vector_to_mouse.normalize() * mouse_force_threshold) - clamped_vector_to_mouse;
        }

    }

}

fn step_physics(world: &mut World) {

    for e in &mut world.entities {
        e.position += e.velocity * world.timestep;
        e.velocity *= world.damping;
    }

}

fn draw_entities(game: &Game) {

    let mouse_push_threshold = 64.0;

    let entity_radius = 8.0;
    let entity_thickness = 2.0;

    let world_mouse_pos = game.mouse_world_position();

    for e in &game.world.entities {

        let distance_to_mouse_in_world = e.position.distance(world_mouse_pos);

        if distance_to_mouse_in_world < mouse_push_threshold
        {
            let arrow_thickness = 2.0;
            let arrow_head_thickness = 2.0;
            let arrow_head_alpha = (mouse_push_threshold - distance_to_mouse_in_world) / mouse_push_threshold;
            draw_arrow(e.position.x, e.position.y, world_mouse_pos.x, world_mouse_pos.y, arrow_thickness, arrow_head_thickness, DARKGRAY.with_alpha(arrow_head_alpha));
        }

        draw_circle_lines(e.position.x, e.position.y, entity_radius, entity_thickness, DARKGRAY);

    }

}

#[macroquad::main("graph")]
async fn main() {

    let mut game = Game::new();
    let number_of_entities = 100;

    spawn_some_entities(&mut game.world, number_of_entities);
    let mut current_physics_time = 0.0;

    loop {

        let dt = get_frame_time();
        game.debug_text.new_frame();

        clear_background(WHITE);

        if current_physics_time > game.world.timestep {
            current_physics_time = 0.0;
            push_entities_near_mouse(&mut game);
            step_physics(&mut game.world);
        }
        else
        {
            current_physics_time += dt;
        }

        draw_entities(&game);

        next_frame().await;

    }

}