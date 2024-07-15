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

pub struct HexBoundary {
    pub vertices: Vec<Hex>,
    pub edges: Vec<Hex>
}

impl HexBoundary {

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
        
        // println!("current standing hex: {:?}, current hand hex: {:?}", current_standing_hex, current_hand_hex);
        // println!("start standing hex: {:?}, start hand hex: {:?} \n", start_standing_hex, start_hand_hex);

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