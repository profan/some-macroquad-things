#![allow(dead_code)]
#![allow(unused_variables)]
#![feature(let_chains)]

use game::RymdGame;
use model::Transform;
use macroquad::prelude::*;
use hecs::*;

use utility::{Kinematic, SteeringOutput, AsVector};
use lockstep_client::app::ApplicationState;

mod game;
mod model;
mod utils;
mod view;

use u64 as EntityID;

fn ship_apply_steering(kinematic: &mut Kinematic, steering_maybe: Option<SteeringOutput>, dt: f32) {

    let turn_rate = 4.0;
    if let Some(steering) = steering_maybe {

        let desired_linear_velocity = steering.linear * dt;

        // project our desired velocity along where we're currently pointing first
        let projected_linear_velocity = desired_linear_velocity * desired_linear_velocity.dot(-kinematic.orientation.as_vector()).max(0.0);
        kinematic.velocity += projected_linear_velocity;

        let turn_delta = steering.angular * turn_rate * dt;
        kinematic.angular_velocity += turn_delta;

    }

}

pub fn get_entity_position(world: &World, entity_id: u64) -> Option<Vec2> {
    world.get::<&Transform>(Entity::from_bits(entity_id).unwrap()).and_then(|t| Ok(t.world_position)).or(Err(())).ok()
}

#[macroquad::main("multi-rymd")]
async fn main() {

    let mut app = ApplicationState::new("multi-rymd", RymdGame::new());

    app.set_target_host("94.13.52.142");
    app.set_debug_text_colour(WHITE);
    app.load_resources().await;

    loop {

        app.handle_messages();
        clear_background(Color::from_hex(0x181425));

        app.update();
        app.draw();

        next_frame().await;

    }

}