use lockstep_client::{game::Game, step::LockstepClient, app::ApplicationState};
use macroquad::prelude::*;
use utility::DebugText;

struct RymdGame {
    is_running: bool
}

impl Game for RymdGame {

    fn is_running(&self) -> bool {
        self.is_running
    }

    fn start_game(&mut self) {
        self.is_running = true;
    }

    fn pause_game(&mut self) {
        self.is_running = false;
    }

    fn stop_game(&mut self) {
        self.is_running = false;
    }

    fn handle_message(&mut self, message: &str) {    
    }

    fn update(&mut self, debug: &mut DebugText, lockstep: &mut LockstepClient) {     
    }

    fn tick(&mut self, debug: &mut DebugText) {
    }

    fn draw(&mut self, debug: &mut DebugText) {
    }

    fn reset(&mut self) {
        self.stop_game();
    }

}

impl RymdGame {
    fn new() -> RymdGame {
        RymdGame { is_running: false }
    }
}

#[macroquad::main("multi-rymd")]
async fn main() {

    let mut app = ApplicationState::new("multi-rymd", Box::new(RymdGame::new()));

    loop {

        app.handle_messages();
        clear_background(WHITE);

        app.update();
        app.draw();

        next_frame().await;

    }

}