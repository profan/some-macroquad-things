#![allow(dead_code)]
#![allow(unused_variables)]
#![feature(async_fn_in_trait)]
#![feature(let_chains)]

use std::{collections::VecDeque, f32::consts::PI};
use nanoserde::{SerJson, DeJson};
use macroquad::prelude::*;
use hecs::*;

use utility::{DebugText, Kinematic, RotatedBy, AsAngle, pursue, arrive, face, SteeringParameters, SteeringOutput, AsVector, arrive_ex, face_ex};
use lockstep_client::{game::Game, step::LockstepClient, step::PeerID, app::ApplicationState};
use prefab::{create_player_ship};

mod utils;
mod orders;
mod physics;
mod sprite;
mod prefab;
mod view;

use u64 as EntityID;
use utils::hecs::CloneableWorld;
use view::RymdGameView;

const DEFAULT_STEERING_PARAMETERS: SteeringParameters = SteeringParameters {

    acceleration: 256.0,

    max_speed: 384.0,
    max_acceleration: 128.0,
    arrive_radius: 64.0,
    slow_radius: 200.0,

    align_max_rotation: 2.0,
    align_max_angular_acceleration: 2.0,
    align_radius: 0.0125 / 4.0,
    align_slow_radius: 0.05 / 4.0,

    separation_threshold: 512.0,
    separation_decay_coefficient: 2048.0

};

fn ship_apply_steering(kinematic: &mut Kinematic, steering_maybe: Option<SteeringOutput>, dt: f32) {

    let turn_rate = 4.0;
    if let Some(steering) = steering_maybe {

        let desired_linear_velocity = steering.linear * dt;

        // project our desired velocity along where we're currently pointing first
        let projected_linear_velocity = desired_linear_velocity * desired_linear_velocity.dot(-kinematic.orientation.as_vector()).max(0.0);
        kinematic.velocity += projected_linear_velocity;

        let turn_delta = steering.angular * turn_rate * dt;
        kinematic.angular_velocity += turn_delta;

    }

}

fn get_entity_position(world: &World, entity_id: u64) -> Option<Vec2> {
    world.get::<&Transform>(Entity::from_bits(entity_id).unwrap()).and_then(|t| Ok(t.world_position)).or(Err(())).ok()
}

impl GameOrder {

    fn is_order_completed(&self, entity: Entity, world: &World) -> bool {
        match self {
            GameOrder::Move { x, y } => {
                if let Ok(transform) = world.get::<&Transform>(entity) {
                    let arbitrary_distance_threshold = 64.0;
                    transform.world_position.distance(vec2(*x, *y)) < arbitrary_distance_threshold
                } else {
                    false
                }
            },
            GameOrder::Attack { entity_id } => todo!(),
            GameOrder::AttackMove { entity_id } => todo!(),
        }
    }

    fn get_target_position(&self, world: &World) -> Option<Vec2> {
        match self {
            GameOrder::Move { x, y } => Some(vec2(*x, *y)),
            GameOrder::Attack { entity_id } => get_entity_position(world, *entity_id),
            GameOrder::AttackMove { entity_id } => get_entity_position(world, *entity_id),
        }
    }

    fn tick(&self, entity: Entity, world: &mut World, dt: f32) {
        match self {
            GameOrder::Move { x, y } => {

                if let Ok(mut dynamic_body) = world.get::<&mut DynamicBody>(entity) {

                    let target_kinematic = Kinematic { position: vec2(*x, *y), ..Default::default() };
                    let time_to_target = 1.0;

                    let arrive_steering_output = arrive_ex(
                        &dynamic_body.kinematic,
                        &target_kinematic,
                        DEFAULT_STEERING_PARAMETERS,
                        time_to_target
                    ).unwrap_or_default();

                    let face_steering_output = face_ex(
                        &dynamic_body.kinematic,
                        &target_kinematic,
                        DEFAULT_STEERING_PARAMETERS,
                        time_to_target
                    ).unwrap_or_default();

                    let final_steering_output = arrive_steering_output + face_steering_output;
                    ship_apply_steering(&mut dynamic_body.kinematic, Some(final_steering_output), dt);

                }

            },
            GameOrder::Attack { entity_id } => (),
            GameOrder::AttackMove { entity_id } => (),
        }
    }
 
}

#[derive(Debug, Copy, Clone, SerJson, DeJson)]
enum GameOrder {
    Move {x : f32, y: f32 },
    Attack { entity_id: EntityID },
    AttackMove { entity_id: EntityID },
}

#[derive(Debug, SerJson, DeJson)]
enum GameMessage {
    Order { entities: Vec<EntityID>, order: GameOrder, add: bool },
}

#[derive(Clone)]
struct Thruster {
    kind: ThrusterKind,
    direction: Vec2,
    angle: f32,
    power: f32
}

#[derive(Clone, Copy, PartialEq)]
pub enum ThrusterKind {
    Main,
    Attitude,
}

#[derive(Clone)]
struct Input {
    forward: bool,
    backward: bool,
    turn_left: bool,
    turn_right: bool,
    fast: bool
}

