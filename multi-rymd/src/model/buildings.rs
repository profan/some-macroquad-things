use hecs::{CommandBuffer, Entity, World};
use macroquad::{math::{Vec2, Rect, vec2}, miniquad::KeyCode};

use crate::PlayerID;
use super::{cancel_pending_orders, create_default_kinematic_body, create_explosion_effect_in_buffer, get_entity_position, Attackable, Blueprint, BlueprintIdentity, Blueprints, Building, Constructor, Controller, Cost, DynamicBody, EntityState, GameOrderType, Health, MovementTarget, Orderable, Producer, Spawner, Sprite, Storage, Transform};

pub fn create_solar_collector_blueprint() -> Blueprint {
    Blueprint {
        id: Blueprints::SolarCollector as i32,
        shortcut: KeyCode::Q,
        name: String::from("Solar Collector"),
        texture: String::from("SOLAR_COLLECTOR"),
        constructor: build_solar_collector,
        cost: Cost { metal: 25.0, energy: 25.0 },
        is_building: true
    }
}

pub fn create_shipyard_blueprint() -> Blueprint {
    Blueprint {
        id: Blueprints::Shipyard as i32,
        shortcut: KeyCode::W,
        name: String::from("Shipyard"),
        texture: String::from("SHIPYARD"),
        constructor: build_shipyard,
        cost: Cost { metal: 100.0, energy: 100.0 },
        is_building: true
    }
}

pub fn create_energy_storage_blueprint() -> Blueprint {
    Blueprint {
        id: Blueprints::EnergyStorage as i32,
        shortcut: KeyCode::E,
        name: String::from("Energy Storage"),
        texture: String::from("ENERGY_STORAGE"),
        constructor: build_energy_storage,
        cost: Cost { metal: 25.0, energy: 0.0 },
        is_building: true
    }
}

pub fn create_metal_storage_blueprint() -> Blueprint {
    Blueprint {
        id: Blueprints::MetalStorage as i32,
        shortcut: KeyCode::R,
        name: String::from("Metal Storage"),
        texture: String::from("METAL_STORAGE"),
        constructor: build_metal_storage,
        cost: Cost { metal: 25.0, energy: 0.0 },
        is_building: true
    }
}

fn on_building_death(world: &World, buffer: &mut CommandBuffer, entity: Entity) {

    destroy_entity_in_construction(world, entity, buffer);
    cancel_pending_orders(world, entity);
    
    let building_position = get_entity_position(world, entity).unwrap();
    create_explosion_effect_in_buffer(buffer, building_position);
    
}

fn destroy_entity_in_construction(world: &World, entity: Entity, buffer: &mut CommandBuffer) {

    let Ok(constructor) = world.get::<&Constructor>(entity) else { return; };
    let Some(target) = constructor.current_target else { return; };

    let Ok(target_state) = world.get::<&EntityState>(target) else { return; };
    if *target_state != EntityState::Ghost { return; };

    if let Ok(mut health) = world.get::<&mut Health>(target) {
        health.kill();
    } else {
        buffer.despawn(target);
    }

}

struct BuildingParameters {

    initial_health: f32,
    maximum_health: f32,
    blueprint: Blueprints,

    pub bounds: Rect,
    pub texture: String

}

fn create_building(world: &mut World, owner: PlayerID, position: Vec2, parameters: BuildingParameters) -> Entity {

    let is_body_enabled = true;
    let is_body_static = true;
    let body_mask = 1 << owner;

    let kinematic = create_default_kinematic_body(position, 0.0);

    let controller = Controller { id: owner };
    let transform = Transform::new(position, 0.0, None);
    let blueprint_identity = BlueprintIdentity::new(parameters.blueprint);
    let health = Health::new_with_current_health_and_callback(parameters.maximum_health, parameters.initial_health, on_building_death);
    let sprite = Sprite { texture: parameters.texture };
    let dynamic_body = DynamicBody { is_enabled: is_body_enabled, is_static: is_body_static, bounds: parameters.bounds, kinematic, mask: body_mask };
    let state = EntityState::Ghost;
    let attackable = Attackable;
    let building = Building;

    world.spawn((controller, transform, blueprint_identity, health, sprite, dynamic_body, state, attackable, building))

}

