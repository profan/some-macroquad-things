use hecs::{World, Entity};
use lockstep_client::step::PeerID;

use super::{Metal, Energy};

#[derive(Debug, Clone)]
pub struct Player {
    pub id: PeerID
}

pub fn create_player_entity(world: &mut World, id: PeerID) -> Entity {

    let default_metal = 1000.0;
    let default_energy = 1000.0;

    let default_metal_pool_size = 1000.0;
    let default_energy_pool_size = 1000.0;

    let metal = Metal { current: default_metal, income: 0.0, base_size: default_metal_pool_size, pool_size: 0.0 };
    let energy = Energy { current: default_energy, income: 0.0, base_size: default_energy_pool_size, pool_size: 0.0 };
    let player = Player { id };

    let new_player = world.spawn((player, metal, energy));
    new_player

}