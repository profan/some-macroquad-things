use std::collections::VecDeque;
use hecs::{CommandBuffer, Entity, World};
use macroquad::{color::Color, input::KeyCode, math::{Rect, Vec2}};
use utility::{Kinematic, RotatedBy, SteeringParameters};
use lockstep_client::step::PeerID;

use crate::PlayerID;
use super::{BeamParameters, Blueprints, BulletParameters, Cost, GameOrder, GameOrderType, PhysicsBody};

#[derive(Clone)]
pub struct Thruster {
    pub kind: ThrusterKind,
    pub direction: Vec2,
    pub angle: f32,
    pub power: f32
}

#[derive(Clone, Copy, PartialEq)]
pub enum ThrusterKind {
    Main,
    Attitude,
}

#[derive(Clone)]
pub struct Input {
    pub forward: bool,
    pub backward: bool,
    pub turn_left: bool,
    pub turn_right: bool,
    pub fast: bool
}

#[derive(Debug, Default, Clone, Copy)]
pub struct PreviousTransform {
    pub transform: Transform
}

#[derive(Debug, Default, Clone, Copy)]
pub struct Transform {
    pub world_position: Vec2,
    pub world_rotation: f32,
    pub local_position: Vec2,
    pub local_rotation: f32,
    pub parent: Option<Entity>
}

impl Transform {