#[derive(Debug, Default, Clone, Copy)]
struct Transform {
    world_position: Vec2,
    world_rotation: f32,
    local_position: Vec2,
    local_rotation: f32,
    parent: Option<Entity>
}

impl Transform {

    pub fn new(position: Vec2, rotation: f32, parent: Option<Entity>) -> Transform {
        Transform {
            world_position: position,
            world_rotation: rotation,
            local_position: position,
            local_rotation: rotation,
            parent: parent
        }
    }

    pub fn get_transform(world: &World, entity: Option<Entity>) -> Transform {
        if let Some(entity) = entity && let Ok(transform) = world.get::<&Transform>(entity) {
            *transform.clone()
        } else {
            Transform { ..Default::default() }
        }
    }

    pub fn world_to_local(&self, world: &World, world_position: Vec2) -> Vec2 {
        let parent_transform = Self::get_transform(world, self.parent);
        (world_position - parent_transform.world_position).rotated_by(-parent_transform.world_rotation)
    }

    pub fn local_to_world(&self, world: &World, local_position: Vec2) -> Vec2 {
        let parent_transform = Self::get_transform(world, self.parent);
        (local_position.rotated_by(parent_transform.world_rotation)) + parent_transform.world_position
    }

    pub fn calculate_transform(&self, world: &World, entity: Entity) -> Transform {

        let mut current_entity = entity;
        let mut calculated_transform = Transform {
            world_position: self.local_position,
            world_rotation: self.local_rotation,
            local_position: self.local_position,
            local_rotation: self.local_rotation,
            parent: self.parent
        };

        while let Ok(mut current_query) = world.query_one::<&Transform>(current_entity) && let Some(current_transform) = current_query.get() && let Some(parent_entity) = current_transform.parent {
            if let Ok(mut parent_query) = world.query_one::<&Transform>(parent_entity) && let Some(parent_transform) = parent_query.get() {
                calculated_transform.world_position = calculated_transform.world_position.rotated_by(parent_transform.local_rotation) + parent_transform.local_position;
                calculated_transform.world_rotation += parent_transform.local_rotation;
                current_entity = parent_entity;
            } else {
                break;
            }
        }

        calculated_transform

    }

}

#[derive(Clone)]
struct DynamicBody {
    pub kinematic: Kinematic,
}

#[derive(Clone)] 
struct Sprite {
    texture: String
}

#[derive(Clone)]
struct AnimatedSprite {
    texture: String,
    current_frame: i32,
    v_frames: i32
}

#[derive(Clone)]
struct Orderable {
    orders: VecDeque<GameOrder>
}

impl Orderable {
    pub fn new() -> Orderable {
        Orderable { orders: VecDeque::new() }
    }
}

#[derive(Clone)]
struct Ship {
    turn_rate: f32,
    thrusters: Vec<Entity>
}

impl Ship {
    pub fn new(turn_rate: f32) -> Ship {
        Ship { turn_rate, thrusters: Vec::new() }
    }
}

#[derive(Debug)]
struct RymdGamePlayer {
    id: PeerID
}

#[derive(Debug)]
struct RymdGameParameters {
    players: Vec<RymdGamePlayer>
}

struct RymdGameModel {
    world: World
}

impl RymdGameModel {

    const TIME_STEP: f32 = 1.0 / 60.0;

    pub fn new() -> RymdGameModel {
        RymdGameModel {
            world: World::new()
        }
    }

    pub fn start(&mut self, parameters: RymdGameParameters) {

        for player in &parameters.players {
            let random_x = rand::gen_range(200, 400);
            let random_y = rand::gen_range(200, 400);
            create_player_ship(&mut self.world, vec2(random_x as f32, random_y as f32));
        }
        
    }

    pub fn stop(&mut self) {
        self.world.clear();
    }

    fn handle_order(&mut self, entities: Vec<EntityID>, order: GameOrder, should_add: bool) {

        for entity_id in entities {
            let Some(entity) = Entity::from_bits(entity_id) else { continue; };
            if let Ok(orderable) = self.world.query_one_mut::<&mut Orderable>(entity) {
                if should_add {
                    orderable.orders.push_back(order);
                } else {
                    orderable.orders.clear();
                    orderable.orders.push_back(order);
                }
            }
        }
        
    }

    pub fn handle_message(&mut self, message: &str) {
        let msg = match GameMessage::deserialize_json(message) {
            Ok(message) => message,
            Err(err) => {
                println!("[RymdGameModel] failed to parse message: {}!", message);
                return;
            }
        };

        println!("[RymdGameModel] got message: {:?}", msg);

        match msg {
            GameMessage::Order { entities, order, add } => self.handle_order(entities, order, add),
        }
    }

