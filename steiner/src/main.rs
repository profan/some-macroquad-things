use core::f32;
use std::collections::{HashMap, HashSet};

use macroquad::{conf::Conf, prelude::*, rand::gen_range};
use utility::{DebugText, GameCamera2D};

struct VertexData {
    position: Vec2,
    is_place: bool
}

struct EdgeData {
    is_place_connection: bool
}

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
struct Vertex(i32);

struct Edge {
    source: Vertex,
    target: Vertex
}

struct Graph {
    nodes: Vec<Vertex>,
    edges: Vec<Edge>,
    node_data: HashMap<Vertex, VertexData>,
    edge_data: HashMap<(Vertex, Vertex), EdgeData>,
}

impl Graph {
    pub fn new() -> Graph {
        Graph { nodes: Vec::new(), edges: Vec::new(), node_data: HashMap::new(), edge_data: HashMap::new() }
    }
}

struct WorldState {
    graph: Graph
}

impl WorldState {
    pub fn new() -> WorldState {
        WorldState {
            graph: Graph::new()
        }
    }
}

fn idx_to_grid_position(idx: i32, w: i32) -> IVec2 {
    return ivec2(idx / w, idx % w);
}

fn grid_position_to_idx(x: i32, y: i32, w: i32) -> i32 {
    return x + y * w;
}

fn is_within_bounds(x: i32, y: i32, size: i32) -> bool {
    x >= 0 && x < size && y >= 0 && y < size
}

fn generate_square_grid_graph(grid_world_size: i32, grid_cell_size: i32) -> Graph {

    assert!(grid_world_size % grid_cell_size == 0);

    let grid_cell_count = grid_world_size / grid_cell_size;
    let number_of_grid_cells = grid_cell_count * grid_cell_count;

    let mut nodes = Vec::new();
    let mut edges = Vec::new();
    let mut node_data = HashMap::new();
    let mut edge_data = HashMap::new();
    
    for idx in 0 .. number_of_grid_cells {
        nodes.push(Vertex(idx));
        let vertex_position = idx_to_grid_position(idx, grid_cell_count) * grid_cell_size;
        node_data.insert(Vertex(idx), VertexData { position: vec2(vertex_position.x as f32, vertex_position.y as f32), is_place: false });
    }

    for x in 0..grid_cell_count {
        for y in 0..grid_cell_count {

            let top = ivec2(x, y - 1);
            let right = ivec2(x + 1, y);
            let bottom = ivec2(x, y + 1);
            let left = ivec2(x - 1, y);

            if is_within_bounds(top.x, top.y, grid_cell_count) {
                let source_idx = grid_position_to_idx(x, y, grid_cell_count);
                let target_idx = grid_position_to_idx(top.x, top.y, grid_cell_count);
                edges.push(Edge { source: Vertex(source_idx), target: Vertex(target_idx) });
                edge_data.insert((Vertex(source_idx), Vertex(target_idx)), EdgeData { is_place_connection: false });
            }

            if is_within_bounds(right.x, right.y, grid_cell_count) {
                let source_idx = grid_position_to_idx(x, y, grid_cell_count);
                let target_idx = grid_position_to_idx(right.x, right.y, grid_cell_count);
                edges.push(Edge { source: Vertex(source_idx), target: Vertex(target_idx) });
                edge_data.insert((Vertex(source_idx), Vertex(target_idx)), EdgeData { is_place_connection: false });
            }

            if is_within_bounds(bottom.x, bottom.y, grid_cell_count) {
                let source_idx = grid_position_to_idx(x, y, grid_cell_count);
                let target_idx = grid_position_to_idx(bottom.x, bottom.y, grid_cell_count);
                edges.push(Edge { source: Vertex(source_idx), target: Vertex(target_idx) });
                edge_data.insert((Vertex(source_idx), Vertex(target_idx)), EdgeData { is_place_connection: false });
            }

            if is_within_bounds(left.x, left.y, grid_cell_count) {
                let source_idx = grid_position_to_idx(x, y, grid_cell_count);
                let target_idx = grid_position_to_idx(left.x, left.y, grid_cell_count);
                edges.push(Edge { source: Vertex(source_idx), target: Vertex(target_idx) });
                edge_data.insert((Vertex(source_idx), Vertex(target_idx)), EdgeData { is_place_connection: false });
            }

        }
    }

    Graph { nodes, edges, node_data: node_data, edge_data: edge_data }

}

