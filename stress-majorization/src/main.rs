use std::{collections::HashMap};

use macroquad::{prelude::{*}, rand::gen_range};
use utility::{draw_arrow, Camera2DExt, DebugText, TextPosition, WithAlpha};
use petgraph::{graph::{UnGraph}, visit::EdgeRef};

struct GraphEdge {
    length: f32
}

fn create_graph_edge(length: f32) -> GraphEdge {
    GraphEdge { length }
}

struct Game {

    camera: Camera2D,
    debug_text: DebugText,
    world_graph: UnGraph::<i32, GraphEdge>,
    world: World

}

impl Game {

    pub fn new() -> Game {

        let camera = Camera2D::from_display_rect_fixed(
            Rect {
                x: 0.0, y: 0.0,
                w: screen_width(),
                h: screen_height()
            }
        );

        let debug_text = DebugText::new();
        let world_graph = UnGraph::new_undirected();
        let world = World::new();

        Game {
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
    entities: HashMap<i32, Entity>
}

impl World {

    pub fn new() -> World {
        World {
            current_entity_idx: 0,
            entities: HashMap::new()
        }
    }

    pub fn clear(&mut self) {
        self.entities.clear()
    }

    pub fn add_entity(&mut self, entity: Entity) {
        self.entities.insert(self.current_entity_idx, entity);
        self.current_entity_idx += 1;
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

fn spawn_some_entities_in_a_grid(world: &mut World, number_of_entities: i32) {

    fn index_to_position(columns: i32, idx: i32) -> IVec2 {
        ivec2(
            idx % columns,
            idx / columns
        )
    }

    let buffer = 100.0;

    let start_x = buffer;
    let end_x = screen_width() - buffer;
    let width = end_x - start_x;

    let start_y = buffer;
    let end_y = screen_height() - buffer;
    let height = end_y - start_y;

    let square_of_entities = (number_of_entities as f32).sqrt() as i32;
    let number_of_columns = square_of_entities;
    let number_of_rows = square_of_entities;

    for i in 0..number_of_entities {

        let pos = index_to_position(number_of_columns, i);
        let x = pos.x;
        let y = pos.y;

        let entity_x = start_x + x as f32 * (width / number_of_columns as f32);
        let entity_y = start_y + y as f32 * (height / number_of_rows as f32);

        world.add_entity(Entity {
            position: vec2(entity_x, entity_y),
            velocity: vec2(0.0, 0.0)
        });

    }

}

/// for each entity in the world, connect it to the next entity (unless it is the last entity), forming a long chain!
fn connect_some_entities_in_a_chain(world: &World, world_graph: &mut UnGraph::<i32, GraphEdge>) {

    // add all the nodes to the graph first

    for (&entity_id, _entity) in &world.entities {
        world_graph.add_node(entity_id);
    }

    // then connect things

    for (&entity_id, _entity) in &world.entities {

        let next_entity_id = entity_id + 1;

        if let Some(_next_entity) = world.get_entity_maybe(next_entity_id) {

            world_graph.add_edge((entity_id as u32).into(), (next_entity_id as u32).into(), create_graph_edge(64.0));

        } else {

            // TODO: maybe connect the last ndoe to the beginning to form a loop instead of a line? nah for now
            // world_graph.add_edge((entity_id as u32).into(), (next_entity_id as u32).into(), create_default_spring());

        }

    }

}

/// for every entity, connect the next three entities to this first entity, then continue and connect the next entity to the first entity and repeat.
fn connect_some_entities_to_hubs(world: &World, world_graph: &mut UnGraph::<i32, GraphEdge>, should_form_loop: bool) {

    // add all the nodes to the graph first

    for (&entity_id, _entity) in &world.entities {
        world_graph.add_node(entity_id);
    }

    // then connect things

    let mut current_entity_idx = 0;
    let mut last_hub_entity_idx = 0;

    // define cluster size
    let cluster_size = 3;

    while current_entity_idx < world.current_entity_idx {
        
        let hub_entity_idx = current_entity_idx;

        // connect the spokes
        for spoke_entity_idx in (current_entity_idx + 1)..(current_entity_idx + (cluster_size + 1)) {
            world_graph.add_edge((hub_entity_idx as u32).into(), (spoke_entity_idx as u32).into(), create_graph_edge(64.0));
            current_entity_idx += 1;
        }

        // connect to our last hub
        if hub_entity_idx != last_hub_entity_idx
        {
            world_graph.add_edge((hub_entity_idx as u32).into(), (last_hub_entity_idx as u32).into(), create_graph_edge(256.0));
        }

        last_hub_entity_idx = hub_entity_idx;
        current_entity_idx += 1;

    }

    if should_form_loop {
        world_graph.add_edge((0 as u32).into(), (last_hub_entity_idx as u32).into(), create_graph_edge(256.0));
    }

}

fn connect_some_entities_as_grid(world: &World, world_graph: &mut UnGraph::<i32, GraphEdge>) {

    let square_of_entities = (world.entities.len() as f32).sqrt() as i32;

    let number_of_rows = square_of_entities;
    let number_of_columns = square_of_entities;

    fn position_to_grid_index(columns: i32, x: i32, y: i32) -> i32 {
        x + y * columns
    }

    // add all the nodes to the graph first

    let mut collected_entities: Vec<i32> = world.entities.iter().map(|e| *e.0).collect();
    collected_entities.sort();

    for entity_id in collected_entities {
        world_graph.add_node(entity_id);
    }

    for x in 0..number_of_columns {
        for y in 0..number_of_rows {

            let current_idx = position_to_grid_index(number_of_columns, x, y);

            let top_left_index = position_to_grid_index(number_of_columns, x - 1, y - 1);
            let top_right_index = position_to_grid_index(number_of_columns, x + 1, y - 1);
            let bottom_left_index = position_to_grid_index(number_of_columns, x - 1, y + 1);
            let bottom_right_index = position_to_grid_index(number_of_columns, x + 1, y + 1);

            let top_index = position_to_grid_index(number_of_columns, x, y - 1);
            let bottom_index = position_to_grid_index(number_of_columns, x, y + 1);
            let left_index = position_to_grid_index(number_of_columns, x - 1, y);
            let right_index = position_to_grid_index(number_of_columns, x + 1, y);

            // tl, tr, bl, br

            if (x - 1) >= 0 && (y - 1) >= 0 {
                world_graph.add_edge((current_idx as u32).into(), (top_left_index as u32).into(), create_graph_edge(64.0));
            }

            if (x + 1) < number_of_columns && (y - 1) >= 0 {
                world_graph.add_edge((current_idx as u32).into(), (top_right_index as u32).into(), create_graph_edge(64.0));
            }

            if (x - 1) >= 0 && (y + 1) < number_of_rows {
                world_graph.add_edge((current_idx as u32).into(), (bottom_left_index as u32).into(), create_graph_edge(64.0));
            }

            if (x + 1) < number_of_columns && (y + 1) < number_of_rows {
                world_graph.add_edge((current_idx as u32).into(), (bottom_right_index as u32).into(), create_graph_edge(64.0));
            }

            // t, b, l, r

            if (y - 1) >= 0 {
                world_graph.add_edge((current_idx as u32).into(), (top_index as u32).into(), create_graph_edge(64.0));
            }

            if (y + 1) < number_of_rows {
                world_graph.add_edge((current_idx as u32).into(), (bottom_index as u32).into(), create_graph_edge(64.0));
            }

            if (x - 1) >= 0 {
                world_graph.add_edge((current_idx as u32).into(), (left_index as u32).into(), create_graph_edge(64.0));
            }

            if (x + 1) < number_of_columns {
                world_graph.add_edge((current_idx as u32).into(), (right_index as u32).into(), create_graph_edge(64.0));
            }

        }
    }
        
}

fn calculate_neighbour_stress(world: &World, entity: &Entity) -> f32 {

    let k = 2;
    let entity_push_threshold = 63.0;

    let mut total_neighbour_stress = 0.0;

    for (_other_entity_id, other_entity) in &world.entities {

        if entity == other_entity { continue }

        if entity.position.distance(other_entity.position) < entity_push_threshold {

            let d_ij = (other_entity.position - entity.position).length();
            let t_ij = entity_push_threshold;

            let s_num = (d_ij  - t_ij) * (d_ij - t_ij);
            let s_denom = t_ij.powi(k);

            total_neighbour_stress += s_num / s_denom;

        }

    }

    total_neighbour_stress

}

fn calculate_neighbour_contribution(world: &World, entity: &Entity) -> Vec2 {

    let _k = 2;
    let entity_push_threshold = 63.0;

    let u_d = 2.0;
    let u = 1.0 / (2.0 * u_d);

    let mut total_neighbour_contribution = vec2(0.0, 0.0);

    for (_other_entity_id, other_entity) in &world.entities {

        if entity == other_entity { continue }

        if entity.position.distance(other_entity.position) < entity_push_threshold {

            let d_ij = (other_entity.position - entity.position).length();
            let t_ij = entity_push_threshold;

            let i_pos = entity.position;
            let j_pos = other_entity.position;

            let u_i = u;
            let xy_delta = -u_i * ((2.0 * (i_pos - j_pos) * (1.0 - t_ij / d_ij)) / 1.0);

            total_neighbour_contribution += xy_delta;

        }

    }

    total_neighbour_contribution

}

/// Calculates the stress function D_k
fn calculate_stress(world: &World, world_graph: &UnGraph::<i32, GraphEdge>) -> f32 {

    let k = 2;
    let mut total_stress = 0.0;

    for node_idx in world_graph.node_indices() {

        let current_entity_id = *world_graph.node_weight(node_idx).unwrap();

        for edge in world_graph.edges(node_idx) {

            let source_entity_id = *world_graph.node_weight(edge.source()).unwrap();
            let target_entity_id = *world_graph.node_weight(edge.target()).unwrap();
            let edge_data = edge.weight();

            if current_entity_id == source_entity_id {

                let source_entity = world.get_entity(source_entity_id);
                let target_entity = world.get_entity(target_entity_id);

                let d_ij = (target_entity.position - source_entity.position).length();
                let t_ij = edge_data.length;

                // calculate direct stress from spring not being at rest length
                let s_num = (d_ij  - t_ij) * (d_ij - t_ij);
                let s_denom = t_ij.powi(k);

                // calculate neighbour contribution to stress
                let neighbour_stress = calculate_neighbour_stress(world, source_entity);

                // accumulating D_k
                total_stress += s_num / s_denom;
                
                // adding neighbour stress as well now
                total_stress += neighbour_stress;

            }

        }
    }

    total_stress

}

/// Employs stress majorization for graph layouting
/// Original: https://dl.acm.org/doi/pdf/10.1145/264645.264657
fn minimize_stress_step(world: &mut World, world_graph: &mut UnGraph::<i32, GraphEdge>) {

    // n is the number of nodes in our graph
    let _n = world.entities.len() as f32;

    // suggested in  the paper
    // let u = 1.0 / (2.0 * n);

    let u_d = 2.0;
    let u = 1.0 / (2.0 * u_d);

    // possible methods:
    // s_0 = direct minimization of absolute stress (0)
    // s_1 = direct minimization of semiproportional stress (1)
    // s_2 = direct minimization of proportional stress (2)
    let _k = 2;

    for node_idx in world_graph.node_indices() {

        let current_entity_id = *world_graph.node_weight(node_idx).unwrap();

        for edge in world_graph.edges(node_idx) {

            let source_entity_id = *world_graph.node_weight(edge.source()).unwrap();
            let target_entity_id = *world_graph.node_weight(edge.target()).unwrap();
            let edge_data = edge.weight();

            if current_entity_id == source_entity_id {

                let source_entity = world.get_entity(source_entity_id);
                let target_entity = world.get_entity(target_entity_id);

                let d_ij = (target_entity.position - source_entity.position).length();
                let t_ij = edge_data.length;
                
                let i_pos = source_entity.position;
                let j_pos = target_entity.position;

                let u_i = u;
                // let x_delta = -u_i * ((2.0 * (x_i - x_j) * (1.0 - t_ij / d_ij)) / t_ij.powi(k));
                // let y_delta = -u_i * ((2.0 * (y_i - y_j) * (1.0 - t_ij / d_ij)) / t_ij.powi(k));
                let xy_delta = -u_i * ((2.0 * (i_pos - j_pos) * (1.0 - t_ij / d_ij)) / 1.0);

                // calculate contribution from any adjacent neighbours contributing to stress
                let neighbour_delta = calculate_neighbour_contribution(world, source_entity);

                // combine the two delta values, hopefully this adds up :D
                let combined_xy_delta = xy_delta + neighbour_delta;

                let source_entity = world.get_entity_mut(source_entity_id);
                source_entity.position += combined_xy_delta;

            }

        }
    }

}

fn calculate_centroid_of_entities(world: &World) -> Vec2 {

    if world.entities.is_empty() {
        return Vec2::ZERO;
    }

    let mut total_value = Vec2::ZERO;

    for (_entity_id, entity) in &world.entities {
        total_value += entity.position;
    }
    
    total_value / world.entities.len() as f32

}

fn center_camera_on_entities(game: &mut Game) {

    let current_camera_center = calculate_centroid_of_entities(&game.world);

    game.camera = Camera2D::from_display_rect_fixed(
        Rect {
            x: 0.0,
            y: 0.0,
            w: screen_width(),
            h: screen_height()
        }
    );

    game.camera.target = current_camera_center;

}

#[macroquad::main("stress-majorization")]
async fn main() {

    let number_of_entities = 24;
    let iterations_per_tick = 50;
    let max_iterations_per_attempt = 100;

    let mut current_iterations = 0;
    let mut has_game_state_been_created = false;
    let mut game = Game::new();

    loop {

        center_camera_on_entities(&mut game);

        let was_reset_pressed = is_key_pressed(KeyCode::R);
        if was_reset_pressed {
            has_game_state_been_created = false;
        }

        if has_game_state_been_created == false || current_iterations > max_iterations_per_attempt {

            game = Game::new();

            // spawn_some_entities(&mut game.world, number_of_entities);
            // spawn_some_entities_in_a_grid(&mut game.world, number_of_entities);
            // connect_some_entities_as_grid(&game.world, &mut game.world_graph);

            spawn_some_entities(&mut game.world, number_of_entities);
            // connect_some_entities_in_a_chain(&game.world, &mut game.world_graph);
            connect_some_entities_to_hubs(&game.world, &mut game.world_graph, false);
            has_game_state_been_created = true;
            current_iterations = 0;

        }

        let needs_to_step = calculate_stress(&game.world, &game.world_graph) > 1.0;

        if needs_to_step {
            for _i in 0..iterations_per_tick {
                minimize_stress_step(&mut game.world, &mut game.world_graph);
                current_iterations += 1;
            }
        }

        let _dt = get_frame_time();
        game.debug_text.new_frame();

        clear_background(WHITE);

        set_camera(&game.camera);
        draw_entities(&game);

        set_default_camera();
        game.debug_text.draw_text("press r to reset the game state", TextPosition::TopLeft, BLACK);
        game.debug_text.draw_text(format!("current graph stress: {}", calculate_stress(&game.world, &game.world_graph)), TextPosition::TopLeft, BLACK);
        game.debug_text.draw_text(format!("current graph iterations: {}", current_iterations), TextPosition::TopLeft, BLACK);
        game.debug_text.draw_text(format!("iterations per attempt: {}", max_iterations_per_attempt), TextPosition::TopLeft, BLACK);
        game.debug_text.draw_text(format!("iterations per frame: {}", iterations_per_tick), TextPosition::TopLeft, BLACK);

        next_frame().await;

    }

}