    pub fn new(position: Vec2, rotation: f32, parent: Option<Entity>) -> Transform {
        Transform {
            world_position: position,
            world_rotation: rotation,
            local_position: position,
            local_rotation: rotation,
            parent
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

        while let Ok(current_transform) = world.get::<&Transform>(current_entity) && let Some(parent_entity) = current_transform.parent {
            if let Ok(parent_transform) = world.get::<&Transform>(parent_entity) {
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
pub struct DynamicBody {
    pub is_static: bool,
    pub is_enabled: bool,
    pub kinematic: Kinematic,
    pub bounds: Rect,
    pub mask: u64
}

#[derive(Clone)]
pub struct DynamicBodyCallback {
    pub on_collision: fn(&World, &mut CommandBuffer, Entity, Entity, &DynamicBody) -> ()
}

impl PhysicsBody for DynamicBody {

    fn enabled(&self) -> bool {
        self.is_enabled
    }

    fn local_bounds(&self) -> Rect {
        self.bounds
    }

    fn bounds(&self) -> Rect {
        self.bounds.offset(self.kinematic.position)
    }

    fn position(&self) -> Vec2 {
        self.kinematic.position
    }

    fn visual_position(&self) -> Vec2 {
        self.kinematic.position - self.bounds.size() / 2.0
    }

    fn local_physics_bounds(&self) -> Rect {
        self.bounds.offset(-self.bounds.size() / 2.0)
    }
    
    fn physics_bounds(&self) -> Rect {
        self.bounds().offset(-self.bounds.size() / 2.0)
    }
    
    fn orientation(&self) -> f32 {
        self.kinematic.orientation
    }

    fn velocity(&self) -> Vec2 {
        self.kinematic.velocity
    }

    fn angular_velocity(&self) -> f32 {
        self.kinematic.angular_velocity
    }

    fn friction(&self) -> f32 {
        self.kinematic.friction_value
    }

    fn mass(&self) -> f32 {
        self.kinematic.mass
    }

    fn position_mut(&mut self) -> &mut Vec2 {
        &mut self.kinematic.position
    }

    fn velocity_mut(&mut self) -> &mut Vec2 {
        &mut self.kinematic.velocity
    }

    fn angular_velocity_mut(&mut self) -> &mut f32 {
        &mut self.kinematic.angular_velocity
    }

    fn friction_mut(&mut self) -> &mut f32 {
        &mut self.kinematic.friction_value
    }

    fn mass_mut(&mut self) -> &mut f32 {
        &mut self.kinematic.mass
    }

    fn apply_impulse(&mut self, impulse: Vec2, offset: Vec2) {
        self.kinematic.velocity += impulse
    }

}

#[derive(Clone)] 
pub struct Sprite {
    pub texture: String
}

impl Sprite {
    pub fn new<T>(texture: T) -> Sprite
        where T: Into<String>
    {
        Sprite { texture: texture.into() }
    }
}

#[derive(Clone)]
pub struct AnimatedSprite {
    pub texture: String,
    pub current_frame: i32,
    pub h_frames: i32
}

#[derive(Clone)]
pub struct Orderable {
    build_order_queue: OrderQueue,
    order_queue: OrderQueue
}

#[derive(Clone)]
struct OrderQueue {
    canceled_orders: VecDeque<GameOrder>,
    orders: VecDeque<GameOrder>
}

impl OrderQueue {

    pub fn new() -> OrderQueue {
        OrderQueue { canceled_orders: VecDeque::new(), orders: VecDeque::new() }
    }

    /// Returns the order first in the queue, if any.
    pub fn first_order(&self) -> Option<&GameOrder> {
        self.orders.front()
    }

    /// Pops and returns the first order in the queue.
    pub fn pop_first_order(&mut self) -> Option<GameOrder> {
        self.orders.pop_front()
    }
    
    /// Pops and returns the first order in the queue of canceled orders.
    pub fn pop_first_canceled_order(&mut self) -> Option<GameOrder> {
        self.canceled_orders.pop_front()
    }
    
    /// Returns a reference to the collection of enqueued orders.
    pub fn orders(&self) -> &VecDeque<GameOrder> {
        &self.orders
    }

    /// Returns a reference to the collection of canceled orders.
    pub fn canceled_orders(&self) -> &VecDeque<GameOrder> {
        &self.canceled_orders
    }

    /// Returns true if there's currently no orders to process.
    pub fn is_queue_empty(&self) -> bool {
        self.orders.is_empty()
    }
    
    /// Cancel the current order.
    pub fn cancel_order(&mut self) {
        let canceled_order = self.orders.pop_front();
        if let Some(order) = canceled_order {
            self.canceled_orders.push_front(order);
        }
    }

    /// Push a new order to the front of the queue.
    pub fn push_order(&mut self, order: GameOrder) {
        self.orders.push_front(order);
    }

    /// Enqueues the order at the end of the queue.
    pub fn queue_order(&mut self, order: GameOrder) {
        self.orders.push_back(order);
    }

    /// Cancel all orders in the queue.
    pub fn cancel_orders(&mut self) {
        self.canceled_orders.extend(self.orders.iter());
        self.orders.clear();
    }

    /// Clear the queue of canceled orders.
    pub fn clear_canceled_orders(&mut self) {
        self.canceled_orders.clear();
    }

}

impl Orderable {

    pub fn new() -> Orderable {
        Orderable {
            order_queue: OrderQueue::new(),
            build_order_queue: OrderQueue::new()
        }
    }

    /// Returns the order first in the queue, if any.
    pub fn first_order(&self, order_type: GameOrderType) -> Option<&GameOrder> {
        match order_type {
            GameOrderType::Order => self.order_queue.first_order(),
            GameOrderType::Construct => self.build_order_queue.first_order(),
        }
    }

    /// Pops and returns the first order in the queue.
    pub fn pop_first_order(&mut self, order_type: GameOrderType) -> Option<GameOrder> {
        match order_type {
            GameOrderType::Order => self.order_queue.pop_first_order(),
            GameOrderType::Construct => self.build_order_queue.pop_first_order(),
        }
    }
    
    /// Pops and returns the first order in the queue of canceled orders.
    pub fn pop_first_canceled_order(&mut self, order_type: GameOrderType) -> Option<GameOrder> {
        match order_type {
            GameOrderType::Order => self.order_queue.pop_first_canceled_order(),
            GameOrderType::Construct => self.build_order_queue.pop_first_canceled_order(),
        }
    }
    
    /// Returns a reference to the collection of enqueued orders.
    pub fn orders(&self, order_type: GameOrderType) -> &VecDeque<GameOrder> {
        match order_type {
            GameOrderType::Order => self.order_queue.orders(),
            GameOrderType::Construct => self.build_order_queue.orders(),
        }
    }

    /// Returns a reference to the collection of canceled orders.
    pub fn canceled_orders(&self, order_type: GameOrderType) -> &VecDeque<GameOrder> {
        match order_type {
            GameOrderType::Order => self.order_queue.canceled_orders(),
            GameOrderType::Construct => self.build_order_queue.canceled_orders(),
        }
    }

    /// Returns true if there's currently no orders to process.
    pub fn is_queue_empty(&self, order_type: GameOrderType) -> bool {
        match order_type {
            GameOrderType::Order => self.order_queue.is_queue_empty(),
            GameOrderType::Construct => self.build_order_queue.is_queue_empty(),
        }
    }
    
    /// Cancel the current order.
    pub fn cancel_order(&mut self, order_type: GameOrderType) {
        match order_type {
            GameOrderType::Order => self.order_queue.cancel_order(),
            GameOrderType::Construct => self.build_order_queue.cancel_order(),
        }
    }

    /// Push a new order to the front of the queue.
    pub fn push_order(&mut self, order: GameOrder) {
        match order.order_type() {
            GameOrderType::Order => self.order_queue.push_order(order),
            GameOrderType::Construct => self.build_order_queue.push_order(order)
        }
    }

    /// Enqueues the order at the end of the queue.
    pub fn queue_order(&mut self, order: GameOrder) {
        match order.order_type() {
            GameOrderType::Order => self.order_queue.queue_order(order),
            GameOrderType::Construct => self.build_order_queue.queue_order(order),
        }
    }

    /// Cancel all orders in the queue.
    pub fn cancel_orders(&mut self, order_type: GameOrderType) {
        match order_type {
            GameOrderType::Order => self.order_queue.cancel_orders(),
            GameOrderType::Construct => self.build_order_queue.cancel_orders()
        }
    }

    /// Clear the queue of canceled orders.
    pub fn clear_canceled_orders(&mut self, order_type: GameOrderType) {
        match order_type {
            GameOrderType::Order => self.order_queue.clear_canceled_orders(),
            GameOrderType::Construct => self.build_order_queue.clear_canceled_orders(),
        }
    }

    /// Returns true if there's any pending orders in the given order queue.
    pub fn has_pending_orders(&self, order_type: GameOrderType) -> bool {
        match order_type {
            GameOrderType::Order => self.orders(order_type).is_empty() == false,
            GameOrderType::Construct => self.orders(order_type).is_empty() == false
        }
    }

    /// Returns true if there's any pending orders in the given order queue.
    pub fn number_of_pending_orders(&self, order_type: GameOrderType) -> i32 {
        match order_type {
            GameOrderType::Order => self.orders(order_type).len() as i32,
            GameOrderType::Construct => self.orders(order_type).len() as i32
        }
    }

}

#[derive(Clone)]
pub struct Ship {
    pub thrusters: Vec<Entity>
}

impl Ship {
    pub fn new() -> Ship {
        Ship { thrusters: Vec::new() }
    }
}

#[derive(Clone)]
pub struct Health {

    full_health: f32,
    current_health: f32,
    last_health: f32,

    pub on_death: Option<fn(world: &World, buffer: &mut CommandBuffer, entity: Entity) -> ()>

}

impl Health {
    pub fn new(full_health: f32) -> Health {
        Health { full_health, current_health: full_health, last_health: full_health, on_death: None }
    }

    pub fn new_with_callback(full_health: f32, on_death_fn: fn(world: &World, buffer: &mut CommandBuffer, entity: Entity) -> ()) -> Health {
        Health { full_health, current_health: full_health, last_health: full_health, on_death: Some(on_death_fn) }
    }

    pub fn new_with_current_health(full_health: f32, current_health: f32) -> Health {
        Health { full_health, current_health, last_health: current_health, on_death: None }
    }

    pub fn new_with_current_health_and_callback(full_health: f32, current_health: f32, on_death_fn: fn(world: &World, buffer: &mut CommandBuffer, entity: Entity) -> ()) -> Health {
        Health { full_health, current_health, last_health: current_health, on_death: Some(on_death_fn) }
    }

    pub fn heal_to_full_health(&mut self) {
        self.current_health = self.full_health;
        self.last_health = self.current_health;
    }

    pub fn damage(&mut self, value: f32) {
        self.last_health = self.current_health;
        self.current_health -= value;
    }

    pub fn kill(&mut self) {
        self.last_health = self.current_health;
        self.current_health = 0.0;
    }

    pub fn heal(&mut self, value: f32) {
        self.damage(-value);
    }

    pub fn set_health(&mut self, value: f32) {
        self.last_health = self.current_health;
        self.current_health = value;
    }

    pub fn current_health(&self) -> f32 {
        self.current_health
    }

    pub fn last_health(&self) -> f32 {
        self.last_health
    }

    pub fn full_health(&self) -> f32 {
        self.full_health
    }

    pub fn is_at_full_health(&self) -> bool {
        self.current_health >= self.full_health
    }
    
    pub fn is_at_or_below_zero_health(&self) -> bool {
        self.current_health <= 0.0
    }

    pub fn current_health_fraction(&self) -> f32 {
        self.current_health / self.full_health
    }
}

pub struct Controller {
    pub id: PeerID
}

pub type BlueprintID = i32;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EntityState {
    Ghost,
    Destroyed,
    Constructed,
    Inactive
}

#[derive(Clone, Copy)]
pub enum UnitState {
    Frozen,
    Destroyed,
    Alive
}

#[derive(Clone)]
pub struct Steering {
    pub parameters: SteeringParameters
}

#[derive(Debug, Clone)]
pub struct Building;

#[derive(Debug, Clone)]
pub struct Constructor {
    pub current_target: Option<Entity>,
    pub constructibles: Vec<BlueprintID>,
    pub build_range: i32,
    pub build_speed: i32,
    pub beam_offset: Vec2,
    pub can_assist: bool
}

impl Constructor {
    pub fn is_constructing(&self) -> bool {
        self.current_target.is_some()
    }

    pub fn has_blueprint(&self, id: BlueprintID) -> bool {
        self.constructibles.contains(&id)
    }
}

#[derive(Debug, Clone)]
pub struct Extractor {
    pub current_target: Option<Entity>,
    pub last_target: Option<Entity>,
    pub extraction_range: i32,
    pub extraction_speed: i32,
    pub beam_offset: Vec2,
    pub is_searching: bool,
    pub is_active: bool
}

impl Extractor {
    pub fn is_extracting(&self) -> bool {
        self.is_active
    }
}

pub struct ResourceSource {
    pub total_metal: f32,
    pub total_energy: f32,
    pub current_metal: f32,
    pub current_energy: f32,
    pub is_finite: bool
}

impl ResourceSource {

    pub fn is_exhausted(&self) -> bool {
        self.is_finite && (self.current_metal - 0.1) <= 0.0 && (self.current_energy - 0.1) <= 0.0
    }

    pub fn is_occupied(&self) -> bool {
        (self.current_metal - 0.1) <= 0.0 && (self.current_energy - 0.1) <= 0.0
    }

    pub fn new_metal_source(metal: f32) -> ResourceSource {
        ResourceSource { total_metal: metal, total_energy: 0.0, current_metal: metal, current_energy: 0.0, is_finite: false }
    }

    pub fn new_energy_source(energy: f32) -> ResourceSource {
        ResourceSource { total_metal: 0.0, total_energy: energy, current_metal: 0.0, current_energy: energy, is_finite: false }
    }

    pub fn new_finite_metal_source(metal: f32) -> ResourceSource {
        ResourceSource { total_metal: metal, total_energy: 0.0, current_metal: metal, current_energy: 0.0, is_finite: true }
    }

    pub fn new_finite_energy_source(energy: f32) -> ResourceSource {
        ResourceSource { total_metal: 0.0, total_energy: energy, current_metal: 0.0, current_energy: energy, is_finite: true }
    }

    pub fn new_source(metal: f32, energy: f32) -> ResourceSource {
        ResourceSource { total_metal: metal, total_energy: energy, current_metal: metal, current_energy: energy, is_finite: false }
    }

    pub fn new_finite_source(metal: f32, energy: f32) -> ResourceSource {
        ResourceSource { total_metal: metal, total_energy: energy, current_metal: metal, current_energy: energy, is_finite: true }
    }

}

#[derive(Debug, Clone)]
pub struct Spawner {
    /// This position is a local offset from the position of the transform this is attached to, and is where units will spawn.
    pub position: Vec2
}

#[derive(Debug, Clone)]
pub struct Blueprint {
    pub id: BlueprintID,
    pub name: String,
    pub texture: String,
    pub shortcut: KeyCode,
    pub constructor: fn(&mut World, PlayerID, Vec2) -> Entity,
    pub is_building: bool,
    pub cost: Cost
}

#[derive(Clone)]
pub struct BlueprintIdentity {
    pub blueprint_id: BlueprintID
}

impl BlueprintIdentity {
    pub fn new(id: Blueprints) -> BlueprintIdentity {
        BlueprintIdentity { blueprint_id: id as i32 }
    }
}

pub fn current_health(world: &World, entity: Entity) -> f32 {
    world.get::<&Health>(entity).unwrap().current_health
}

pub fn max_health(world: &World, entity: Entity) -> f32 {
    world.get::<&Health>(entity).unwrap().full_health
}

pub struct RotationTarget {
    pub target: Option<Vec2>
}

pub struct MovementTarget { 
    pub target: Option<Vec2>
}

pub struct ExtractionTarget {
    pub target: Option<Vec2>
}

pub struct Attacker {
    pub target: Option<Entity>,
    pub range: f32
}

impl Attacker {
    pub fn new(range: f32) -> Attacker {
        Attacker { target: None, range }
    }
}

pub struct Attackable;

pub struct Projectile {
    pub damage: f32,
    pub lifetime: f32,
    pub velocity: f32
}

pub struct Beam {
    pub position: Vec2,
    pub target: Vec2,
    pub damage: f32,
    pub fired: bool,
    pub color: Color
}

pub struct ProjectileWeapon {

    pub offset: Vec2,
    pub fire_rate: f32,
    pub deviation: f32,
    pub cooldown: f32,

    /// Angle from the forward of this weapon the describes the arc within which it can fire!
    pub fire_arc: f32,

    pub projectile: BulletParameters

}

pub struct BeamWeapon {

    pub offset: Vec2,
    pub fire_rate: f32,
    pub deviation: f32,
    pub cooldown: f32,

    /// Angle from the forward of this weapon the describes the arc within which it can fire!
    pub fire_arc: f32,

    pub beam: BeamParameters

}

pub struct Effect {
    pub total_lifetime: f32,
    pub lifetime: f32
}

pub struct Impact;

impl Effect {
    pub fn new(lifetime: f32) -> Effect {
        Effect {
            total_lifetime: lifetime,
            lifetime
        }
    }

    pub fn current_lifetime_fraction(&self) -> f32 {
        self.lifetime / self.total_lifetime
    }
}

pub struct Commander;