fn generate_random_places_on_grid(state: &mut WorldState) {

    let min_number_of_random_nodes = 4;
    let max_number_of_random_nodes = 12;
    let number_of_random_nodes = gen_range(min_number_of_random_nodes, max_number_of_random_nodes);

    for _ in 0..number_of_random_nodes {
        let random_vertex = state.graph.nodes[gen_range(0, state.graph.nodes.len())];
        let vertex_data = state.graph.node_data.get_mut(&random_vertex).unwrap();
        vertex_data.is_place = true;
    }

}

fn find_closest_visited_place_and_unvisited_place(visited_places: &Vec<Vertex>, unvisited_places: &Vec<Vertex>, get_position_fn: impl Fn(Vertex) -> Vec2) -> (usize, usize) {

    let mut current_min_visited_place_idx = 0;
    let mut current_min_unvisited_place_idx = 0;
    let mut current_min_distance = f32::MAX;

    for (visited_idx, visited_place) in visited_places.iter().enumerate() {

        let a = get_position_fn(*visited_place);

        for (unvisited_idx, unvisited_place) in unvisited_places.iter().enumerate() {

            let b = get_position_fn(*unvisited_place);
            let d = a.distance(b);

            if d < current_min_distance {
                current_min_unvisited_place_idx = unvisited_idx;
                current_min_visited_place_idx = visited_idx;
                current_min_distance = d;
            }

        }

    }

    (current_min_visited_place_idx, current_min_unvisited_place_idx)

}

fn reconstruct_path(came_from: &HashMap<Vertex, Vertex>, mut current: Vertex) -> Vec<Vertex> {
    let mut total_path = vec![current];
    while came_from.contains_key(&current) {
        current = came_from[&current];
        total_path.insert(0, current);
    }
    total_path
}

fn astar(source: Vertex, target: Vertex, graph: &Graph) -> Vec<Vertex> {

    let h = |n: Vertex| graph.node_data[&n].position.distance(graph.node_data[&target].position);

    let mut open_set: HashSet<Vertex> = HashSet::new();
    open_set.insert(source);

    let mut came_from: HashMap<Vertex, Vertex> = HashMap::new();
    let mut g_score: HashMap<Vertex, f32> = HashMap::new();
    let mut f_score: HashMap<Vertex, f32> = HashMap::new();

    g_score.insert(source, 0.0);
    f_score.insert(source, h(source));

    while open_set.len() > 0 {

        // #NOTE: this is absolutel beyond heinous, but the point of this example is not an efficient A* implementation, it is demonstrating steiner trees on graphs
        let current: Vertex = *open_set.iter().min_by(|a, b| f_score.get(&a).unwrap_or(&f32::MAX).partial_cmp(&f_score.get(&b).unwrap_or(&f32::MAX)).unwrap()).unwrap();
        if current == target {
            return reconstruct_path(&came_from, current);
        }

        open_set.remove(&current);

        for neighbour in graph.edges.iter().filter(|e| e.source == current).map(|e| e.target) {
            let tentative_g_score = *g_score.get(&current).unwrap_or(&f32::MAX);
            if tentative_g_score < *g_score.get(&neighbour).unwrap_or(&f32::MAX) {
                came_from.insert(neighbour, current);
                g_score.insert(neighbour, tentative_g_score);
                f_score.insert(neighbour, tentative_g_score + h(neighbour));
                if open_set.contains(&neighbour) == false {
                    open_set.insert(neighbour);
                }
            }
        }

    }

    Vec::new()

}

