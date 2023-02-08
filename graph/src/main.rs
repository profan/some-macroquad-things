use std::{collections::HashMap};

use macroquad::{prelude::{*}, rand::gen_range};
use utility::{DebugText, draw_arrow, WithAlpha, TextPosition};
use petgraph::{graph::{UnGraph}, visit::EdgeRef};

const PHYSICS_TIMESTEP: f32 = 1.0 / 60.0;

/// Spring-Damper system per Hooke's Law, F = -kx - bv
/// where:
/// x = vector displacement of the end of the spring from it’s equilibrium position
/// k = tightness of the spring
/// b = coefficient of damping
/// v = relative velocity between the two points connected by the spring
struct Spring {
    damping: f32,
    tightness: f32,
    rest_length: f32
}

impl Spring {

    pub fn evaluate_spring_force(&self, entity_a: &Entity, entity_b: &Entity) -> Vec2 {

        let clamped_vector_displacement = (entity_b.position - entity_a.position).clamp_length(0.0, self.rest_length);
        let x = (entity_b.position - entity_a.position) - clamped_vector_displacement;
        let k = self.tightness;
        let b = self.damping;
        let v = entity_b.velocity - entity_a.velocity;
        let f = -k*x - b*v;

        f

    }

}

struct Game {

    current_physics_time: f32,

    camera: Camera2D,
    debug_text: DebugText,
    world_graph: UnGraph::<i32, Spring>,
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

        let current_physics_time = 0.0;
        let debug_text = DebugText::new();
        let world_graph = UnGraph::new_undirected();
        let world = World::new();

        Game {
            current_physics_time,
            camera,
            debug_text,
            world_graph,
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
    current_entity_idx: i32,
    entities: HashMap<i32, Entity>,
    timestep: f32,
    damping: f32,
}

impl World {

    pub fn new() -> World {
        World {
            current_entity_idx: 0,
            entities: HashMap::new(),
            timestep: PHYSICS_TIMESTEP,
            damping: 0.985,
        }
    }

    pub fn clear(&mut self) {
        self.entities.clear()
    }

    pub fn add_entity(&mut self, entity: Entity) {
        self.entities.insert(self.current_entity_idx, entity);
        self.current_entity_idx += 1;;
    }

    pub fn get_entity_mut(&mut self, idx: i32) -> &mut Entity {
        self.entities.get_mut(&idx).unwrap()
    }

    pub fn get_entity(&self, idx: i32) -> &Entity {
        self.entities.get(&idx).unwrap()
    }

