use std::collections::VecDeque;

use hexx::{EdgeDirection, Hex};

use crate::ClientID;

pub fn first_neighbour_matching(hex: Hex, matching_fn: impl Fn(Hex) -> bool) -> Option<Hex> {

    if matching_fn(hex) == false {
        return None;
    }

    for n in hex.all_neighbors() {
        if matching_fn(n) == false {
            return Some(n);
        }
    }

    return None;

}

/// Returns the first neighbour that is also neighbouring the specific hex, if there is any!
pub fn next_neighbour_also_neighbouring_hex(hex: Hex, neighbour: Hex) -> Option<Hex> {

    for n in hex.all_neighbors() {
        if n.distance_to(neighbour) == 1 {
            return Some(n);
        }
    }

    return None;

}

enum TracingMode {
    RotatingFeet,
    RotatingHand
}

pub enum HexBoundaryDirection {
    Clockwise,
    CounterClockwise
}

pub struct HexBoundary {
    pub vertices: Vec<Hex>,
    pub edges: Vec<Hex>
}

impl HexBoundary {

    pub fn direction(&self) -> HexBoundaryDirection {
        HexBoundaryDirection::Clockwise
    }

    pub fn is_loop(&self) -> bool {

        if self.vertices.is_empty() {
            return false;
        }

        for (idx, v) in self.vertices.iter().enumerate() {
            for (o_idx, o) in self.vertices.iter().enumerate() {
                if idx != o_idx && v == o {
                    return false;
                }
            }
        }

        true

    }

    pub fn inner(&self) -> &Vec<Hex> {
        if self.vertices.len() < self.edges.len() {
            &self.vertices
        } else {
            &self.edges
        }
    }

    pub fn outer(&self) -> &Vec<Hex> {
        if self.vertices.len() > self.edges.len() {
            &self.vertices
        } else {
            &self.edges
        }
    }

    pub fn hex_inside_boundary(&self, matching_fn: impl Fn(Hex) -> bool) -> Option<Hex> {

        if self.inner() == &self.edges {
            for &hex in self.inner() {
                if matching_fn(hex) {
                    return Some(hex);
                }
            }
        } else {
            for hex in self.inner() {
                for n in hex.all_neighbors() {
                    if matching_fn(n) && self.outer().contains(&n) == false {
                        return Some(n);
                    }
                }
            }
        }

        None

    }

}

/// Flood fills outwards from the starting hex given the function describing if a specific hex should be included, and a function that gets called on each iteration, returns the amount of filled hexes.
pub fn flood_fill_hexes<T>(state: &mut T, start_hex: Hex, inside_fn: impl Fn(&T, Hex) -> bool, mut set_fn: impl FnMut(&mut T, Hex) -> ()) -> i32 {

    let mut set_hexes = 0;
    let mut queue: VecDeque<Hex> = VecDeque::new();
    queue.push_back(start_hex);

    while let Some(n) = queue.pop_front() {
        if inside_fn(state, n) {
            set_fn(state, n);
            set_hexes += 1;
            for n in n.all_neighbors() {
                queue.push_back(n);
            }
        }
    }

    set_hexes

}

/// Returns the boundary that is formed by walking around in a clockwise fashion, if there is a boundary.
pub fn trace_hex_boundary(start_hand_hex: Hex, in_boundary_fn: impl Fn(Hex) -> bool) -> Option<HexBoundary> {

    let mut current_hand_hex = start_hand_hex;
    let Some(mut current_standing_hex) = first_neighbour_matching(start_hand_hex, &in_boundary_fn) else {
        return None;
    };

    let start_standing_hex = current_standing_hex;
    let mut current_tracing_mode = TracingMode::RotatingFeet;
    let mut hex_edge_list: Vec<Hex> = vec![current_standing_hex];
    let mut hex_vertex_list: Vec<Hex> = vec![current_hand_hex];
    let mut standing_moved = false;
    let mut hand_moved = false;

    loop {

        match current_tracing_mode {

            TracingMode::RotatingFeet => {

                let new_standing_hex = current_standing_hex.rotate_ccw_around(current_hand_hex, 1);

                if in_boundary_fn(new_standing_hex) == false {
                    current_standing_hex = new_standing_hex;
                    hex_edge_list.push(new_standing_hex);
                    standing_moved = true;
                } else {
                    current_tracing_mode = TracingMode::RotatingHand;
                }

            },

            TracingMode::RotatingHand => {

                let new_hand_hex = current_hand_hex.rotate_cw_around(current_standing_hex, 1);

                if in_boundary_fn(new_hand_hex) {
                    current_hand_hex = new_hand_hex;
                    hex_vertex_list.push(new_hand_hex);
                    hand_moved = true;
                } else {
                    current_tracing_mode = TracingMode::RotatingFeet;
                }

            }

        }

        if current_standing_hex == start_standing_hex && current_hand_hex == start_hand_hex && (standing_moved || hand_moved) {
            break;
        }

    }

    if let Some(last_vertex) = hex_vertex_list.last() {
        if *last_vertex == start_hand_hex {
            hex_vertex_list.pop();
        }
    }

    if let Some(last_edge) = hex_edge_list.last() {
        if *last_edge == start_standing_hex {
            hex_edge_list.pop();
        }
    }

    let hex_boundary = HexBoundary {
        vertices: hex_vertex_list,
        edges: hex_edge_list
    };

    Some(hex_boundary)

}