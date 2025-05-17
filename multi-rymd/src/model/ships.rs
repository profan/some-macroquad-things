use std::f32::consts::PI;

use hecs::*;
use macroquad::prelude::*;
use utility::{AsAngle, RotatedBy, SteeringParameters};

use crate::PlayerID;
use crate::model::{Transform, Orderable, AnimatedSprite, Thruster, DynamicBody, Ship, ThrusterKind};
use super::{cancel_pending_orders, create_default_kinematic_body, create_explosion_effect_in_buffer, get_entity_position, get_player_team_allegiance, Attackable, Attacker, BeamParameters, BeamWeapon, Blueprint, BlueprintIdentity, Blueprints, Commander, Constructor, Controller, Cost, EntityState, Extractor, Health, MovementTarget, Producer, ProjectileWeapon, RotationTarget, Steering, ARROWHEAD_STEERING_PARAMETERS, COMMANDER_STEERING_PARAMETERS, DEFAULT_STEERING_PARAMETERS, DRAGONFLY_STEERING_PARAMETERS, EXTRACTOR_STEERING_PARAMETERS, SIMPLE_BEAM_PARAMETERS, SIMPLE_BULLET_PARAMETERS};

#[derive(Bundle)]
pub struct ShipThruster {
    transform: Transform,
    thruster: Thruster
}

impl ShipThruster {
    pub fn new(position: Vec2, direction: Vec2, angle: f32, velocity: f32, amount: f32, kind: ThrusterKind, parent: Entity) -> ShipThruster {
        ShipThruster {
            transform: Transform::new(position, direction.as_angle(), Some(parent)),
            thruster: Thruster { kind, direction, angle, power: velocity, rate: amount },
        }
    }
}

pub fn create_commander_ship_blueprint() -> Blueprint {
    Blueprint {
        id: Blueprints::Commander as i32,
        shortcut: KeyCode::J,
        name: String::from("Commander Ship"),
        texture: String::from("PLAYER_SHIP"),
        constructor: build_commander_ship,
        cost: Cost { metal: 500.0, energy: 500.0 },
        is_building: false
    }
}

pub fn create_commissar_ship_blueprint() -> Blueprint {
    Blueprint {
        id: Blueprints::Commander as i32,
        shortcut: KeyCode::K,
        name: String::from("Commissar Ship"),
        texture: String::from("ENEMY_SHIP"),
        constructor: build_commissar_ship,
        cost: Cost { metal: 500.0, energy: 500.0 },
        is_building: false
    }
}

pub fn create_arrowhead_ship_blueprint() -> Blueprint {
    Blueprint {
        id: Blueprints::Arrowhead as i32,
        shortcut: KeyCode::Y,
        name: String::from("Arrowhead (Fighter)"),
        texture: String::from("ARROWHEAD"),
        constructor: build_arrowhead_ship,
        cost: Cost { metal: 100.0, energy: 50.0 },
        is_building: false
    }
}

pub fn create_dragonfly_ship_blueprint() -> Blueprint {
        Blueprint {
        id: Blueprints::Dragonfly as i32,
        shortcut: KeyCode::D,
        name: String::from("Dragonfly (Drone)"),
        texture: String::from("DRAGONFLY"),
        constructor: build_dragonfly_ship,
        cost: Cost { metal: 20.0, energy: 10.0 },
        is_building: false
    }
}

pub fn create_extractor_ship_blueprint() -> Blueprint {
    Blueprint {
        id: Blueprints::Extractor as i32,
        shortcut: KeyCode::U,
        name: String::from("Extractor (Assist)"),
        texture: String::from("EXTRACTOR"),
        constructor: build_extractor_ship,
        cost: Cost { metal: 250.0, energy: 250.0 },
        is_building: false
    }
}

pub fn create_grunt_ship_blueprint() -> Blueprint {
    Blueprint {
        id: Blueprints::Grunt as i32,
        shortcut: KeyCode::I,
        name: String::from("Grunt (Fighter)"),
        texture: String::from("ENEMY_GRUNT"),
        constructor: build_grunt_ship,
        cost: Cost { metal: 50.0, energy: 25.0 },
        is_building: false
    }
}

