use core::f32;
use std::{cmp::Ordering};

use fnv::{FnvHashMap, FnvHashSet};
use hecs::{Entity, World};
use macroquad::math::{ivec2, IVec2, Rect, Vec2};

use super::get_entity_position;

pub struct SpatialQueryManager {
    entities: FnvHashSet<Entity>,
    buckets: FnvHashMap<IVec2, Vec<Entity>>,
    bucket_size: i32
}

impl SpatialQueryManager {

    pub fn new(bucket_size: i32) -> SpatialQueryManager {
        SpatialQueryManager {
            entities: FnvHashSet::default(),
            buckets: FnvHashMap::default(),
            bucket_size
        }
    }

    pub fn get_bucket_bounds(bucket_position: IVec2, bucket_size: i32) -> Rect {
        Rect {
            x: bucket_position.x as f32,
            y: bucket_position.y as f32,
            w: bucket_size as f32,
            h: bucket_size as f32
        }
    }

    pub fn is_position_in_bucket(bucket_position: IVec2, bucket_size: i32, position: Vec2) -> bool {
        Self::get_bucket_bounds(bucket_position, bucket_size).contains(position)
    }

    pub fn get_clamped_bucket_world_position(&self, position: Vec2) -> IVec2 {
        let position_floored = position.as_ivec2();
        let mut position_clamped_x = (position_floored.x / self.bucket_size) * self.bucket_size;
        let mut position_clamped_y = (position_floored.y / self.bucket_size) * self.bucket_size;
        if position.x < 0.0 {
            position_clamped_x -= self.bucket_size;
        }
        if position.y < 0.0 {
            position_clamped_y -= self.bucket_size;
        }
        ivec2(position_clamped_x, position_clamped_y)
    }

    fn get_matching_bucket(&self, position: Vec2) -> Option<(IVec2, &Vec<Entity>)> {
        let matching_bucket_position = self.get_clamped_bucket_world_position(position);
        let matching_bucket = self.buckets.get(&matching_bucket_position);
        if let Some(matching_bucket) = matching_bucket {
            Some((matching_bucket_position, matching_bucket))
        } else {
            None
        }
    }
    
    fn get_matching_bucket_mut(&mut self, position: Vec2) -> Option<(IVec2, &mut Vec<Entity>)> {
        let matching_bucket_position = self.get_clamped_bucket_world_position(position);
        let matching_bucket = self.buckets.get_mut(&matching_bucket_position);
        if let Some(matching_bucket) = matching_bucket {
            Some((matching_bucket_position, matching_bucket))
        } else {
            None
        }
    }

    fn create_new_bucket(&mut self, position: Vec2, entity: Entity) {
        let clamped_bucket_position = self.get_clamped_bucket_world_position(position);
        self.buckets.insert(clamped_bucket_position, vec![entity]);
    }
    
    pub fn is_entity_registered(&self, entity: Entity) -> bool {
        self.entities.contains(&entity)
    }

    pub fn add_entity(&mut self, entity: Entity, position: Vec2) {

        let bucket_size = self.bucket_size;
        if let Some((bucket_position, bucket)) = self.get_matching_bucket_mut(position) {

            // if there's an existing bucket and we're in it, just add ourselves there
            bucket.push(entity);

        } else {

            // otherwise create a new bucket with us in it :)
            self.create_new_bucket(position, entity);

        }

        self.entities.insert(entity);

    }

    pub fn remove_entity(&mut self, entity: Entity, position: Vec2) {

        let bucket_size = self.bucket_size;
        if let Some((bucket_position, bucket)) = self.get_matching_bucket_mut(position) {
            bucket.retain(|e| *e != entity);
        }

        self.entities.remove(&entity);

    }

    pub fn update_entity_position(&mut self, entity: Entity, old_position: Vec2, new_position: Vec2) {

        let old_bucket_position = self.get_clamped_bucket_world_position(old_position);
        let new_bucket_position = self.get_clamped_bucket_world_position(new_position);

        if old_bucket_position != new_bucket_position {
            self.remove_entity(entity, old_position);
            self.add_entity(entity, new_position);
        }

    }

    pub fn entities_within_overlapping_line_segment_sorted_by<F>(&self, a: Vec2, b: Vec2, mut sort_fn: F) -> Vec<Entity>
        where F: FnMut(Entity, Entity) -> Option<Ordering>
    {

        let mut sorted_entities = Vec::new();
        sorted_entities.extend(self.entities_within_overlapping_line_segment(a, b));
        sorted_entities.sort_by(|&a, &b| sort_fn(a, b).unwrap());

        sorted_entities

    }

