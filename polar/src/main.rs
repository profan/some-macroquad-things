#![feature(clamp)]
use macroquad::prelude::*;
use std::cmp::Ordering;
use std::f32::consts::*;

#[derive(Default, Debug, Clone)]
struct Particle {
    position: Vec2,
    velocity: Vec2
}

#[derive(Debug)]
struct State {

    current_mode: i32,

    current_particle_count: usize,
    new_particle_count: usize,

    particles: Vec<Particle>

}

impl State {

    fn should_render_points(&self) -> bool {
        self.current_mode > 0
    }

    fn should_render_lines(&self) -> bool {
        self.current_mode > 1
    }

    fn should_render_ids(&self) -> bool {
        self.current_mode > 2
    }

}

// min distance from edge on either axis when spawned
const PARTICLE_EDGE_BOUNDS: f32 = 64.0;
const MAX_VELOCITY: f32  = 32.0;

fn cartesian_to_polar(p: Vec2) -> (f32, f32) {
    let r = p.x()*p.x() + p.y()*p.y();
    let phi = p.y().atan2(p.x());
    (r, phi)
}

#[macroquad::main("polar")]
async fn main() {

    let mut state = State {

        current_mode: 1,

        current_particle_count: 0,
        new_particle_count: 32,
        particles: Vec::new()

    };
    
    fn update_particle_state(state: &mut State, dt: f32) {

        let screen_w = screen_width();
        let screen_h = screen_height();

        // handle particle count changing and such
        if state.new_particle_count != state.current_particle_count {

            // add this many more particles
            if state.new_particle_count > state.current_particle_count {

                for _ in state.current_particle_count .. state.new_particle_count {
                    let r_x = rand::gen_range(PARTICLE_EDGE_BOUNDS, screen_w - PARTICLE_EDGE_BOUNDS);
                    let r_y = rand::gen_range(PARTICLE_EDGE_BOUNDS, screen_h - PARTICLE_EDGE_BOUNDS);
                    let r_vx = rand::gen_range(-MAX_VELOCITY, MAX_VELOCITY);
                    let r_vy = rand::gen_range(-MAX_VELOCITY, MAX_VELOCITY);
                    state.particles.push(Particle {
                        position: vec2(r_x, r_y),
                        velocity: vec2(r_vx, r_vy)
                    });
                }

                state.current_particle_count = state.new_particle_count;

            } else {
                
                state.particles.resize(state.new_particle_count, Particle::default());
                state.current_particle_count = state.new_particle_count;

            }

        }

        // integrate particle movement
        for particle in &mut state.particles {

            particle.position += particle.velocity * dt;
            
            // handle out of bounds

            if particle.position.x() < 0.0 || particle.position.x() > screen_w {
                if particle.velocity.x() > 0.0 && particle.position.x() > 0.0 {
                    *particle.velocity.x_mut() *= -1.0;
                } else if particle.velocity.x() < 0.0 && particle.position.x() < 0.0 {
                    *particle.velocity.x_mut() *= -1.0;
                }
            }

            if particle.position.y() < 0.0 || particle.position.y() > screen_h {
                if particle.velocity.y() > 0.0 && particle.position.y() > 0.0 {
                    *particle.velocity.y_mut() *= -1.0;
                } else if particle.velocity.y() < 0.0 && particle.position.y() < 0.0 {
                    *particle.velocity.y_mut() *= -1.0;
                }
            }

        }

        {

            // compute centroid
            let centroid = state.particles.iter().fold(Vec2::zero(), |acc, x| acc + x.position) / state.particles.len() as f32;

            // sort our particles by polar coordinates so we can draw a linee
            let sort_by_polar_coordinates = |a: &Particle, b: &Particle| -> Ordering {
                let (_a_r, a_phi) = cartesian_to_polar(a.position - centroid);
                let (_b_r, b_phi) = cartesian_to_polar(b.position - centroid);
                ((a_phi * 180.0/PI) as i32).cmp(&((b_phi * 180.0/PI) as i32)) // this is a hack, because we want a total ordering
            };

            state.particles.sort_by(sort_by_polar_coordinates);
        }

    }

    fn render_particle_state(state: &State) {

        if state.should_render_lines() {

            for slice in state.particles.windows(2) {
                match &slice {
                    &[a, b] => {
                        draw_line(
                            a.position.x(), a.position.y(),
                            b.position.x(), b.position.y(),
                            1.0, WHITE
                        )
                    },
                    _ => {}
                }
            }
    
            // draw end cap for line
            let first = &state.particles.first().unwrap();
            let last = &state.particles.last().unwrap();
            draw_line(
                last.position.x(), last.position.y(),
                first.position.x(), first.position.y(),
                1.0, WHITE
            );

        }

        if state.should_render_points() {

            for particle in &state.particles {
                let pos = particle.position;
                draw_circle(pos.x(), pos.y(), 1.0, WHITE);
            }

        }

        if state.should_render_ids() {

            for (idx, particle) in state.particles.iter().enumerate() {
                let pos = particle.position;
                draw_text(idx.to_string().as_str(), pos.x(), pos.y(), 16.0, WHITE);
            }

        }

    }

    loop {

        let dt = get_frame_time();

        if is_key_pressed(KeyCode::Tab) {
            state.current_mode = (state.current_mode + 1) % 4;
        }

        let (_wheel_x, wheel_y) = mouse_wheel();
        let clamped_new_count = (state.new_particle_count as i32 + wheel_y as i32).clamp(1, 1000);
        state.new_particle_count = clamped_new_count as usize;

        update_particle_state(&mut state, dt);
        render_particle_state(&state);

        draw_text(
            format!("tab to change mode, current: {}", state.current_mode).as_str(),
            32.0, 32.0,
            16.0, WHITE
        );

        draw_text(
            format!("current particle count (scroll): {}", state.current_particle_count).as_str(),
            32.0, 48.0,
            16.0, WHITE
        );

        next_frame().await

    }

}