use std::{ops::{Deref, DerefMut}, collections::{HashMap, hash_map::Entry}, thread::current, f32::consts::PI};

use macroquad::prelude::*;
use macroquad::ui::*;

use utility::{draw_cube_ex, draw_cube_wires_ex, create_camera, draw_grid_ex};

const INVALID_ENTITY_ID: i32 = i32::MAX;

#[derive(Copy, Clone, PartialEq, Hash, Eq)]
struct EntityId(i32);

impl Deref for EntityId {
    type Target = i32;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for EntityId {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Default for EntityId {
    fn default() -> Self {
        Self(INVALID_ENTITY_ID)
    }
}

#[derive(Default)]
struct Transform {
    local_position: Vec3,
    local_rotation: Quat
}

struct Hierarchy {

    /// maps children to their parents, if the given entity has any parent transform
    hierarchy: HashMap<EntityId, EntityId>,

    /// stores the transforms of each entity that has a transform, if it has one
    transforms: HashMap<EntityId, Transform>
    
}

impl Hierarchy {

    pub fn new() -> Hierarchy {
        Hierarchy {
            hierarchy: HashMap::new(),
            transforms: HashMap::new()
        }
    }

    pub fn get_local_position(&self, id: EntityId) -> Vec3 {
        self.transforms[&id].local_position
    }

    pub fn get_local_rotation(&self, id: EntityId) -> Quat {
        self.transforms[&id].local_rotation
    }

    pub fn get_world_rotation(&self, id: EntityId) -> Quat {

        let local_rotation = self.transforms[&id].local_rotation;

        // if no parent, just return local rotation
        if self.hierarchy.contains_key(&id) == false {
            return local_rotation;
        }

        let mut current_parent = self.hierarchy.get(&id);
        let mut current_rotation = Quat::IDENTITY;

        while let Some(parent) = current_parent {
            current_rotation = self.get_local_rotation(*parent) * current_rotation;
            current_parent = self.hierarchy.get(parent);
        }

        current_rotation * local_rotation

    }

    pub fn get_world_position(&self, id: EntityId) -> Vec3 {

        let local_position = self.transforms[&id].local_position;

        // if no parent, just return local position
        if self.hierarchy.contains_key(&id) == false {
            return local_position
        }

        let mut current_parent = self.hierarchy.get(&id);
        let mut world_position = local_position;

        while let Some(parent) = current_parent {
            let parent_position = self.get_local_position(*parent);
            let parent_rotation = self.get_local_rotation(*parent);
            world_position = parent_rotation * world_position + parent_position;
            current_parent = self.hierarchy.get(parent);
        }

        world_position

    }

    pub fn get_parent(&self, id: EntityId) -> Option<EntityId> {
        self.hierarchy.get(&id).copied()
    }

    pub fn set_world_position(&mut self, id: EntityId, world_position: Vec3) {
        self.set_local_position(id, self.world_to_local(id, world_position));
    }

    pub fn set_world_rotation(&mut self, id: EntityId, world_rotation: Quat) {
        let parent_world_rotation = self.hierarchy.get(&id).and_then(|p| Some(self.get_world_rotation(*p))).unwrap_or(Quat::IDENTITY);
        self.set_local_rotation(id, world_rotation * parent_world_rotation.conjugate());
    }

    pub fn set_local_position(&mut self, id: EntityId, local_position: Vec3) {
        match self.transforms.entry(id) {
            Entry::Occupied(o) => o.into_mut().local_position = local_position,
            Entry::Vacant(v) => { v.insert(Transform { local_position, ..Default::default() }); }
        }
    }

    pub fn set_local_rotation(&mut self, id: EntityId, local_rotation: Quat) {
        match self.transforms.entry(id) {
            Entry::Occupied(o) => o.into_mut().local_rotation = local_rotation,
            Entry::Vacant(v) => { v.insert(Transform { local_rotation, ..Default::default() }); }
        }
    }

    pub fn set_parent(&mut self, id: EntityId, parent_id: EntityId) {
        self.hierarchy.insert(id, parent_id);
    }

    pub fn world_to_local(&self, id: EntityId, world_position: Vec3) -> Vec3 {
        let parent_world_position = if let Some(p) = self.get_parent(id) { self.get_world_position(p) } else { Vec3::ZERO };
        let parent_world_rotation = if let Some(p) = self.get_parent(id) { self.get_world_rotation(p) } else { Quat::IDENTITY };
        (parent_world_rotation.inverse() * world_position) - parent_world_position
    }

    pub fn local_to_world(&self, id: EntityId, local_position: Vec3) -> Vec3 {
        let parent_world_position = if let Some(p) = self.get_parent(id) { self.get_world_position(p) } else { Vec3::ZERO };
        let parent_world_rotation = if let Some(p) = self.get_parent(id) { self.get_world_rotation(p) } else { Quat::IDENTITY };
        (parent_world_rotation * local_position) + parent_world_position
    }

}

struct World {
    entity_id: EntityId,
    entities: Vec<EntityId>,
    hierarchy: Hierarchy
}

impl World {

    pub fn new() -> World {
        World {
            entity_id: EntityId(0),
            entities: Vec::new(),
            hierarchy: Hierarchy::new()
        }
    }

    pub fn create_entity(&mut self) -> EntityId {
        let current_entity_id = self.entity_id;
        self.entities.push(current_entity_id);
        (*self.entity_id) += 1;
        current_entity_id
    }

}

fn spawn_cube_entity_with_parent(world: &mut World, local_position: Vec3, local_rotation: Quat, parent: EntityId) -> EntityId {

    let new_entity = spawn_cube_entity(world, local_position, local_rotation);
    world.hierarchy.set_parent(new_entity, parent);
    new_entity

}

fn spawn_cube_entity(world: &mut World, local_position: Vec3, local_rotation: Quat) -> EntityId {

    let new_entity = world.create_entity();
    world.hierarchy.set_local_position(new_entity, local_position);
    world.hierarchy.set_local_rotation(new_entity, local_rotation);
    new_entity

}

fn draw_cube_entity(world: &World, entity_id: EntityId) {

    let world_position = world.hierarchy.get_world_position(entity_id);
    let world_rotation = world.hierarchy.get_world_rotation(entity_id);
    draw_cube_wires_ex(world_position, world_rotation, Vec3::ONE, BLACK);
    draw_grid_ex(world_position - world_rotation * vec3(0.0, 0.5, 0.0), world_rotation, 4, 0.5, RED, GRAY);
    
    let other_world_position = world.hierarchy.local_to_world(entity_id, world.hierarchy.get_local_position(entity_id));
    // let world_rotation = world.hierarchy.get_world_rotation(*entity_id);
    draw_cube_wires_ex(other_world_position, world_rotation, Vec3::ONE, GREEN)
    
}

#[macroquad::main("transformy")]
async fn main() {

    let mut world = World::new();
    
    let start = vec3(4.0, 0.0, 0.0);

    let e1 = spawn_cube_entity(&mut world, start + vec3(0.0, 0.5, 0.0), Quat::from_rotation_y(PI / 4.0));
    let e2 = spawn_cube_entity_with_parent(&mut world, vec3(0.0, 2.0, 0.0), Quat::from_rotation_x(PI / 4.0), e1);
    let _e3 = spawn_cube_entity_with_parent(&mut world, vec3(0.0, 2.0, 0.0), Quat::from_rotation_y(PI / 4.0), e2);

    let mut going_right = true;
    let mut current_camera_x = 4.0;

    let camera_x_speed = 8.0;
    let camera_x_min = -16.0;
    let camera_x_max = 16.0;

    loop {

        let dt = get_frame_time();

        if going_right {
            current_camera_x += camera_x_speed * dt;
            if current_camera_x >= camera_x_max {
                current_camera_x = camera_x_max;
                going_right = false;
            }
        } else {
            current_camera_x -= camera_x_speed * dt;
            if current_camera_x <= camera_x_min {
                current_camera_x = camera_x_min;
                going_right = true;
            }
        }

        set_camera(&create_camera(vec3(current_camera_x, 8.0, -12.0), Vec3::Y, start));

        clear_background(WHITE);

        draw_grid(16, 1.0, RED, GRAY);

        for entity_id in &world.entities {
            draw_cube_entity(&world, *entity_id);
        }

        set_default_camera();

        next_frame().await;

    }

}
