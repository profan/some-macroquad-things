use std::collections::VecDeque;
use hecs::{Entity, World};
use macroquad::math::{Vec2, Rect};
use utility::{Kinematic, RotatedBy, SteeringParameters};
use lockstep_client::step::PeerID;

use crate::model::BlueprintID;
use super::{GameOrder, PhysicsBody, GameOrderType};

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
    pub bounds: Rect
}

impl PhysicsBody for DynamicBody {

    fn enabled(&self) -> bool {
        self.is_enabled
    }

    fn bounds(&self) -> Rect {
        self.bounds.offset(self.kinematic.position)
    }

    fn position(&self) -> Vec2 {
        self.kinematic.position
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
            GameOrderType::Order => self.build_order_queue.pop_first_canceled_order(),
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

}

#[derive(Clone)]
pub struct Ship {
    pub turn_rate: f32,
    pub thrusters: Vec<Entity>
}

impl Ship {
    pub fn new(turn_rate: f32) -> Ship {
        Ship { turn_rate, thrusters: Vec::new() }
    }
}

#[derive(Clone)]
pub struct Health {
    full_health: i32,
    current_health: i32,
    last_health: i32
}

impl Health {
    pub fn new(full_health: i32) -> Health {
        Health { full_health, current_health: full_health, last_health: full_health }
    }

    pub fn new_with_current_health(full_health: i32, current_health: i32) -> Health {
        Health { full_health, current_health, last_health: current_health }
    }

    pub fn heal_to_full_health(&mut self) {
        self.current_health = self.full_health;
        self.last_health = self.current_health;
    }

    pub fn damage(&mut self, value: i32) {
        self.last_health = self.current_health;
        self.current_health -= value;
    }

    pub fn heal(&mut self, value: i32) {
        self.damage(-value);
    }

    pub fn current_health(&self) -> i32 {
        self.current_health
    }

    pub fn last_health(&self) -> i32 {
        self.last_health
    }

    pub fn full_health(&self) -> i32 {
        self.full_health
    }

    pub fn is_at_full_health(&self) -> bool {
        self.current_health >= self.full_health
    }

    pub fn current_health_fraction(&self) -> f32 {
        self.full_health as f32 / self.current_health as f32
    }
}

pub struct Controller {
    pub id: PeerID
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

#[derive(Clone)]
pub struct BlueprintIdentity {
    pub blueprint_id: BlueprintID
}

pub fn current_health(world: &World, entity: Entity) -> i32 {
    world.get::<&Health>(entity).unwrap().current_health
}

pub fn max_health(world: &World, entity: Entity) -> i32 {
    world.get::<&Health>(entity).unwrap().full_health
}