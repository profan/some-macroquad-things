use hecs::World;
use macroquad::math::vec2;

use crate::{game::{RymdGameParameters, RymdGameTeam}, model::{create_asteroid, create_player_entity, spawn_commander_ship, Commander, Controller, Health, Player, RymdGameModel}, PlayerID};

pub fn create_players(model: &mut RymdGameModel, parameters: &RymdGameParameters) {

    for player in &parameters.players {
        create_player_entity(&mut model.world, player.id);
    }

}

pub fn create_player_commander_ships(model: &mut RymdGameModel, parameters: &RymdGameParameters) {

    for player in &parameters.players {

        let start_random_x = model.random.gen_range(-400, 400);
        let start_random_y = model.random.gen_range(-400, 400);

        let commander_ship = spawn_commander_ship(&mut model.world, player.id, vec2(start_random_x as f32, start_random_y as f32));
    
    }

}

pub fn get_number_of_commanders_of_player(world: &mut World, player_id: PlayerID) -> i32 {
    let mut number_of_commanders = 0;
    for (e, (commander, controller)) in world.query_mut::<(&Commander, &Controller)>() {         
        if controller.id == player_id {
            number_of_commanders += 1;
        }
    }
    number_of_commanders
}

pub fn is_commander_dead_for_player(world: &mut World, player_id: PlayerID) -> bool {
    for (e, player) in world.query_mut::<&Player>() {
        if player.id == player_id {
            return get_number_of_commanders_of_player(world, player_id) <= 0;
        }
    }
    false
}

pub fn is_any_commander_still_alive_in_team(world: &mut World, team: &RymdGameTeam) -> bool {
    let mut has_alive_commander = false;
    for &player_id in &team.players {
        if is_commander_dead_for_player(world, player_id) == false {
            has_alive_commander = true;
        }
    }
    has_alive_commander
}

pub fn destroy_all_units_controlled_by_team(world: &mut World, team: &RymdGameTeam) {
    for (e, (controller, health)) in world.query_mut::<(&Controller, &mut Health)>() {
        if team.players.contains(&controller.id) {
            health.kill();
        }
    }
}

pub fn create_asteroid_clumps(model: &mut RymdGameModel, number_of_asteroid_clumps: i32, number_of_asteroids: i32) {

    for i in 0..number_of_asteroid_clumps {

        let asteroid_clump_random_x = model.random.gen_range(-4000, 4000);
        let asteroid_clump_random_y = model.random.gen_range(-4000, 4000);

        for i in 0..number_of_asteroids {

            let random_x = model.random.gen_range(asteroid_clump_random_x - 400, asteroid_clump_random_x + 400);
            let random_y = model.random.gen_range(asteroid_clump_random_y - 400, asteroid_clump_random_y + 400);

            let new_asteroid = create_asteroid(&mut model.world, vec2(random_x as f32, random_y as f32), 0.0);

        }

    }

}