    fn tick_orderables(&mut self) {

        let mut in_progress_orders = Vec::new();
        let mut completed_orders = Vec::new();

        for (e, orderable) in self.world.query::<&Orderable>().iter() {
            if let Some(order) = orderable.orders.front() {
                if order.is_order_completed(e, &self.world) {
                    completed_orders.push(e);
                } else {
                    in_progress_orders.push(e);
                }
            }
        }

        for &e in &in_progress_orders {
            if let Ok(orderable) = self.world.query_one_mut::<&Orderable>(e).cloned() {
                if let Some(order) = orderable.orders.front() {
                    order.tick(e, &mut self.world, Self::TIME_STEP);
                }
            }
        }

        for &e in &completed_orders {
            if let Ok(orderable) = self.world.query_one_mut::<&mut Orderable>(e) {
                orderable.orders.pop_front();
            }
        }

    }

    fn tick_transforms(&mut self) {

        let mut updated_transforms = Vec::new();

        for (e, transform) in self.world.query::<&Transform>().iter() {
            updated_transforms.push(
                (e, transform.calculate_transform(&self.world, e))
            );
        }

        for (e, transform) in updated_transforms {
            let _ = self.world.insert_one(e, transform);
        }

    }

    fn tick_physics_bodies(&mut self) {

        for (e, (transform, body)) in self.world.query_mut::<(&mut Transform, &mut DynamicBody)>() {

            body.kinematic.integrate(Self::TIME_STEP);
            body.kinematic.apply_friction(Self::TIME_STEP);

            transform.local_position = body.kinematic.position;
            transform.local_rotation = body.kinematic.orientation;

        }

    }

    pub fn calculate_transform(world: &World, entity: Entity, transform: &Transform) -> Transform {

        let mut current_entity = entity;
        let mut calculated_transform = Transform {
            world_position: transform.local_position,
            world_rotation: transform.local_rotation,
            local_position: transform.local_position,
            local_rotation: transform.local_rotation,
            parent: transform.parent
        };

        while let Ok(mut current_query) = world.query_one::<&Transform>(current_entity) && let Some(current_transform) = current_query.get() && let Some(parent_entity) = current_transform.parent {
            if let Ok(mut parent_query) = world.query_one::<&Transform>(parent_entity) && let Some(parent_transform) = parent_query.get() {
                calculated_transform.world_position = calculated_transform.world_position.rotated_by(parent_transform.local_rotation) + parent_transform.local_position;
                calculated_transform.world_rotation += parent_transform.local_rotation;
                current_entity = parent_entity;
            } else {
                break;
            }
        }

        calculated_transform

    }

    pub fn tick(&mut self) {
        self.tick_orderables();
        self.tick_transforms();
        self.tick_physics_bodies();
    }

}

struct RymdGame {
    model: RymdGameModel,
    view: RymdGameView,
    is_started: bool,
    is_running: bool,
    is_paused: bool
}

impl Game for RymdGame {

    fn is_running(&self) -> bool {
        self.is_running
    }

    fn is_paused(&self) -> bool {
        self.is_paused
    }

    fn start_game(&mut self, lockstep: &LockstepClient) {

        if self.is_started {

            self.is_running = true;
            self.is_paused = false;

        } else {

            let game_parameters = if lockstep.is_singleplayer() {
                RymdGameParameters { players: vec![RymdGamePlayer { id: lockstep.peer_id() }] }
            } else {
                let game_players = lockstep.peers().iter().map(|client| RymdGamePlayer { id: client.id } ).collect();
                RymdGameParameters { players: game_players }
            };

            self.model.start(game_parameters);
            
            self.is_running = true;
            self.is_started = true;
            self.is_paused = false;

        }

    }

    fn resume_game(&mut self) {
        self.is_paused = false;
    }

    fn pause_game(&mut self) {
        self.is_paused = true;
    }

    fn stop_game(&mut self) {
        self.is_running = false;
        self.is_started = false;
        self.is_paused = false;
        self.model.stop()
    }

    fn handle_message(&mut self, peer_id: PeerID, message: &str) {    
        self.model.handle_message(message);
    }

    fn update(&mut self, debug: &mut DebugText, lockstep: &mut LockstepClient) {
        self.model.tick();
        self.view.update(&mut self.model);    
    }

    fn update_view(&mut self, debug: &mut DebugText, lockstep: &mut LockstepClient) {
        self.view.tick(&mut self.model.world, lockstep);
    }

    fn draw(&mut self, debug: &mut DebugText) {
        self.view.draw(&mut self.model.world, debug);
    }

    fn reset(&mut self) {
        self.stop_game();
    }

    async fn load_resources(&mut self) {
        self.view.load_resources().await;
    }

}

impl RymdGame {
    fn new() -> RymdGame {
        RymdGame {
            model: RymdGameModel::new(),
            view: RymdGameView::new(),
            is_running: false,
            is_started: false,
            is_paused: false
        }
    }
}

#[macroquad::main("multi-rymd")]
async fn main() {

    let mut app = ApplicationState::new("multi-rymd", RymdGame::new());

    app.set_target_host("94.13.52.142");
    app.set_debug_text_colour(WHITE);
    app.load_resources().await;

    loop {

        app.handle_messages();
        clear_background(Color::from_hex(0x181425));

        app.update();
        app.draw();

        next_frame().await;

    }

}