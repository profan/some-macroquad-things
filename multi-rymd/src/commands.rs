use lockstep_client::step::LockstepClient;
use nanoserde::{DeJson, SerJson};


#[derive(Debug, SerJson, DeJson)]
pub enum GameCommand {
    Message { text: String }
}

pub trait CommandsExt {
    fn send_chat_message(&mut self, message: String);
}

impl CommandsExt for LockstepClient {

    fn send_chat_message(&mut self, message: String) {
        let game_command = GameCommand::Message { text: message };
        self.send_generic_message(&game_command.serialize_json());
    }
    
}