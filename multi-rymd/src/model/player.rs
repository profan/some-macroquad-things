use hecs::{World, Entity};
use lockstep_client::step::PeerID;

use crate::PlayerID;

use super::{Metal, Energy};

#[derive(Debug, Clone)]
pub struct Player {
    pub id: PeerID,
    pub team_mask: u64
}

pub fn create_player_entity(world: &mut World, id: PeerID) -> Entity {

    let default_metal = 1000.0;
    let default_energy = 1000.0;

    let default_metal_pool_size = 1000.0;
    let default_energy_pool_size = 1000.0;

    let metal = Metal { current: default_metal, income: 0.0, base_size: default_metal_pool_size, pool_size: 0.0 };
    let energy = Energy { current: default_energy, income: 0.0, base_size: default_energy_pool_size, pool_size: 0.0 };
    let player = Player { id, team_mask: 0 };

    
    world.spawn((player, metal, energy))

}

pub fn set_default_metal_pool_size(world: &mut World, pool_amount: i32, pool_size: i32) {

    for (e, (player, metal)) in world.query_mut::<(&Player, &mut Metal)>() {
        metal.current = pool_amount as f32;
        metal.base_size = pool_size as f32;
    }

}

pub fn set_default_energy_pool_size(world: &mut World, pool_amount: i32, pool_size: i32) {
    
    for (e, (player, energy)) in world.query_mut::<(&Player, &mut Energy)>() {
        energy.current = pool_amount as f32;
        energy.base_size = pool_size as f32;
    }

}

pub fn are_players_allied(player_a: &Player, player_b: &Player) -> bool {
    player_a.team_mask & player_b.team_mask != 0
}

pub fn are_players_hostile(player_a: &Player, player_b: &Player) -> bool {
    are_players_allied(player_a, player_b) == false
}

pub fn set_player_team_allegiance(world: &mut World, player_id: PlayerID, allegiance: u64) {

    let (e, player) = world.query_mut::<&mut Player>()
        .into_iter()
        .find(|(e, p)| p.id == player_id)
        .unwrap_or_else(|| panic!("player with id: {} didn't exist? this is fatal!", player_id));

    player.team_mask = allegiance

}