fn on_ship_death(world: &World, buffer: &mut CommandBuffer, entity: Entity) {

    destroy_ship_thrusters(world, entity, buffer);
    cancel_pending_orders(world, entity);
    
    let ship_position = get_entity_position(world, entity).unwrap();
    create_explosion_effect_in_buffer(buffer, ship_position);
    
}

fn destroy_ship_thrusters(world: &World, entity: Entity, buffer: &mut CommandBuffer) {
    if let Ok(ship) = world.get::<&Ship>(entity) {
        for &target in &ship.thrusters {
            if let Ok(mut health) = world.get::<&mut Health>(target) {
                health.kill();
            } else {
                buffer.despawn(target);
            }
        }
    }
}

struct ShipParameters {

    pub initial_health: f32,
    pub maximum_health: f32,
    pub blueprint: Blueprints,

    // texture, bounds of the ship accordingly
    pub bounds: Rect,
    pub texture: String,
    pub texture_h_frames: i32,

    // steering related parameters
    pub steering_parameters: SteeringParameters

}

fn rotate_ship_parts(world: &mut World, entity: Entity, angle: f32) {

    for e in world.query_one_mut::<&Ship>(entity).unwrap().thrusters.clone() {
        let thruster = world.query_one_mut::<&mut ShipThruster>(e).unwrap();
        thruster.transform.local_rotation += angle;
    }

}

/// Creates a basic ship with all the components needed for the "base".
fn create_ship(world: &mut World, owner: PlayerID, position: Vec2, parameters: ShipParameters) -> Entity {

    let controller = Controller { id: owner };
    let transform = Transform::new(position, 0.0, None);

    let blueprint_identity = BlueprintIdentity::new(parameters.blueprint);
    let health = Health::new_with_current_health_and_callback(parameters.maximum_health, parameters.initial_health, on_ship_death);

    let is_body_enabled: bool = true;
    let is_body_static: bool = true;
    let body_mask = 1 << get_player_team_allegiance(world, owner);

    let steering_parameters = parameters.steering_parameters;
    let kinematic_body = create_default_kinematic_body(position, 0.0);

    let dynamic_body = DynamicBody { is_static: is_body_static, is_enabled: is_body_enabled, kinematic: kinematic_body, mask: body_mask, bounds: parameters.bounds };
    let sprite = AnimatedSprite { texture: parameters.texture, current_frame: 0, h_frames: parameters.texture_h_frames };
    let steering = Steering { parameters: steering_parameters };
    let ship = Ship::new();
    let orderable = Orderable::new();
    let state = EntityState::Ghost;
    let attackable = Attackable;

    let movement_target = MovementTarget { target: None };
    let rotation_target = RotationTarget { target: None };

    world.spawn((
        controller, transform, blueprint_identity, health,
        dynamic_body, sprite, steering, ship, orderable, state, attackable,
        movement_target, rotation_target
    ))

}

pub fn spawn_commander_ship(world: &mut World, owner: PlayerID, position: Vec2) -> Entity {

    let commander_ship = build_commander_ship(world, owner, position);
    if let Ok(mut health) = world.get::<&mut Health>(commander_ship) {
        health.heal_to_full_health();
    }

    commander_ship

}

