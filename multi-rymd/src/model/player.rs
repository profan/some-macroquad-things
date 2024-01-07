use hecs::{World, Entity};
use lockstep_client::step::PeerID;

use super::{Metal, Energy};

#[derive(Debug, Clone)]
pub struct Player {
    pub id: PeerID
}

pub fn create_player_entity(world: &mut World, id: PeerID) -> Entity {

    let default_metal = 100;
    let default_energy = 100;

    let metal = Metal { current: default_metal };
    let energy = Energy { current: default_energy };
    let player = Player { id };

    let new_player = world.spawn((player, metal, energy));
    new_player

}