    pub fn entities_within_overlapping_line_segment(&self, a: Vec2, b: Vec2) -> Vec<Entity> {

        let distance_chunk_size = (self.bucket_size as f32) / 4.0;
        let number_of_chunks_to_test = a.distance(b) / distance_chunk_size;
        let vector_to_target = b - a;

        let mut last_bucket_position = IVec2::MAX;
        let mut entities_overlapping_segment = Vec::new();

        for i in 0..number_of_chunks_to_test as i32 {

            let current_factor = i as f32 / number_of_chunks_to_test;
            let current_position = a + (vector_to_target * current_factor);

            if let Some((bucket_position, bucket)) = self.get_matching_bucket(current_position) && bucket_position != last_bucket_position {
                entities_overlapping_segment.extend_from_slice(bucket.as_slice());
                last_bucket_position = bucket_position;
            }

        }

        entities_overlapping_segment

    }

    pub fn entities_within_rect_sorted_by<F>(&self, search_bounds: Rect, mut sort_fn: F) -> Vec<Entity>
        where F: FnMut(Entity, Entity) -> Option<Ordering>
    {

        let mut sorted_entities = Vec::new();
        sorted_entities.extend(self.entities_within_rect(search_bounds));
        sorted_entities.sort_by(|&a, &b| sort_fn(a, b).unwrap());

        sorted_entities

    }

    pub fn entities_within_rect(&self, search_bounds: Rect) -> impl Iterator::<Item = Entity> + '_ {

        let bucket_size = self.bucket_size;

        self.buckets.iter()
            .filter(move |(&bucket_position, bucket)| Self::get_bucket_bounds(bucket_position, bucket_size).overlaps(&search_bounds))
            .flat_map(|(bucket_position, bucket)| bucket)
            .copied()

    }

    pub fn entities_within_min_max_sorted_by<F>(&self, min: Vec2, max: Vec2, mut sort_fn: F) -> Vec<Entity>
        where F: FnMut(Entity, Entity) -> Option<Ordering>
    {

        let mut sorted_entities = Vec::new();
        sorted_entities.extend(self.entities_within_min_max(min, max));
        sorted_entities.sort_by(|&a, &b| sort_fn(a, b).unwrap());

        sorted_entities

    }

    pub fn entities_within_min_max(&self, min: Vec2, max: Vec2) -> impl Iterator::<Item = Entity> + '_  {

        let bucket_size = self.bucket_size;
        let position_bounds = Rect::new(min.x, min.y, max.x - min.x, max.y - min.y);
        self.entities_within_rect(position_bounds)

    }

    pub fn entities_within_radius_sorted_by<F>(&self, position: Vec2, radius: f32, mut sort_fn: F) -> Vec<Entity>
        where F: FnMut(Entity, Entity) -> Option<Ordering>
    {

        let mut sorted_entities = Vec::new();
        sorted_entities.extend(self.entities_within_radius(position, radius));
        sorted_entities.sort_by(|&a, &b| sort_fn(a, b).unwrap());

        sorted_entities

    }

    pub fn entities_within_radius(&self, position: Vec2, radius: f32) -> impl Iterator::<Item = Entity> + '_  {
        
        let bucket_size = self.bucket_size;
        let position_bounds = Rect::new(position.x - radius * 0.5, position.y - radius * 0.5, radius * 2.0, radius * 2.0);
        self.entities_within_rect(position_bounds)

    }

    pub fn buckets(&self) -> impl Iterator::<Item = (&IVec2, &Vec<Entity>)> {
        self.buckets.iter()
    }

}

/// Returns an ordering where the entity closer to the world position should be first.
pub fn entity_distance_sort_function(world: &World, world_position: Vec2, a: Entity, b: Entity) -> Option<Ordering> {
    let a_world_position = get_entity_position(world, a).expect("can't sort entities by position that don't have Transform!");
    let b_world_position = get_entity_position(world, b).expect("can't sort entities by position that don't have Transform!");
    let a_distance = a_world_position.distance_squared(world_position);
    let b_distance = b_world_position.distance_squared(world_position);
    a_distance.partial_cmp(&b_distance)
}