pub fn build_solar_collector(world: &mut World, owner: PlayerID, position: Vec2) -> Entity {

    let solar_collector_size = 64.0;
    let solar_collector_bounds = Rect { x: 0.0, y: 0.0, w: solar_collector_size, h: solar_collector_size };

    let maximum_solar_collector_health = 1000.0;
    let initial_solar_collector_health = 10.0;

    let solar_collector_parameters = BuildingParameters {

        initial_health: initial_solar_collector_health,
        maximum_health: maximum_solar_collector_health,
        blueprint: Blueprints::SolarCollector,

        bounds: solar_collector_bounds,
        texture: "SOLAR_COLLECTOR".to_string()

    };

    let solar_collector = create_building(world, owner, position, solar_collector_parameters);
    let resource_producer = Producer { metal: 0.0, energy: 10.0 };

    let _ = world.insert(solar_collector, (resource_producer,));

    solar_collector

}

pub fn build_shipyard(world: &mut World, owner: PlayerID, position: Vec2) -> Entity {

    let shipyard_size = 128.0;
    let shipyard_bounds = Rect { x: shipyard_size / 4.0, y: 0.0, w: shipyard_size / 2.0, h: shipyard_size };

    let maximum_shipyard_health = 1000.0;
    let initial_shipyard_health = 10.0;

    let shipyard_build_speed = 100;
    let shipyard_blueprints = vec![Blueprints::Arrowhead as i32, Blueprints::Grunt as i32];

    let shipyard_parameters = BuildingParameters {
        
        initial_health: initial_shipyard_health,
        maximum_health: maximum_shipyard_health,
        blueprint: Blueprints::Shipyard,

        bounds: shipyard_bounds,
        texture: "SHIPYARD".to_string()

    };

    let shipyard = create_building(world, owner, position, shipyard_parameters);

    let spawner = Spawner { position: vec2(-(shipyard_size / 5.0), 0.0) };
    let constructor = Constructor { current_target: None, constructibles: shipyard_blueprints, build_range: shipyard_size as i32 / 2, build_speed: shipyard_build_speed, beam_offset: -vec2(0.0, 8.0), can_assist: false };
    let movement_target = MovementTarget { target: None };
    let orderable = Orderable::new();

    let _ = world.insert(shipyard, (spawner, constructor, movement_target, orderable));

    shipyard

}

pub fn build_energy_storage(world: &mut World, owner: PlayerID, position: Vec2) -> Entity {

    let energy_storage_size = 32.0;
    let energy_storage_bounds = Rect { x: 0.0, y: 0.0, w: energy_storage_size, h: energy_storage_size };

    let maximum_energy_storage_health = 250.0;
    let initial_energy_storage_health = 10.0;
    let energy_storage_amount = 1000.0;

    let energy_storage_parameters = BuildingParameters {

        initial_health: initial_energy_storage_health,
        maximum_health: maximum_energy_storage_health,
        blueprint: Blueprints::EnergyStorage,

        bounds: energy_storage_bounds,
        texture: "ENERGY_STORAGE".to_string()

    };

    let energy_storage = create_building(world, owner, position, energy_storage_parameters);

    let storage = Storage { metal: 0.0, energy: energy_storage_amount };
    let _ = world.insert(energy_storage, (storage,));

    energy_storage

}

pub fn build_metal_storage(world: &mut World, owner: PlayerID, position: Vec2) -> Entity {

    let metal_storage_size = 32.0;
    let metal_storage_bounds = Rect { x: 0.0, y: 0.0, w: metal_storage_size, h: metal_storage_size };

    let maximum_metal_storage_health = 250.0;
    let initial_metal_storage_health = 10.0;
    let metal_storage_amount = 1000.0;

    let metal_storage_parameters = BuildingParameters {

        initial_health: initial_metal_storage_health,
        maximum_health: maximum_metal_storage_health,
        blueprint: Blueprints::MetalStorage,

        bounds: metal_storage_bounds,
        texture: "METAL_STORAGE".to_string()

    };

    let metal_storage = create_building(world, owner, position, metal_storage_parameters);

    let storage = Storage { metal: metal_storage_amount, energy: 0.0 };
    let _ = world.insert(metal_storage, (storage, ));

    metal_storage

}