pub fn build_commander_ship(world: &mut World, owner: PlayerID, position: Vec2) -> Entity {

    let commander_ship_size = 32.0;
    let commander_bounds = Rect { x: 0.0, y: 0.0, w: commander_ship_size, h: commander_ship_size };

    let initial_commander_health = 250.0;
    let maximum_commander_health = 1000.0;

    let commander_build_speed = 100;
    let commander_build_range = 100;
    let commander_build_offset = -vec2(commander_bounds.w / 8.0, 0.0);
    let commander_blueprints = vec![
        Blueprints::Shipyard as i32,
        Blueprints::SolarCollector as i32,
        Blueprints::EnergyStorage as i32,
        Blueprints::MetalStorage as i32,
        Blueprints::EnergyConverter as i32
    ];

    let commander_thruster_power = 64.0;
    let commander_turn_thruster_power = 16.0;

    let commander_metal_income = 10.0;
    let commander_energy_income = 10.0;

    let commander_steering_parameters = COMMANDER_STEERING_PARAMETERS;

    let commander_ship_parameters = ShipParameters {

        initial_health: initial_commander_health,
        maximum_health: maximum_commander_health,
        blueprint: Blueprints::Commander,

        bounds: commander_bounds,
        texture: "PLAYER_SHIP".to_string(),
        texture_h_frames: 3,

        steering_parameters: commander_steering_parameters

    };

    let commander_ship_body = create_ship(world, owner, position, commander_ship_parameters);

    let commander = Commander {};

    let constructor = Constructor {
        current_target: None,
        constructibles: commander_blueprints,
        build_speed: commander_build_speed,
        build_range: commander_build_range,
        beam_offset: commander_build_offset.rotated_by(PI / 2.0),
        can_assist: true
    };

    let extractor = Extractor {
        current_target: None,
        last_target: None,
        extraction_range: commander_build_range,
        extraction_speed: commander_build_speed,
        beam_offset: commander_build_offset.rotated_by(PI / 2.0),
        is_searching: false,
        is_active: false
    };

    let producer = Producer {
        metal: commander_metal_income,
        energy: commander_energy_income
    };

    let _ = world.insert(commander_ship_body, (commander, constructor, extractor, producer));

    // add ship thrusters
    let commander_ship_thruster_left_top = world.spawn(ShipThruster::new(vec2(-14.0, 4.0).rotated_by(PI/ 2.0), -Vec2::X.rotated_by(PI/ 2.0), -(PI / 2.0), commander_turn_thruster_power, commander_turn_thruster_power, ThrusterKind::Attitude, commander_ship_body));
    let commander_ship_thuster_right_top = world.spawn(ShipThruster::new(vec2(14.0, 4.0).rotated_by(PI/ 2.0), Vec2::X.rotated_by(PI/ 2.0), PI / 2.0, commander_turn_thruster_power, commander_turn_thruster_power, ThrusterKind::Attitude, commander_ship_body));

    let commander_ship_thruster_left_bottom = world.spawn(ShipThruster::new(vec2(-14.0, 4.0).rotated_by(PI/ 2.0), Vec2::X.rotated_by(PI/ 2.0), -(PI / 2.0), commander_turn_thruster_power, commander_turn_thruster_power, ThrusterKind::Attitude, commander_ship_body));
    let commander_ship_thuster_right_bottom = world.spawn(ShipThruster::new(vec2(14.0, 4.0).rotated_by(PI/ 2.0), -Vec2::X.rotated_by(PI/ 2.0), PI / 2.0, commander_turn_thruster_power, commander_turn_thruster_power, ThrusterKind::Attitude, commander_ship_body));

    let commander_ship_thruster_main = world.spawn(ShipThruster::new(vec2(0.0, 10.0).rotated_by(PI/ 2.0), Vec2::Y.rotated_by(PI/ 2.0), 0.0, commander_thruster_power, commander_thruster_power, ThrusterKind::Main, commander_ship_body));

    {
        let mut commander_ship = world.get::<&mut Ship>(commander_ship_body).unwrap();
        commander_ship.thrusters.push(commander_ship_thruster_left_top);
        commander_ship.thrusters.push(commander_ship_thuster_right_top);
        commander_ship.thrusters.push(commander_ship_thruster_left_bottom);
        commander_ship.thrusters.push(commander_ship_thuster_right_bottom);
        commander_ship.thrusters.push(commander_ship_thruster_main);
    }

    commander_ship_body

}

