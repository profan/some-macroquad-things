use lockstep_client::{game::Game, step::LockstepClient};
use lockstep_client::step::PeerID;
use utility::DebugText;

use crate::model::RymdGameModel;
use crate::view::RymdGameView;

#[derive(Debug)]
pub struct RymdGamePlayer {
    pub id: PeerID
}

#[derive(Debug)]
pub struct RymdGameParameters {
    pub players: Vec<RymdGamePlayer>
}

pub struct RymdGame {
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
    pub fn new() -> RymdGame {
        RymdGame {
            model: RymdGameModel::new(),
            view: RymdGameView::new(),
            is_running: false,
            is_started: false,
            is_paused: false
        }
    }
}