#![allow(dead_code)]
#![allow(unused_variables)]
#![feature(async_fn_in_trait)]
#![feature(let_chains)]

use model::{RymdGameModel, Transform};
use macroquad::prelude::*;
use hecs::*;

use utility::{DebugText, Kinematic, SteeringOutput, AsVector};
use lockstep_client::{game::Game, step::LockstepClient, step::PeerID, app::ApplicationState};

mod model;
mod utils;
mod view;

use u64 as EntityID;
use view::RymdGameView;

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

#[derive(Debug)]
struct RymdGamePlayer {
    id: PeerID
}

#[derive(Debug)]
struct RymdGameParameters {
    players: Vec<RymdGamePlayer>
}

struct RymdGame {
    model: RymdGameModel,
    view: RymdGameView,
    is_started: bool,
    is_running: bool,
    is_paused: bool
}

impl Game for RymdGame {

    fn is_running(&self) -> bool {
        self.is_running
    }

    fn is_paused(&self) -> bool {
        self.is_paused
    }

    fn start_game(&mut self, lockstep: &LockstepClient) {

        if self.is_started {

            self.is_running = true;
            self.is_paused = false;

        } else {

            let game_parameters = if lockstep.is_singleplayer() {
                RymdGameParameters { players: vec![RymdGamePlayer { id: lockstep.peer_id() }] }
            } else {
                let game_players = lockstep.peers().iter().map(|client| RymdGamePlayer { id: client.id } ).collect();
                RymdGameParameters { players: game_players }
            };

            self.model.start(game_parameters);
            
            self.is_running = true;
            self.is_started = true;
            self.is_paused = false;

        }

    }

    fn resume_game(&mut self) {
        self.is_paused = false;
    }

    fn pause_game(&mut self) {
        self.is_paused = true;
    }

    fn stop_game(&mut self) {
        self.is_running = false;
        self.is_started = false;
        self.is_paused = false;
        self.model.stop()
    }

    fn handle_message(&mut self, peer_id: PeerID, message: &str) {    
        self.model.handle_message(message);
    }

    fn update(&mut self, debug: &mut DebugText, lockstep: &mut LockstepClient) {
        self.model.tick();
        self.view.update(&mut self.model);    
    }

    fn update_view(&mut self, debug: &mut DebugText, lockstep: &mut LockstepClient) {
        self.view.tick(&mut self.model.world, lockstep);
    }

    fn draw(&mut self, debug: &mut DebugText) {
        self.view.draw(&mut self.model.world, debug);
    }

    fn reset(&mut self) {
        self.stop_game();
    }

    async fn load_resources(&mut self) {
        self.view.load_resources().await;
    }

}

impl RymdGame {
    fn new() -> RymdGame {
        RymdGame {
            model: RymdGameModel::new(),
            view: RymdGameView::new(),
            is_running: false,
            is_started: false,
            is_paused: false
        }
    }
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