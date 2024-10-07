use core::f32;
use std::collections::{HashMap, HashSet};

use hecs::Entity;
use macroquad::math::{ivec2, IVec2, Rect, Vec2};

pub struct SpatialQueryManager {
    entities: HashSet<Entity>,
    buckets: HashMap<IVec2, Vec<Entity>>,
    bucket_size: i32
}

impl SpatialQueryManager {

    pub fn new(bucket_size: i32) -> SpatialQueryManager {
        SpatialQueryManager {
            entities: HashSet::new(),
            buckets: HashMap::new(),
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
    
    fn get_matching_bucket(&mut self, position: Vec2) -> Option<(IVec2, &mut Vec<Entity>)> {
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
        if let Some((bucket_position, bucket)) = self.get_matching_bucket(position) {

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
        if let Some((bucket_position, bucket)) = self.get_matching_bucket(position) {
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

    pub fn entities_within_rect(&self, search_bounds: Rect) -> impl Iterator::<Item = Entity> + '_ {

        let bucket_size = self.bucket_size;

        self.buckets.iter()
            .filter(move |(&bucket_position, bucket)| Self::get_bucket_bounds(bucket_position, bucket_size).overlaps(&search_bounds))
            .flat_map(|(bucket_position, bucket)| bucket)
            .map(|e| *e)

    }

    pub fn entities_within(&self, min: Vec2, max: Vec2) -> impl Iterator::<Item = Entity> + '_  {

        let bucket_size = self.bucket_size;
        let position_bounds = Rect::new(min.x, min.y, max.x - min.x, max.y - max.y);
        self.entities_within_rect(position_bounds)

    }

    pub fn entities_near(&self, position: Vec2, radius: f32) -> impl Iterator::<Item = Entity> + '_  {
        
        let bucket_size = self.bucket_size;
        let position_bounds = Rect::new(position.x - radius * 0.5, position.y - radius * 0.5, radius * 2.0, radius * 2.0);
        self.entities_within_rect(position_bounds)

    }

    pub fn buckets(&self) -> impl Iterator::<Item = (&IVec2, &Vec<Entity>)> {
        self.buckets.iter()
    }

}