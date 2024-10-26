use nanoserde::{DeJson, SerJson};

use crate::step::TurnCommand;

#[derive(Clone, Debug, SerJson, DeJson)]
pub enum GenericCommand {
    Message(String)
}

#[derive(Debug, SerJson, DeJson)]
pub enum ApplicationCommand  {

    /// Passed through to the game layer, regardless of whether there's a running session, do not use this to modify game state directly when the game is running!
    GenericCommand(GenericCommand),

    /// Passed through to the game layer but only when there's a running session
    TurnCommand(TurnCommand)

}