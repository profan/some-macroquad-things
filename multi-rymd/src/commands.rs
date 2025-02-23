use lockstep_client::step::LockstepClient;
use nanoserde::{DeJson, SerJson};


#[derive(Debug, SerJson, DeJson)]
pub enum GameCommand {
    Message { text: String },
    JoinTeam { team_id: i32 },
    LeaveTeam
}

pub trait CommandsExt {
    fn send_chat_message(&mut self, message: String);
    fn send_join_team_message(&mut self, team_id: i32);
    fn send_leave_team_message(&mut self);
}

impl CommandsExt for LockstepClient {

    fn send_chat_message(&mut self, message: String) {
        let game_command = GameCommand::Message { text: message };
        self.send_generic_message(&game_command.serialize_json());
    }

    fn send_join_team_message(&mut self, team_id: i32) {
        let join_team_message = GameCommand::JoinTeam { team_id };
        self.send_generic_message(&join_team_message.serialize_json());
    }

    fn send_leave_team_message(&mut self) {
        let leave_team_message = GameCommand::LeaveTeam;
        self.send_generic_message(&leave_team_message.serialize_json());
    }
    
}