    pub fn get_entity_maybe(&self, idx: i32) -> Option<&Entity> {
        self.entities.get(&idx)
    }

}

#[derive(PartialEq)]
struct Entity {
    position: Vec2,
    velocity: Vec2
}

fn create_default_spring() -> Spring {
    Spring {
        damping: 0.25,
        tightness: 2.25,
        rest_length: 64.0
    }
}

fn create_default_spring_with_length(rest_length: f32) -> Spring {
    return Spring {
        rest_length: rest_length,
        ..create_default_spring()
    }
}

fn spawn_some_entities(world: &mut World, number_of_entities: i32) {

    let buffer = 100.0;

    let start_x = buffer;
    let end_x = screen_width() - buffer;

    let start_y = buffer;
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

/// for each entity in the world, connect it to the next entity (unless it is the last entity), forming a long chain!
fn connect_some_entities_in_a_chain(world: &World, world_graph: &mut UnGraph::<i32, Spring>) {

    // add all the nodes to the graph first

    for (&entity_id, _entity) in &world.entities {
        world_graph.add_node(entity_id);
    }

    // then connect things

    for (&entity_id, _entity) in &world.entities {

        let next_entity_id = entity_id + 1;

        if let Some(_next_entity) = world.get_entity_maybe(next_entity_id) {

            world_graph.add_edge((entity_id as u32).into(), (next_entity_id as u32).into(), create_default_spring());

        } else {

            // TODO: maybe connect the last ndoe to the beginning to form a loop instead of a line? nah for now
            // world_graph.add_edge((entity_id as u32).into(), (next_entity_id as u32).into(), create_default_spring());

        }

    }

}

/// for every entity, connect the next three entities to this first entity, then continue and connect the next entity to the first entity and repeat.
fn connect_some_entities_to_hubs(world: &World, world_graph: &mut UnGraph::<i32, Spring>) {

    // add all the nodes to the graph first

    for (&entity_id, _entity) in &world.entities {
        world_graph.add_node(entity_id);
    }

    // then connect things

    let mut current_entity_idx = 0;
    let mut last_hub_entity_idx = 0;

    while current_entity_idx < world.current_entity_idx {
        
        let hub_entity_idx = current_entity_idx;

        // connect the spokes
        for spoke_entity_idx in (current_entity_idx + 1)..(current_entity_idx + 4) {
            world_graph.add_edge((hub_entity_idx as u32).into(), (spoke_entity_idx as u32).into(), create_default_spring());
            current_entity_idx += 1;
        }

        // connect to our last hub
        if hub_entity_idx != last_hub_entity_idx
        {
            world_graph.add_edge((hub_entity_idx as u32).into(), (last_hub_entity_idx as u32).into(), create_default_spring_with_length(256.0));
        }

        last_hub_entity_idx = hub_entity_idx;
        current_entity_idx += 1;

    }

}

fn push_entities_near_mouse(game: &mut Game) {

    let is_left_mouse_down = is_mouse_button_down(MouseButton::Left);
    let is_right_mouse_down = is_mouse_button_down(MouseButton::Right);
    let is_shift_down = is_key_down(KeyCode::LeftShift);

    let mouse_push_threshold = 64.0;

    let mouse_force_threshold = 32.0;
    let mouse_world_position = game.mouse_world_position();

    for (_entity_id, e) in &mut game.world.entities {

        let vector_to_mouse = e.position - mouse_world_position;
        let clamped_vector_to_mouse = vector_to_mouse.clamp_length_max(mouse_force_threshold);

        if e.position.distance(mouse_world_position) < mouse_push_threshold || is_shift_down {

            let force_vector = if is_shift_down == false {
                (clamped_vector_to_mouse.normalize() * mouse_force_threshold) - clamped_vector_to_mouse
            } else {
                (clamped_vector_to_mouse.normalize() * mouse_force_threshold)
            };

            // when right mouse is down, push
            if is_right_mouse_down {
                e.velocity += force_vector;
            }

            // when left mouse is down, pull
            if is_left_mouse_down {
                e.velocity -= force_vector;
            }

        }

    }

}

fn push_entities_near_eachother(game: &mut Game) {

    let entity_push_threshold = 64.0;
    let entity_push_force = 8.0;

    let mut forces_to_apply = Vec::new();

    for (entity_id, entity) in &game.world.entities {
        for (other_entity_id, other_entity) in &game.world.entities {

            if entity_id == other_entity_id { continue }

            if entity.position.distance(other_entity.position) < entity_push_threshold {
                let force_vector = (other_entity.position - entity.position).normalize() * entity_push_force;
                forces_to_apply.push((*entity_id, *other_entity_id, force_vector));
            }

        }
    }

    for (source_entity_id, target_entity_id, force_vector) in &forces_to_apply {

        let source_entity = game.world.get_entity_mut(*source_entity_id);
        source_entity.velocity += -(*force_vector / 2.0);

        let target_entity = game.world.get_entity_mut(*target_entity_id);
        target_entity.velocity += *force_vector / 2.0;
        
    }

}

fn step_physics(world: &mut World, world_graph: &UnGraph::<i32, Spring>) {

    let w = screen_width();
    let h = screen_height();

    let spring_forces = calculate_physics_for_springs(world, world_graph);
    step_physics_for_entities(world, &spring_forces, w, h);

}

fn calculate_physics_for_springs(world: &mut World, world_graph: &UnGraph::<i32, Spring>) -> Vec<(i32, i32, Vec2)> {

    let mut accumulated_forces = Vec::new();

    for node_idx in world_graph.node_indices() {

        let current_entity_id = *world_graph.node_weight(node_idx).unwrap();

        for edge in world_graph.edges(node_idx) {

            let source_entity_id = *world_graph.node_weight(edge.source()).unwrap();
            let target_entity_id = *world_graph.node_weight(edge.target()).unwrap();
            let data = edge.weight();

            // only evaluate springs using the source to avoid evaluating spring forces twice
            if current_entity_id == source_entity_id {
                let source_entity = world.get_entity(source_entity_id);
                let target_entity = world.get_entity(target_entity_id);
                let v = data.evaluate_spring_force(source_entity, target_entity);
                accumulated_forces.push((source_entity_id, target_entity_id, v));
            }

        }
    }

    accumulated_forces
    
}

fn step_physics_for_entities(world: &mut World, spring_forces: &Vec<(i32, i32, Vec2)>, w: f32, h: f32) {

    let with_gravity = false;

    for (entity_id, e) in &mut world.entities {

        // main physics integration step

        e.position += e.velocity * world.timestep;
        e.velocity *= world.damping;

        if with_gravity {
            e.velocity += vec2(0.0, 9.82);
        }

        // apply spring forces, if any for this entity

        if let Some((src_id, target_id, v)) = spring_forces.into_iter().find(|(src_id, target_id, _v)| src_id == entity_id || target_id == entity_id) {

            // force and counter-force

            if src_id == entity_id {
                e.velocity += -(*v / 2.0);
            }

            if target_id == entity_id {
                e.velocity += *v / 2.0;
            }

        }

        // keep our little objects inside the box

        let clamp_offset = 0.5;

        if e.position.x < 0.0 || e.position.x > w {

            if (e.velocity.x < 0.0 && e.position.x < 0.0) || (e.velocity.x > 0.0 && e.position.x > w) {
                e.velocity.x *= -1.0;
            }

            if e.position.x < 0.0 {
                e.position.x = clamp_offset;
            }

            if e.position.x > h {
                e.position.x = w - clamp_offset;
            }

        }

        if e.position.y < 0.0 || e.position.y > h {

            if (e.velocity.y < 0.0 && e.position.y < 0.0) || (e.velocity.y > 0.0 && e.position.y > h) {
                e.velocity.y *= -1.0;
            }

            if e.position.y < 0.0 {
                e.position.y = clamp_offset;
            }

            if e.position.y > h {
                e.position.y = h - clamp_offset;
            }

        }

    }

}

fn draw_entities(game: &Game) {

    let is_shift_down = is_key_down(KeyCode::LeftShift);

    let mouse_push_threshold = 64.0;

    let entity_radius = 8.0;
    let entity_thickness = 2.0;

    let world_mouse_pos = game.mouse_world_position();

    // draw all the entities

    for (_entity_id, e) in &game.world.entities {

        let distance_to_mouse_in_world = e.position.distance(world_mouse_pos);

        let arrow_thickness = 2.0;
        let arrow_head_thickness = 2.0;

        if distance_to_mouse_in_world < mouse_push_threshold {
            let arrow_head_alpha = (mouse_push_threshold - distance_to_mouse_in_world) / mouse_push_threshold;
            draw_arrow(e.position.x, e.position.y, world_mouse_pos.x, world_mouse_pos.y, arrow_thickness, arrow_head_thickness, DARKGRAY.with_alpha(arrow_head_alpha));
        }

        if is_shift_down {
            let arrow_head_alpha = 0.5;
            draw_arrow(e.position.x, e.position.y, world_mouse_pos.x, world_mouse_pos.y, arrow_thickness, arrow_head_thickness, DARKGRAY.with_alpha(arrow_head_alpha));
        }

        draw_circle_lines(e.position.x, e.position.y, entity_radius, entity_thickness, DARKGRAY);

    }

    // draw all the entities each entity is connected to

    for node_idx in game.world_graph.node_indices() {

        for edge in game.world_graph.edges(node_idx) {

            let source_entity_id = *game.world_graph.node_weight(edge.source()).unwrap();
            let target_entity_id = *game.world_graph.node_weight(edge.target()).unwrap();

            let source_entity = game.world.get_entity(source_entity_id);
            let target_entity = game.world.get_entity(target_entity_id);

            draw_line(source_entity.position.x, source_entity.position.y, target_entity.position.x, target_entity.position.y, entity_thickness, DARKGRAY);

        }

    }
    

}

#[macroquad::main("graph")]
async fn main() {

    let number_of_entities = 32;
    let mut has_game_state_been_created = false;
    let mut game = Game::new();

    loop {

        let was_reset_pressed = is_key_pressed(KeyCode::R);
        if was_reset_pressed {
            has_game_state_been_created = false;
        }

        if has_game_state_been_created == false {

            game = Game::new();

            spawn_some_entities(&mut game.world, number_of_entities);
            // connect_some_entities(&game.world, &mut game.world_graph);
            connect_some_entities_to_hubs(&game.world, &mut game.world_graph);

            has_game_state_been_created = true;

        }

        let dt = get_frame_time();
        game.debug_text.new_frame();

        clear_background(WHITE);

        game.debug_text.draw_text("left mouse to pull objects towards the mouse", TextPosition::TopLeft, BLACK);
        game.debug_text.draw_text(".. or right mouse to push them away", TextPosition::TopLeft, BLACK);
        game.debug_text.draw_text("(+ shift to do it for everything)", TextPosition::TopLeft, BLACK);
        game.debug_text.draw_text("press r to reset the game state", TextPosition::TopLeft, BLACK);

        if game.current_physics_time > game.world.timestep {
            game.current_physics_time = 0.0;
            push_entities_near_mouse(&mut game);
            push_entities_near_eachother(&mut game);
            step_physics(&mut game.world, &mut game.world_graph);
        } else {
            game.current_physics_time += dt;
        }

        draw_entities(&game);

        next_frame().await;

    }

}
