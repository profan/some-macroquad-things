use std::{collections::HashMap, f32::consts::PI};
use macroquad::{prelude::*, ui::{hash, root_ui}};
use utility::{wrap, AdjustHue, AsAngle, GameCamera2D, RotatedBy};

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

fn draw_l_system_as_tree(l_system: &LSystem, draw_forward_characters: &str, angle: f32, length: f32) {

    let w = screen_width();
    let h = screen_height();

    let angle_radians = angle.to_radians();

    let thickness = 1.0;
    let mut position = vec2(w / 2.0, h);
    let mut last_position = position;
    let mut rotation = vec2(0.0, length);
    let mut color = PINK;

    let mut stack = Vec::new();

    for c in l_system.state.chars() {

        match c {
            v if draw_forward_characters.contains(v) => {
                position = position + rotation;
                color.r += (1.0 / 255.0) * rotation.normalize().as_angle() % PI / 2.0;
            },
            '-' => {
                rotation = rotation.rotated_by(-angle_radians);
                color.g += (1.0 / 255.0) * rotation.normalize().as_angle() % PI / 2.0;
            },
            '+' => {
                rotation = rotation.rotated_by(angle_radians);
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
    let mut camera = GameCamera2D::new();

    let mut current_branch_bend_angle = 60.0;
    let mut current_branch_length = 2.0;

    let mut current_initial_state = "F".to_string();
    let mut current_draw_forward_characters = "FG".to_string();
    let mut current_number_of_steps = 8;

    let mut current_number_of_steps_str = current_number_of_steps.to_string();
    let mut current_branch_bend_angle_str = current_branch_bend_angle.to_string();
    let mut current_branch_length_str = current_branch_length.to_string();
    let mut current_rules: Vec<String> = vec![
        "F=F-[G+F+G]-F".to_string(),
        "G=GG".to_string()
    ];


    let mut current_branch_bend_angle_start = current_branch_bend_angle;
    let mut current_branch_bend_angle_end = current_branch_bend_angle;
    let mut current_branch_bend_interpolation_speed = 10.0;
    let mut current_branch_bend_interpolation_up = true;

    let mut current_branch_bend_angle_start_str = current_branch_bend_angle.to_string();
    let mut current_branch_bend_angle_end_str = current_branch_bend_angle.to_string();
    let mut current_branch_bend_interpolation_speed_str = current_branch_bend_interpolation_speed.to_string();

    let mut current_branch_branch_length_start = current_branch_length;
    let mut current_branch_branch_length_end = current_branch_length;
    let mut current_branch_length_interpolation_speed = 1.0;
    let mut current_branch_length_interpolation_up = true;

    let mut current_branch_branch_length_start_str = current_branch_length.to_string();
    let mut current_branch_branch_length_end_str = current_branch_length.to_string();
    let mut current_branch_length_interpolation_speed_str = current_branch_length_interpolation_speed.to_string();

    let mut should_interpolate_current_bend_angle = false;
    let mut should_interpolate_current_branch_length = false;

    // allow toggling the ui to make it a nice screensaver thing?
    let mut should_render_debug_ui = true;

    // will re-evaluate the tree on next go around
    let mut should_reevaluate = true;

    loop {

        let dt = get_frame_time();
        clear_background(BLACK.lighten(0.1));

        if is_key_pressed(KeyCode::K) {
            should_render_debug_ui = !should_render_debug_ui;
        }

        if should_render_debug_ui {

            let window_height = 280.0 + current_rules.len() as f32 * 16.0;
            root_ui().window(hash!(), vec2(32.0, 64.0), vec2(384.0, window_height), |w| {

                w.label(None, "initial system state");
                w.input_text(hash!(), "state", &mut current_initial_state);
                w.input_text(hash!(), "draw forward characters", &mut current_draw_forward_characters);
                w.input_text(hash!(), "number of steps", &mut current_number_of_steps_str);
                current_number_of_steps = current_number_of_steps_str.parse().unwrap_or(6);

                w.label(None, "tree parameters");

                if should_interpolate_current_bend_angle == false {
                    w.input_text(hash!(), "branch bend angle", &mut current_branch_bend_angle_str);
                    current_branch_bend_angle = current_branch_bend_angle_str.parse().unwrap_or(current_branch_bend_angle);
                }

                // let mut last_should_interpolate_bend_angle_state = should_interpolate_current_bend_angle;
                w.checkbox(hash!(), "interpolate bend angle?", &mut should_interpolate_current_bend_angle);
                // let should_interpolate_bend_angle_state_changed = should_interpolate_current_bend_angle != last_should_interpolate_bend_angle_state;

                if should_interpolate_current_bend_angle {
                    w.input_text(hash!(), "branch bend angle start", &mut current_branch_bend_angle_start_str);
                    w.input_text(hash!(), "branch bend angle end", &mut current_branch_bend_angle_end_str);
                    w.input_text(hash!(), "branch bend interpolation speed", &mut current_branch_bend_interpolation_speed_str);

                    // if should_interpolate_bend_angle_state_changed {
                    //     current_branch_bend_angle = current_branch_bend_angle_start
                    // }

                    current_branch_bend_angle_start = current_branch_bend_angle_start_str.parse().unwrap_or(current_branch_bend_angle_start);
                    current_branch_bend_angle_end = current_branch_bend_angle_end_str.parse().unwrap_or(current_branch_bend_angle_end);
                    current_branch_bend_interpolation_speed = current_branch_bend_interpolation_speed_str.parse().unwrap_or(current_branch_bend_interpolation_speed);
                }

                if should_interpolate_current_branch_length == false {
                    w.input_text(hash!(), "branch length", &mut current_branch_length_str);
                    current_branch_length = current_branch_length_str.parse().unwrap_or(current_branch_length);
                }

                w.checkbox(hash!(), "interpolate branch length?", &mut should_interpolate_current_branch_length);
                if should_interpolate_current_branch_length {
                    w.input_text(hash!(), "branch branch length start", &mut current_branch_branch_length_start_str);
                    w.input_text(hash!(), "branch branch length end", &mut current_branch_branch_length_end_str);
                    w.input_text(hash!(), "branch length interpolation speed", &mut current_branch_length_interpolation_speed_str);

                    current_branch_branch_length_start = current_branch_branch_length_start_str.parse().unwrap_or(current_branch_branch_length_start);
                    current_branch_branch_length_end = current_branch_branch_length_end_str.parse().unwrap_or(current_branch_branch_length_end);
                    current_branch_length_interpolation_speed = current_branch_length_interpolation_speed_str.parse().unwrap_or(current_branch_length_interpolation_speed);
                }

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

            });

        }

        camera.push();

        let angle_interpolation_direction = if current_branch_bend_interpolation_up { 1.0 } else { -1.0 };
        let length_interpolation_direction = if current_branch_length_interpolation_up { 1.0 } else { -1.0 };

        if should_interpolate_current_bend_angle {

            let last_branch_bend_angle_end: f32 = current_branch_bend_angle_end;
            current_branch_bend_angle_end = current_branch_bend_angle_end.max(current_branch_bend_angle_start);
            current_branch_bend_angle_start = current_branch_bend_angle_start.min(last_branch_bend_angle_end);

            current_branch_bend_angle = (current_branch_bend_angle + angle_interpolation_direction * current_branch_bend_interpolation_speed * dt).clamp(current_branch_bend_angle_start, current_branch_bend_angle_end);
            if current_branch_bend_angle <= current_branch_bend_angle_start {
                current_branch_bend_interpolation_up = true
            } else if current_branch_bend_angle >= current_branch_bend_angle_end {
                current_branch_bend_interpolation_up = false;
            }
        }

        if should_interpolate_current_branch_length {

            let last_branch_length_angle_end: f32 = current_branch_branch_length_end;
            current_branch_branch_length_end = current_branch_branch_length_end.max(current_branch_branch_length_start);
            current_branch_branch_length_start = current_branch_branch_length_start.min(last_branch_length_angle_end);
            
            current_branch_length = (current_branch_length + length_interpolation_direction * current_branch_length_interpolation_speed * dt).clamp(current_branch_branch_length_start, current_branch_branch_length_end);
            if current_branch_length <= current_branch_branch_length_start {
                current_branch_length_interpolation_up = true
            } else if current_branch_length >= current_branch_branch_length_end {
                current_branch_length_interpolation_up = false;
            }
        }

        draw_l_system_as_tree(&l_system, &current_draw_forward_characters, current_branch_bend_angle, -current_branch_length);
        camera.pop();
        camera.tick(dt);
        
        next_frame().await;

    }
    
}
