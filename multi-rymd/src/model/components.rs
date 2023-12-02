use std::collections::VecDeque;
use hecs::{Entity, World};
use macroquad::math::{Vec2, Rect};
use utility::{Kinematic, RotatedBy};
use lockstep_client::step::PeerID;

use super::{GameOrder, PhysicsBody};

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
    pub kinematic: Kinematic,
    pub bounds: Rect
}

impl PhysicsBody for DynamicBody {

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

    fn bounds_mut(&mut self) -> &mut Rect {
        &mut self.bounds
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

#[derive(Clone)]
pub struct AnimatedSprite {
    pub texture: String,
    pub current_frame: i32,
    pub h_frames: i32
}

#[derive(Clone)]
pub struct Orderable {
    pub orders: VecDeque<GameOrder>
}

impl Orderable {
    pub fn new() -> Orderable {
        Orderable { orders: VecDeque::new() }
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
    pub full_health: i32,
    pub health: i32
}

impl Health {
    pub fn new(health: i32) -> Health {
        Health { full_health: health, health: health }
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