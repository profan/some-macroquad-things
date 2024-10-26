use lockstep_client::step::LockstepClient;

pub trait CommandsExt {
    fn send_chat_message(&mut self, message: String);
}

impl CommandsExt for LockstepClient {
    fn send_chat_message(&mut self, message: String) {
        todo!()
    }
}