#![feature(async_fn_in_trait)]

use lockstep_client::{app::ApplicationState, step::LockstepClient, step::PeerID, game::Game};
use nanoserde::{SerJson, DeJson};
use macroquad::prelude::*;
use utility::DebugText;

#[derive(Debug, SerJson, DeJson)]
enum GameMessage {
    SpawnCircle { x: i32, y: i32 }
}

struct Circle {
    position: Vec2
}

impl Circle {
    pub fn new(x: i32, y: i32) -> Circle {
        Circle {
            position: vec2(x as f32, y as f32)
        }
    }
}

pub struct ExampleGame {
    circles: Vec<Circle>,
    is_running: bool
}

impl ExampleGame {

    pub fn new() -> ExampleGame {
        ExampleGame {
            circles: Vec::new(),
            is_running: false
        }
    }

}

impl Game for ExampleGame {

    fn is_running(&self) -> bool {
        self.is_running
    }

    fn start_game(&mut self) {
        self.is_running = true;
    }

    fn stop_game(&mut self) {
        self.is_running = false;
    }

    fn pause_game(&mut self) {
        self.is_running = false;
    }

    fn handle_message(&mut self, peer_id: PeerID, message: &str) {

        let msg = match GameMessage::deserialize_json(message) {
            Ok(msg) => msg,
            Err(err) => {
                println!("GameState: failed to deserialize message: {}, with error: {}", message, err);
                return;
            }
        };

        match msg {
            GameMessage::SpawnCircle { x, y } => {
                self.circles.push(Circle::new(x, y));
                println!("spawned circle at: ({}, {})!", x, y);
            }
        };
        
    }

    fn update(&mut self, debug: &mut DebugText, lockstep: &mut LockstepClient) {

        if is_mouse_button_pressed(MouseButton::Left) {
            send_spawn_circle_message(lockstep);
        }

        if self.is_running {
            self.tick(debug);
        } else {
            panic!("Game - update called when not supposed to be running, should be impossible?");
        }

    }

    fn tick(&mut self, debug: &mut DebugText) {

    }

    fn draw(&mut self, debug: &mut DebugText) {

        let circle_radius = 32.0;

        for c in &self.circles {
            draw_circle(c.position.x, c.position.y, circle_radius, RED);
        }

    }

    fn reset(&mut self) {
        self.is_running = false;
        self.circles.clear();
    }

    async fn load_resources(&mut self) {

    }

}

fn send_spawn_circle_message(lockstep: &mut LockstepClient) {
    let mouse_position: Vec2 = mouse_position().into();
    let spawn_circle_message = GameMessage::SpawnCircle { x: mouse_position.x as i32, y: mouse_position.y as i32 };
    lockstep.send_command(spawn_circle_message.serialize_json());
}

#[macroquad::main("lockstep-example-client")]
async fn main() {

    let mut app = ApplicationState::new("lockstep-example-client", ExampleGame::new());
    app.load_resources().await;

    loop {

        app.handle_messages();
        clear_background(WHITE);

        app.update();
        app.draw();

        next_frame().await;
        
    }

}
