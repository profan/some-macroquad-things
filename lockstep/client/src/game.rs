use utility::DebugText;
use crate::step::LockstepClient;

pub trait Game {

    fn is_running(&self) -> bool;
    fn start_game(&mut self);
    fn pause_game(&mut self);
    fn stop_game(&mut self);

    fn handle_message(&mut self, message: &str);
    fn update(&mut self, debug: &mut DebugText, lockstep: &mut LockstepClient);
    fn tick(&mut self, debug: &mut DebugText);
    fn draw(&mut self, debug: &mut DebugText);
    fn reset(&mut self);
    
}