fn connect_random_places_on_grid(state: &mut WorldState) {

    let all_unvisited_places: Vec<Vertex> = state.graph.nodes.iter().filter(|n| state.graph.node_data[n].is_place).map(|v| *v).collect();
    let mut unvisited_places = all_unvisited_places;

    if unvisited_places.len() < 2 {
        return;
    }

    let random_first_visited_place = unvisited_places.remove(gen_range(0, unvisited_places.len()));
    let mut visited_places = vec![random_first_visited_place];

    while unvisited_places.len() > 0 {

        let (visited_place_idx, unvisited_place_idx) = find_closest_visited_place_and_unvisited_place(&visited_places, &unvisited_places, |v| state.graph.node_data[&v].position);

        let visited_place = visited_places[visited_place_idx];
        let unvisited_place = unvisited_places[unvisited_place_idx];

        unvisited_places.remove(unvisited_place_idx);
        visited_places.push(unvisited_place);

        let path = astar(visited_place, unvisited_place, &state.graph);

        for i in 0..path.len() {
            let current_node = path[i];
            if i + 1 < path.len() {
                let next_node = path[i + 1];
                let edge_data = state.graph.edge_data.get_mut(&(current_node, next_node)).unwrap();
                edge_data.is_place_connection = true;
            }
        }

    }

}

fn rebuild_world_state(state: &mut WorldState) {

    let grid_cell_size = 32;
    let grid_world_size = 512;
    state.graph = generate_square_grid_graph(grid_world_size, grid_cell_size);

}

fn draw_graph(graph: &Graph) {

    let vertex_radius = 4.0;
    let edge_thickness = 2.0;

    let place_vertex_thickness_multiplier = 2.0;
    let path_edge_thickness_multiplier = 4.0;

    for vertex in &graph.nodes {
        let data = &graph.node_data[vertex];
        let position = data.position;
        
        if data.is_place {
            draw_circle(position.x, position.y, vertex_radius * place_vertex_thickness_multiplier, BLACK);
        } else {
            draw_circle(position.x, position.y, vertex_radius, BLACK);
        }
    }

    for edge in &graph.edges {
        let pos_source = &graph.node_data[&edge.source].position;
        let pos_target = &graph.node_data[&edge.target].position;
        let data = &graph.edge_data[&(edge.source, edge.target)];

        if data.is_place_connection {
            draw_line(pos_source.x, pos_source.y, pos_target.x, pos_target.y, edge_thickness * path_edge_thickness_multiplier, BLACK);
        } else {
            draw_line(pos_source.x, pos_source.y, pos_target.x, pos_target.y, edge_thickness, BLACK);
        }
    }

}

fn draw_world_state(state: &WorldState) {
    draw_graph(&state.graph);
}

fn window_conf() -> Conf {
    Conf {
        miniquad_conf: miniquad::conf::Conf {
            window_title: "steiner tree problem".to_owned(),
            sample_count: 4,
            ..Default::default()
        },
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {

    let mut should_rebuild: bool = true;
    let mut should_place_random_nodes: bool = false;
    let mut should_create_path_between_random_nodes: bool = false;

    let mut world_state = WorldState::new();
    let mut debug_text = DebugText::new();
    let mut camera = GameCamera2D::new();

    loop {

        let dt = get_frame_time();

        debug_text.new_frame();
        clear_background(WHITE);

        camera.push();
        
        if should_rebuild {
            rebuild_world_state(&mut world_state);
            should_rebuild = false;
        }

        if should_place_random_nodes {
            generate_random_places_on_grid(&mut world_state);
            should_place_random_nodes = false;
        }

        if should_create_path_between_random_nodes {
            connect_random_places_on_grid(&mut world_state);
            should_create_path_between_random_nodes = false;
        }

        if is_key_pressed(KeyCode::R) {
            should_rebuild = true;
        }

        if is_key_pressed(KeyCode::G) {
            should_place_random_nodes = true;
        }
        
        if is_key_pressed(KeyCode::C) {
            should_create_path_between_random_nodes = true;
        }

        draw_world_state(&world_state);

        camera.pop();
        camera.tick(dt);

        debug_text.draw_text("steiner tree problem", utility::TextPosition::TopLeft, BLACK);
        debug_text.draw_text(format!("current graph - nodes: {}, edges: {}", world_state.graph.nodes.len(), world_state.graph.edges.len()), utility::TextPosition::TopLeft, BLACK);
        debug_text.draw_text(" - press R to recreate the graph", utility::TextPosition::TopLeft, BLACK);
        debug_text.draw_text(" - press G to generate new random places on the graph", utility::TextPosition::TopLeft, BLACK);
        debug_text.draw_text(" - press C to connect the new random places on the graph", utility::TextPosition::TopLeft, BLACK);
        next_frame().await;
    }
}