pub fn build_commissar_ship(world: &mut World, owner: PlayerID, position: Vec2) -> Entity {

    let commissar_ship_size = 32.0;
    let commissar_bounds = Rect { x: 0.0, y: 0.0, w: commissar_ship_size, h: commissar_ship_size };

    let initial_commissar_health = 250.0;
    let maximum_commissar_health = 1000.0;

    let commissar_build_speed = 100;
    let commissar_build_range = 100;
    let commissar_build_offset = -vec2(commissar_bounds.w / 8.0, 0.0);
    let commissar_blueprints = vec![Blueprints::Shipyard as i32, Blueprints::SolarCollector as i32, Blueprints::EnergyStorage as i32, Blueprints::MetalStorage as i32];

    let commissar_thruster_power = 64.0;
    let commissar_turn_thruster_power = 16.0;

    let commissar_metal_income = 10.0;
    let commissar_energy_income = 10.0;

    let commissar_steering_parameters = DEFAULT_STEERING_PARAMETERS;

    let commissar_ship_parameters = ShipParameters {

        initial_health: initial_commissar_health,
        maximum_health: maximum_commissar_health,
        blueprint: Blueprints::Commander,

        bounds: commissar_bounds,
        texture: "PLAYER_SHIP".to_string(),
        texture_h_frames: 3,

        steering_parameters: commissar_steering_parameters

    };

    let commissar_ship_body = create_ship(world, owner, position, commissar_ship_parameters);

    let commander = Commander {};

    let constructor = Constructor {
        current_target: None,
        constructibles: commissar_blueprints,
        build_speed: commissar_build_speed,
        build_range: commissar_build_range,
        beam_offset: commissar_build_offset.rotated_by(PI / 2.0),
        can_assist: true
    };

    let producer = Producer {
        metal: commissar_metal_income,
        energy: commissar_energy_income
    };

    let _ = world.insert(commissar_ship_body, (commander, constructor, producer));

    // add ship thrusters
    let commissar_ship_thruster_left_top = world.spawn(ShipThruster::new(vec2(-14.0, 4.0).rotated_by(PI/ 2.0), (-Vec2::X).rotated_by(PI/ 2.0), -(PI / 2.0), commissar_turn_thruster_power, commissar_turn_thruster_power, ThrusterKind::Attitude, commissar_ship_body));
    let commissar_ship_thuster_right_top = world.spawn(ShipThruster::new(vec2(14.0, 4.0).rotated_by(PI/ 2.0), Vec2::X.rotated_by(PI/ 2.0), PI / 2.0, commissar_turn_thruster_power, commissar_turn_thruster_power, ThrusterKind::Attitude, commissar_ship_body));

    let commissar_ship_thruster_left_bottom = world.spawn(ShipThruster::new(vec2(-14.0, 4.0).rotated_by(PI/ 2.0), Vec2::X.rotated_by(PI/ 2.0), -(PI / 2.0), commissar_turn_thruster_power, commissar_turn_thruster_power, ThrusterKind::Attitude, commissar_ship_body));
    let commissar_ship_thuster_right_bottom = world.spawn(ShipThruster::new(vec2(14.0, 4.0).rotated_by(PI/ 2.0), (-Vec2::X).rotated_by(PI/ 2.0), PI / 2.0, commissar_turn_thruster_power, commissar_turn_thruster_power, ThrusterKind::Attitude, commissar_ship_body));

    let commissar_ship_thruster_main = world.spawn(ShipThruster::new(vec2(0.0, 10.0).rotated_by(PI/ 2.0), Vec2::Y, 0.0, commissar_thruster_power, commissar_thruster_power, ThrusterKind::Main, commissar_ship_body));

    let mut commissar_ship = world.get::<&mut Ship>(commissar_ship_body).unwrap();
    commissar_ship.thrusters.push(commissar_ship_thruster_left_top);
    commissar_ship.thrusters.push(commissar_ship_thuster_right_top);
    commissar_ship.thrusters.push(commissar_ship_thruster_left_bottom);
    commissar_ship.thrusters.push(commissar_ship_thuster_right_bottom);
    commissar_ship.thrusters.push(commissar_ship_thruster_main);

    commissar_ship_body

}

