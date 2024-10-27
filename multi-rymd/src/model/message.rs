use nanoserde::{SerJson, DeJson};

use crate::EntityID;
use crate::model::GameOrder;

#[derive(Debug, SerJson, DeJson)]
pub enum GameMessage {
    Order { entities: Vec<EntityID>, order: GameOrder, add: bool },
}