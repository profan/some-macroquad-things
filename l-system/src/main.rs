use std::{collections::HashMap, f32::consts::PI};
use macroquad::{prelude::*, ui::{hash, root_ui}};
use utility::{AdjustHue, AsAngle, RotatedBy};

#[derive(Debug)]
struct LSystem {
    state: String,
    rules: HashMap<String, String>
}

impl LSystem {

    fn new<T: Into<String>>(state: T) -> LSystem {
        LSystem { state: state.into(), rules: HashMap::new() }
    }

    fn add_rule<T: Into<String>>(&mut self, from: T, to: T) {
        self.rules.insert(from.into(), to.into());
    }

    fn nth_step(&mut self, n: i32) {

        if n == 0 {
            return
        }

        let mut new_state = self.state.clone();
        
        for (pattern, replacement) in &self.rules {
            new_state = new_state.replace(pattern, &replacement);
        }

        self.state = new_state;
        self.nth_step(n - 1);

    }

}

fn window_conf() -> Conf {
    Conf {
        window_title: "l-system".to_owned(),
        sample_count: 4,
        ..Default::default()
    }
}

fn draw_l_system_as_tree(l_system: &LSystem, angle: f32, length: f32) {

    let w = screen_width();
    let h = screen_height();

    let thickness = 1.0;
    let mut position = vec2(w / 2.0, h);
    let mut last_position = position;
    let mut rotation = vec2(0.0, length);
    let mut color = PINK;

    let mut stack = Vec::new();

    for c in l_system.state.chars() {

        match c {
            'F' => {
                position = position + rotation;
                color.r += (1.0 / 255.0) * rotation.normalize().as_angle() % PI / 2.0;
            },
            '-' => {
                rotation = rotation.rotated_by(-angle);
                color.g += (1.0 / 255.0) * rotation.normalize().as_angle() % PI / 2.0;
            },
            '+' => {
                rotation = rotation.rotated_by(angle);
                color.b += (1.0 / 255.0) * rotation.normalize().as_angle() % PI / 2.0;
            },
            '[' => {
                stack.push((position, rotation, last_position, color));
            },
            ']' => {
                (position, rotation, last_position, color) = stack.pop().unwrap();
            },
            _ => {
                // HAH, well nothing i guess?
            }
        }

        draw_line(last_position.x, last_position.y, position.x, position.y, thickness, color);
        last_position = position;

    }

}

#[macroquad::main(window_conf)]
async fn main() {

    let mut l_system = LSystem::new("");

    let mut current_branch_bend_angle = 0.45;
    let mut current_branch_length = 1.5;

    let mut current_initial_state = "X".to_string();
    let mut current_number_of_steps = 6;

    let mut current_number_of_steps_str = current_number_of_steps.to_string();
    let mut current_branch_bend_angle_str = current_branch_bend_angle.to_string();
    let mut current_branch_length_str = current_branch_length.to_string();
    let mut current_rules: Vec<String> = vec![
        "X=F-[[X]+X]+F[+FX]-X".to_string(),
        "F=FF".to_string()
    ];

    // will re-evaluate the tree on next go around
    let mut should_reevaluate = true;

    loop {

        clear_background(BLACK.lighten(0.1));

        let window_height = 220.0 + current_rules.len() as f32 * 16.0;
        root_ui().window(hash!(), vec2(32.0, 64.0), vec2(384.0, window_height), |w| {

            w.label(None, "initial system state");
            w.input_text(hash!(), "state", &mut current_initial_state);
            w.input_text(hash!(), "number of steps", &mut current_number_of_steps_str);
            current_number_of_steps = current_number_of_steps_str.parse().unwrap_or(6);

            w.label(None, "tree parameters");
            w.input_text(hash!(), "branch bend angle", &mut current_branch_bend_angle_str);
            w.input_text(hash!(), "branch length", &mut current_branch_length_str);

            w.label(None, "rule parameters");

            let mut rules_to_delete = Vec::new();
            for (idx, ref mut pattern_and_replacement) in current_rules.iter_mut().enumerate() {
                w.input_text(hash!(idx), "pattern = replacement", pattern_and_replacement);
                w.same_line(180.0);
                if w.button(None, "x") {
                    rules_to_delete.push(idx);
                }
            }

            for rule_idx in rules_to_delete {
                current_rules.swap_remove(rule_idx);
            }

            if w.button(None, "add rule") {
                current_rules.push(String::new());
            }

            if w.button(None, "evaluate") || should_reevaluate {

                l_system = LSystem::new(current_initial_state.clone());
                
                for pattern_and_replacement in &current_rules {            
                    let split_elements: Vec<&str> = pattern_and_replacement.split("=").collect();
                    if split_elements.len() == 2 {
                        let pattern = split_elements[0];
                        let replacement = split_elements[1];
                        l_system.add_rule(pattern, replacement);
                    }
                }

                l_system.nth_step(current_number_of_steps);
                should_reevaluate = false;

            }

            current_branch_bend_angle = current_branch_bend_angle_str.parse().unwrap_or(0.45);
            current_branch_length = current_branch_length_str.parse().unwrap_or(1.5);

            if is_key_pressed(KeyCode::Enter) {
                w.clear_input_focus();
            }

        });

        draw_l_system_as_tree(&l_system, current_branch_bend_angle, -current_branch_length);
        next_frame().await;

    }
    
}