pub fn build_arrowhead_ship(world: &mut World, owner: PlayerID, position: Vec2) -> Entity {

    let arrowhead_ship_size = 32.0;
    let arrowhead_bounds = Rect { x: 0.0, y: 0.0, w: arrowhead_ship_size, h: arrowhead_ship_size };

    let initial_arrowhead_health = 1.0;
    let maximum_arrowhead_health = 250.0;

    let arrowhead_thruster_power = 64.0;
    let arrowhead_turn_thruster_power = 16.0;
    let arrowhead_fire_rate = 0.25;
    let arrowhead_fire_deviation = 0.1;
    let arrowhead_fire_cooldown = 0.0;
    let arrowhead_fire_arc = PI / 8.0; // 22.5 degrees
    let arrowhead_range = 256.0;

    let arrowhead_steering_parameters = ARROWHEAD_STEERING_PARAMETERS;

    let arrowhead_ship_parameters = ShipParameters {

        initial_health: initial_arrowhead_health,
        maximum_health: maximum_arrowhead_health,
        blueprint: Blueprints::Arrowhead,

        bounds: arrowhead_bounds,
        texture: "ARROWHEAD".to_string(),
        texture_h_frames: 1,

        steering_parameters: arrowhead_steering_parameters

    };

    let arrowhead_ship_body = create_ship(world, owner, position, arrowhead_ship_parameters);

    let projectile_weapon = ProjectileWeapon {
        offset: vec2(0.0, -(arrowhead_ship_size / 2.0)).rotated_by(PI / 2.0),
        fire_rate: arrowhead_fire_rate,
        fire_arc: arrowhead_fire_arc,
        deviation: arrowhead_fire_deviation,
        cooldown: arrowhead_fire_cooldown,
        projectile: SIMPLE_BULLET_PARAMETERS
    };

    // let beam_weapon = BeamWeapon {
    //     offset: vec2(0.0, -(arrowhead_ship_size / 2.0)).rotated_by(PI / 2.0),
    //     fire_rate: arrowhead_fire_rate,
    //     deviation: arrowhead_fire_deviation,
    //     cooldown: arrowhead_fire_cooldown,
    //     beam: SIMPLE_BEAM_PARAMETERS
    // };

    let attacker = Attacker::new(arrowhead_range);
    let _ = world.insert(arrowhead_ship_body, (projectile_weapon, attacker));

    // add ship thrusters
    let arrowhead_ship_thruster_left_top = world.spawn(ShipThruster::new(vec2(-14.0, 4.0).rotated_by(PI/ 2.0), -Vec2::X.rotated_by(PI/ 2.0), -(PI / 2.0), arrowhead_turn_thruster_power, arrowhead_turn_thruster_power, ThrusterKind::Attitude, arrowhead_ship_body));
    let arrowhead_ship_thuster_right_top = world.spawn(ShipThruster::new(vec2(14.0, 4.0).rotated_by(PI/ 2.0), Vec2::X.rotated_by(PI/ 2.0), PI / 2.0, arrowhead_turn_thruster_power, arrowhead_turn_thruster_power, ThrusterKind::Attitude, arrowhead_ship_body));

    let arrowhead_ship_thruster_left_bottom = world.spawn(ShipThruster::new(vec2(-14.0, 4.0).rotated_by(PI/ 2.0), Vec2::X.rotated_by(PI/ 2.0), -(PI / 2.0), arrowhead_turn_thruster_power, arrowhead_turn_thruster_power, ThrusterKind::Attitude, arrowhead_ship_body));
    let arrowhead_ship_thuster_right_bottom = world.spawn(ShipThruster::new(vec2(14.0, 4.0).rotated_by(PI/ 2.0), (-Vec2::X).rotated_by(PI/ 2.0), PI / 2.0, arrowhead_turn_thruster_power, arrowhead_turn_thruster_power, ThrusterKind::Attitude, arrowhead_ship_body));

    let arrowhead_ship_thruster_main = world.spawn(ShipThruster::new(vec2(0.0, 10.0).rotated_by(PI/ 2.0), Vec2::Y.rotated_by(PI/ 2.0), 0.0, arrowhead_thruster_power, arrowhead_thruster_power, ThrusterKind::Main, arrowhead_ship_body));

    let mut arrowhead_ship = world.get::<&mut Ship>(arrowhead_ship_body).unwrap();
    arrowhead_ship.thrusters.push(arrowhead_ship_thruster_left_top);
    arrowhead_ship.thrusters.push(arrowhead_ship_thuster_right_top);
    arrowhead_ship.thrusters.push(arrowhead_ship_thruster_left_bottom);
    arrowhead_ship.thrusters.push(arrowhead_ship_thuster_right_bottom);
    arrowhead_ship.thrusters.push(arrowhead_ship_thruster_main);

    arrowhead_ship_body

}

