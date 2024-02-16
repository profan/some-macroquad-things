use super::{Player, player};

use hecs::World;
use lockstep_client::step::PeerID;

pub trait Resource {
    fn capacity(&self) -> i64;
    fn income(&self) -> i64;
    fn value(&self) -> i64;
    fn need(&self) -> i64;
}

#[derive(Debug, Clone)]
pub struct Cost {
    pub metal: f32,
    pub energy: f32
}

pub struct Energy {
    pub current: f32
}

pub struct Metal {
    pub current: f32
}

/// Attempts to provide this amount of metal to the given player's energy pool.
pub fn provide_metal(player_id: PeerID, world: &World, amount: f32) -> bool {
    consume_metal(player_id, world, -amount)
}

/// Attempts to provide this amount of energy to the given player's energy pool.
pub fn provide_energy(player_id: PeerID, world: &World, amount: f32) -> bool {
    consume_energy(player_id, world, -amount)
}

/// Attempts to consume the specific amount of metal from this player's resources, returns true if successful.
pub fn consume_metal(player_id: PeerID, world: &World, amount: f32) -> bool {

    if let Some((current_player_entity, current_player)) = world.query::<&Player>().iter().filter(|(e, p)| p.id == player_id).nth(0) {
        if let Ok(mut metal) = world.get::<&mut Metal>(current_player_entity) && metal.current >= amount {
            metal.current -= amount;
            true
        } else {
            false
        }
    } else {
        false
    }

}

/// Attempts to consume the specific amount of energy from this player's resources, returns true if successful.
pub fn consume_energy(player_id: PeerID, world: &World, amount: f32) -> bool {

    if let Some((current_player_entity, current_player)) = world.query::<&Player>().iter().filter(|(e, p)| p.id == player_id).nth(0) {
        if let Ok(mut energy) = world.get::<&mut Energy>(current_player_entity) && energy.current >= amount {
            energy.current -= amount;
            true
        } else {
            false
        }
    } else {
        false
    }

}

/// Returns the current amount of metal in the given player's resource pool.
pub fn current_metal(player_id: PeerID, world: &World) -> f32 {

    if let Some((current_player_entity, current_player)) = world.query::<&Player>().iter().filter(|(e, p)| p.id == player_id).nth(0) {
        if let Ok(metal) = world.get::<&Metal>(current_player_entity) {
            metal.current
        } else {
            0.0
        }
    } else {
        0.0
    }

}

/// Returns the current amount of energy in the given player's resource pool.
pub fn current_energy(player_id: PeerID, world: &World) -> f32 {

    if let Some((current_player_entity, current_player)) = world.query::<&Player>().iter().filter(|(e, p)| p.id == player_id).nth(0) {
        if let Ok(energy) = world.get::<&Energy>(current_player_entity) {
            energy.current
        } else {
            0.0
        }
    } else {
        0.0
    }

}