pub fn build_dragonfly_ship(world: &mut World, owner: PlayerID, position: Vec2) -> Entity {

    let dragonfly_ship_size = 16.0;
    let dragonfly_bounds = Rect { x: 0.0, y: 0.0, w: dragonfly_ship_size, h: dragonfly_ship_size };

    let initial_dragonfly_health = 1.0;
    let maximum_dragonfly_health = 50.0;

    let dragonfly_thruster_power = 16.0;
    let dragonfly_thruster_rate = 64.0;

    let dragonfly_fire_rate = 1.0;
    let dragonfly_fire_deviation = 0.1;
    let dragonfly_fire_cooldown = 0.0;
    let dragonfly_fire_arc = PI / 8.0; // 22.5 degrees
    let dragonfly_range = 256.0;

    let dragonfly_beam_damage = 7.5;
    let dragonfly_beam_range = 64.0;

    let dragonfly_steering_parameters = DRAGONFLY_STEERING_PARAMETERS;

    let dragonfly_ship_parameters = ShipParameters {

        initial_health: initial_dragonfly_health,
        maximum_health: maximum_dragonfly_health,
        blueprint: Blueprints::Dragonfly,

        bounds: dragonfly_bounds,
        texture: "DRAGONFLY".to_string(),
        texture_h_frames: 1,

        steering_parameters: dragonfly_steering_parameters

    };

    let dragonfly = create_ship(world, owner, position, dragonfly_ship_parameters);

    let beam_weapon = BeamWeapon {
         offset: vec2(0.0, -(dragonfly_ship_size - 4.0)).rotated_by(PI / 2.0),
         fire_rate: dragonfly_fire_rate,
         deviation: dragonfly_fire_deviation,
         cooldown: dragonfly_fire_cooldown,
         fire_arc: dragonfly_fire_arc,
         beam: BeamParameters {
            damage: dragonfly_beam_damage,
            range: dragonfly_beam_range,
            ..SIMPLE_BEAM_PARAMETERS
         }
    };

    let attacker = Attacker::new(dragonfly_range);

    let _ = world.insert(dragonfly, (beam_weapon, attacker));

    let dragonfly_ship_thruster_main = world.spawn(ShipThruster::new(vec2(0.0, 5.0).rotated_by(PI/ 2.0), Vec2::Y.rotated_by(PI/ 2.0), 0.0, dragonfly_thruster_power, dragonfly_thruster_rate, ThrusterKind::Main, dragonfly));

    let mut dragonfly_ship = world.get::<&mut Ship>(dragonfly).unwrap();
    dragonfly_ship.thrusters.push(dragonfly_ship_thruster_main);

    dragonfly

}

pub fn build_extractor_ship(world: &mut World, owner: PlayerID, position: Vec2) -> Entity {

    let extractor_ship_size = 32.0;
    let extractor_bounds = Rect { x: 0.0, y: 0.0, w: extractor_ship_size, h: extractor_ship_size };

    let initial_extractor_health = 1.0;
    let maximum_extractor_health = 250.0;

    let extractor_build_speed = 100;
    let extractor_build_range = 100;
    let extractor_build_offset = -vec2(extractor_bounds.w / 8.0, 0.0);

    let extractor_thruster_power = 64.0;
    let extractor_turn_thruster_power = 16.0;

    let extractor_steering_parameters = EXTRACTOR_STEERING_PARAMETERS;

    let extractor_ship_parameters = ShipParameters {

        initial_health: initial_extractor_health,
        maximum_health: maximum_extractor_health,
        blueprint: Blueprints::Extractor,

        bounds: extractor_bounds,
        texture: "EXTRACTOR".to_string(),
        texture_h_frames: 1,

        steering_parameters: extractor_steering_parameters

    };

    let extractor = Extractor {
        current_target: None,
        last_target: None,
        extraction_range: extractor_build_speed,
        extraction_speed: extractor_build_range,
        beam_offset: extractor_build_offset.rotated_by(PI / 2.0),
        is_searching: false,
        is_active: false
    };

    let extractor_ship_body = create_ship(world, owner, position, extractor_ship_parameters);
    let _ = world.insert(extractor_ship_body, (extractor,));

    // add ship thrusters
    let extractor_ship_thruster_left_top = world.spawn(ShipThruster::new(vec2(-14.0, 4.0).rotated_by(PI/ 2.0), (-Vec2::X).rotated_by(PI/ 2.0), -(PI / 2.0), extractor_turn_thruster_power, extractor_turn_thruster_power, ThrusterKind::Attitude, extractor_ship_body));
    let extractor_ship_thuster_right_top = world.spawn(ShipThruster::new(vec2(14.0, 4.0).rotated_by(PI/ 2.0), Vec2::X.rotated_by(PI/ 2.0), PI / 2.0, extractor_turn_thruster_power, extractor_turn_thruster_power, ThrusterKind::Attitude, extractor_ship_body));

    let extractor_ship_thruster_left_bottom = world.spawn(ShipThruster::new(vec2(-14.0, 4.0).rotated_by(PI/ 2.0), Vec2::X.rotated_by(PI/ 2.0), -(PI / 2.0), extractor_turn_thruster_power, extractor_turn_thruster_power, ThrusterKind::Attitude, extractor_ship_body));
    let extractor_ship_thuster_right_bottom = world.spawn(ShipThruster::new(vec2(14.0, 4.0).rotated_by(PI/ 2.0), (-Vec2::X).rotated_by(PI/ 2.0), PI / 2.0, extractor_turn_thruster_power, extractor_turn_thruster_power, ThrusterKind::Attitude, extractor_ship_body));

    let extractor_ship_thruster_main = world.spawn(ShipThruster::new(vec2(0.0, 10.0).rotated_by(PI/ 2.0), Vec2::Y.rotated_by(PI/ 2.0), 0.0, extractor_thruster_power, extractor_thruster_power, ThrusterKind::Main, extractor_ship_body));

    let mut extractor_ship = world.get::<&mut Ship>(extractor_ship_body).unwrap();
    extractor_ship.thrusters.push(extractor_ship_thruster_left_top);
    extractor_ship.thrusters.push(extractor_ship_thuster_right_top);
    extractor_ship.thrusters.push(extractor_ship_thruster_left_bottom);
    extractor_ship.thrusters.push(extractor_ship_thuster_right_bottom);
    extractor_ship.thrusters.push(extractor_ship_thruster_main);

    extractor_ship_body

}

pub fn build_grunt_ship(world: &mut World, owner: PlayerID, position: Vec2) -> Entity {

    let grunt_ship_size = 16.0;
    let grunt_bounds = Rect { x: 0.0, y: 0.0, w: grunt_ship_size, h: grunt_ship_size };

    let initial_grunt_health = 1.0;
    let maximum_grunt_health = 100.0;

    let grunt_thruster_power = 32.0;
    let grunt_turn_thruster_power = 16.0;
    let grunt_fire_rate = 0.75;
    let grunt_fire_deviation = 0.1;
    let grunt_fire_cooldown = 0.0;
    let grunt_fire_range = 256.0;
    let grunt_fire_arc = PI / 8.0;

    let grunt_steering_parameters = DEFAULT_STEERING_PARAMETERS;

    let grunt_ship_parameters = ShipParameters {

        initial_health: initial_grunt_health,
        maximum_health: maximum_grunt_health,
        blueprint: Blueprints::Grunt,

        bounds: grunt_bounds,
        texture: "ENEMY_GRUNT".to_string(),
        texture_h_frames: 1,

        steering_parameters: grunt_steering_parameters

    };

    let grunt_ship_body = create_ship(world, owner, position, grunt_ship_parameters);

    let projectile_weapon = ProjectileWeapon {
        offset: vec2(0.0, -(grunt_ship_size / 2.0)).rotated_by(PI / 2.0),
        fire_rate: grunt_fire_rate,
        fire_arc: grunt_fire_arc,
        deviation: grunt_fire_deviation,
        cooldown: grunt_fire_cooldown,
        projectile: SIMPLE_BULLET_PARAMETERS
    };

    let attacker = Attacker::new(grunt_fire_range);

    let _ = world.insert(grunt_ship_body, (projectile_weapon, attacker));
    let grunt_ship_thruster_main = world.spawn(ShipThruster::new(vec2(0.0, 4.0), Vec2::Y, 0.0, grunt_thruster_power, grunt_thruster_power, ThrusterKind::Main, grunt_ship_body));

    let mut grunt_ship = world.get::<&mut Ship>(grunt_ship_body).unwrap();
    grunt_ship.thrusters.push(grunt_ship_thruster_main);

    grunt_ship_body

}