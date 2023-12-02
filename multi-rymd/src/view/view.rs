use std::collections::HashMap;

use macroquad_particles::{EmitterConfig, Emitter};
use utility::{is_point_inside_rect, draw_texture_centered_with_rotation, set_texture_filter, draw_texture_centered_with_rotation_frame, DebugText, TextPosition, AsVector, RotatedBy, draw_arrow, draw_text_centered, draw_texture_centered, WithAlpha};
use lockstep_client::{step::LockstepClient, app::yakui_min_column};
use macroquad_particles::*;
use macroquad::prelude::*;
use hecs::*;
use yakui::Alignment;

use crate::PlayerID;
use crate::model::{BlueprintID, Building};
use crate::model::{RymdGameModel, Orderable, Transform, Sprite, AnimatedSprite, GameOrdersExt, DynamicBody, Thruster, Ship, ThrusterKind, Constructor, Controller, Health, get_entity_position};

use super::calculate_sprite_bounds;

fn building_state_to_alpha(building: Option<&Building>) -> f32 {
    if let Some(building) = building {
        match building.state {
            crate::model::BuildingState::Ghost => 0.75,
            crate::model::BuildingState::Destroyed => 1.0,
            crate::model::BuildingState::Constructed => 1.0,
        }
    } else {
        1.0
    }
}

struct ConstructionState {
    current_blueprint_id: Option<BlueprintID>
}

impl ConstructionState {
    fn new() -> ConstructionState {
        ConstructionState {
            current_blueprint_id: None
        }
    }

    fn is_previewing(&self) -> bool {
        self.current_blueprint_id.is_some()
    }

    fn preview_blueprint(&mut self, blueprint_id: BlueprintID) {
        self.current_blueprint_id = Some(blueprint_id)
    }

    fn cancel_blueprint(&mut self) {
        self.current_blueprint_id = None
    }

    fn finalize_blueprint(&mut self, model: &RymdGameModel, lockstep: &mut LockstepClient) {
        
        if let Some(blueprint_id) = self.current_blueprint_id {

            let should_add_to_queue = is_key_down(KeyCode::LeftShift);
            let current_build_position: Vec2 = mouse_position().into();
    
            for (e, (_t, _o, s, _c)) in model.world.query::<(&Transform, &Orderable, &Selectable, &Constructor)>().iter() {
                if s.is_selected {
                    lockstep.send_build_order(e, current_build_position, blueprint_id, should_add_to_queue);
                    println!("[RymdGameView] attempted to send build order for position: {} and blueprint: {}", current_build_position, blueprint_id);
                }
            }

            self.current_blueprint_id = None;

        }

    }

    fn tick_and_draw(&mut self, model: &RymdGameModel, resources: &Resources, lockstep: &mut LockstepClient) {

        let should_cancel = is_mouse_button_released(MouseButton::Right) || is_mouse_button_released(MouseButton::Middle);
        let should_build = is_mouse_button_released(MouseButton::Left);
        let mouse_world_position: Vec2 = mouse_position().into();

        if let Some(blueprint_id) = self.current_blueprint_id {

            let blueprint = model.blueprint_manager.get_blueprint(blueprint_id).expect("could not find the blueprint in the manager somehow, should be impossible!");
            let blueprint_preview_texture = resources.get_texture_by_name(&blueprint.texture);
            let blueprint_preview_position = mouse_world_position;

            draw_texture_centered(
                blueprint_preview_texture,
                blueprint_preview_position.x,
                blueprint_preview_position.y,
                WHITE.with_alpha(0.5)
            );

            if should_build {
                self.finalize_blueprint(model, lockstep);
            }

            if should_cancel {
                self.cancel_blueprint()
            }
            
        }

    }
}

struct OrderingState {
    points: Vec<Vec2>
}

impl OrderingState {

    fn new() -> OrderingState {
        OrderingState { points: Vec::new() }
    }

    fn add_point(&mut self, point: Vec2) {

        let point_add_threshold = 16.0;

        if self.points.len() > 0 {
            if point.distance(self.points[self.points.len() - 1]) >= point_add_threshold {
                self.points.push(point);
            }
        } else {
            self.points.push(point);
        }

    }

    fn get_point(&self, count: usize, idx: usize) -> Vec2 {
        if count > self.points.len() {
            let partition = count / self.points.len();
            self.points[(partition * idx) % self.points.len()]
        } else {
            let partition = self.points.len() / count;
            self.points[(partition * idx) % self.points.len()]
        }
    }
    
    fn clear_points(&mut self) {
        self.points.clear();
    }

}

struct SelectionState {
    is_active: bool,
    start: Vec2,
    end: Vec2
}

impl SelectionState {

    fn new() -> SelectionState {
        SelectionState {
            is_active: false,
            start: Vec2::ZERO,
            end: Vec2::ZERO
        }
    }

    fn as_bounds(&self) -> (Vec2, Vec2) {

        let min_x = self.start.x.min(self.end.x);
        let max_x = self.start.x.max(self.end.x);
        let min_y = self.start.y.min(self.end.y);
        let max_y = self.start.y.max(self.end.y);

        (vec2(min_x, min_y), vec2(max_x, max_y))

    }

    fn as_rect(&self) -> Rect {

        let (min, max) = self.as_bounds();

        Rect {
            x: min.x,
            y: min.y,
            w: max.x - min.x,
            h: max.y - min.y
        }

    }

}

#[derive(Debug)]
struct Selectable {
    is_selected: bool
}

#[derive(Debug)]
struct Bounds {
    rect: Rect
}

impl Bounds {
    fn as_radius(&self) -> f32 {
        self.rect.size().max_element() / 2.0 // #TODO: this is hella magic number tho
    }
}

struct Particles {
    emitter: Emitter
}

struct Resources {
    placeholder_texture: Texture2D,
    particle_emitters: HashMap<String, Emitter>,
    particle_emitter_configs: HashMap<String, EmitterConfig>,
    textures: HashMap<String, Texture2D>
}

impl Resources {

    fn create_placeholder_texture() -> Texture2D {
        let placeholder_size = 32;
        let placeholder_image = Image::gen_image_color(placeholder_size, placeholder_size, WHITE);
        let placeholder_texture = Texture2D::from_image(&placeholder_image);
        set_texture_filter(placeholder_texture, FilterMode::Nearest);
        placeholder_texture
    }

    fn new() -> Resources {
        Resources {
            placeholder_texture: Self::create_placeholder_texture(),
            particle_emitters: HashMap::new(),
            particle_emitter_configs: HashMap::new(),
            textures: HashMap::new()
        }
    }

    fn get_emitter_config_by_name(&self, name: &str) -> EmitterConfig {
        if let Some(emitter) = self.particle_emitter_configs.get(name) {
            emitter.clone()
        } else {
            panic!("no such emitter config: {}, exiting!", name);
        }
    }

    fn get_emitter_by_name(&mut self, name: &str) -> &mut Emitter {
        if let Some(emitter) = self.particle_emitters.get_mut(name) {
            emitter
        } else {
            panic!("no such emitter: {}, exiting!", name);
        }
    }

    fn get_texture_by_name(&self, name: &str) -> Texture2D {
        if let Some(texture) = self.textures.get(name) {
            *texture
        } else {
            self.placeholder_texture
        }
    }

    async fn load_texture_or_placeholder(&mut self, name: &str, path: &str, filter: FilterMode) {
        let texture = load_texture(path).await.unwrap_or(self.placeholder_texture);
        if texture != self.placeholder_texture {
            set_texture_filter(texture, filter);
        }
        self.register_texture(name, texture);
    }

    fn register_texture(&mut self, name: &str, texture: Texture2D) {
        self.textures.insert(name.to_string(), texture);
    }

    fn register_emitter(&mut self, name: &str, emitter: Emitter) {
        self.particle_emitters.insert(name.to_string(), emitter);
    }

    fn register_emitter_config(&mut self, name: &str, emitter_config: EmitterConfig) {
        self.particle_emitter_configs.insert(name.to_string(), emitter_config);
    }

    fn create_particle_emitter_configurations(&mut self) {

        fn engine() -> EmitterConfig {
            EmitterConfig {
                lifetime: 0.4,
                lifetime_randomness: 0.1,
                amount: 10,
                emitting: false,
                initial_direction_spread: 0.5,
                initial_velocity_randomness: 0.75,
                initial_velocity: 0.0,
                size: 1.0,
                blend_mode: BlendMode::Additive,
                ..Default::default()
            }
        }
    
        fn small_engine() -> EmitterConfig {
            EmitterConfig {
                lifetime: 0.4*2.0,
                lifetime_randomness: 0.1,
                amount: 10,
                emitting: false,
                initial_direction_spread: 0.25,
                initial_velocity: 0.0,
                size: 0.5,
                blend_mode: BlendMode::Additive,
                ..Default::default()
            }
        }
    
        fn thruster() -> EmitterConfig {
            EmitterConfig {
                lifetime: 0.4,
                lifetime_randomness: 0.1,
                amount: 10,
                emitting: false,
                initial_direction_spread: 0.1,
                initial_velocity: 0.0,
                size: 0.75,
                blend_mode: BlendMode::Additive,
                ..Default::default()
            }
        }
     
        fn smoke() -> EmitterConfig {
            EmitterConfig {
                lifetime: 1.0,
                lifetime_randomness: 0.5,
                amount: 10,
                emitting: false,
                initial_direction_spread: 0.75,
                initial_velocity: 0.0,
                size: 6.0,
                blend_mode: BlendMode::Additive,
                ..Default::default()
            }
        }
    
        fn impact() -> EmitterConfig {
            EmitterConfig {
                lifetime: 1.0,
                lifetime_randomness: 0.1,
                amount: 8,
                emitting: false,
                initial_direction_spread: 0.5,
                initial_velocity: 0.0,
                size: 1.0,
                blend_mode: BlendMode::Additive,
                ..Default::default()
            }
        }
    
        fn repair() -> EmitterConfig {
            EmitterConfig {
                lifetime: 1.0,
                lifetime_randomness: 0.1,
                amount: 1,
                emitting: false,
                initial_direction_spread: 0.1,
                initial_velocity: 0.0,
                size: 1.0,
                blend_mode: BlendMode::Additive,
                ..Default::default()
            }
        }

        let ship_engine_emitter_config = EmitterConfig {
            local_coords: false,
            texture: Some(self.get_texture_by_name("EXHAUST")),
            ..engine()
        };

        let ship_engine_emitter = Emitter::new(ship_engine_emitter_config.clone());

        self.register_emitter("STANDARD", ship_engine_emitter);
        self.register_emitter_config("STANDARD", ship_engine_emitter_config);
    
        let ship_turn_emitter_config = EmitterConfig {
            local_coords: true,
            texture: Some(self.get_texture_by_name("EXHAUST_SMALL")),
            ..thruster()
        };

        let ship_turn_emitter = Emitter::new(ship_turn_emitter_config.clone());

        self.register_emitter("STANDARD_TURN", ship_turn_emitter);
        self.register_emitter_config("STANDARD_TURN", ship_turn_emitter_config);

    }

    async fn load(&mut self) {

        let placeholder_size = 32;
        let placeholder_image = Image::gen_image_color(placeholder_size, placeholder_size, WHITE);
        let placeholder_texture = Texture2D::from_image(&placeholder_image);

        self.load_texture_or_placeholder("BG_TEXTURE", "raw/space.png", FilterMode::Nearest).await;
        self.load_texture_or_placeholder("PLAYER_SHIP", "raw/player_ship.png", FilterMode::Nearest).await;
        self.load_texture_or_placeholder("SIMPLE_BULLET", "raw/simple_bullet.png", FilterMode::Nearest).await;
        self.load_texture_or_placeholder("SMALL_SIMPLE_BULLET", "raw/small_simple_bullet.png", FilterMode::Nearest).await;
        self.load_texture_or_placeholder("MUZZLE_FLASH", "raw/muzzle_flash.png", FilterMode::Nearest).await;
        self.load_texture_or_placeholder("EXPLOSION", "raw/explosion_1_small.png", FilterMode::Nearest).await;
        self.load_texture_or_placeholder("ENEMY_GRUNT", "raw/enemy_grunt.png", FilterMode::Nearest).await;
        self.load_texture_or_placeholder("ENEMY_MEDIUM_GRUNT", "raw/enemy_medium_grunt.png", FilterMode::Nearest).await;
        self.load_texture_or_placeholder("ENEMY_GRUNT_REPAIR", "raw/enemy_grunt_repair.png", FilterMode::Nearest).await;
        self.load_texture_or_placeholder("HAMMERHEAD", "raw/hammerhead.png", FilterMode::Nearest).await;
        self.load_texture_or_placeholder("POWER_STATION", "raw/power_station.png", FilterMode::Nearest).await;
        self.load_texture_or_placeholder("POWER_STATION_TURRET", "raw/power_station_turret.png", FilterMode::Nearest).await;

        self.load_texture_or_placeholder("EXHAUST", "raw/exhaust.png", FilterMode::Linear).await;
        self.load_texture_or_placeholder("EXHAUST_SMALL", "raw/exhaust_small.png", FilterMode::Linear).await;

        // now create/Load our particle emitters as well
        self.create_particle_emitter_configurations();
        
    }

}

pub struct RymdGameView {
    player_id: PlayerID,
    construction: ConstructionState,
    selection: SelectionState,
    ordering: OrderingState,
    resources: Resources,
}

impl RymdGameView {

    pub fn new() -> RymdGameView {
        RymdGameView {
            player_id: 0,
            construction: ConstructionState::new(),
            ordering: OrderingState::new(),
            selection: SelectionState::new(),
            resources: Resources::new()
        }
    }

    pub fn start(&mut self, player_id: PlayerID) {
        let _ = std::mem::replace(&mut self.construction, ConstructionState::new());
        self.player_id = player_id
    }

    pub async fn load_resources(&mut self) {
        self.resources.load().await;
    }

    pub fn unload_resources(&mut self) {

    }
    
    fn handle_selection(&mut self, world: &mut World) {

        if self.construction.is_previewing() {
            return;
        }

        // CTRL+A should select all units
        let is_selecting_all = is_key_down(KeyCode::LeftControl) && is_key_down(KeyCode::A);
        if is_selecting_all {
            self.perform_select_all(world);
            return;
        }

        let mouse_position: Vec2 = mouse_position().into();
        let is_adding_to_selection: bool = is_key_down(KeyCode::LeftShift);
        let is_removing_from_selection = is_key_down(KeyCode::LeftControl);
        let mut selection_turned_inactive = false;

        if is_mouse_button_pressed(MouseButton::Left) {
            self.selection.start = mouse_position;
            self.selection.end = mouse_position;
            self.selection.is_active = true;
        }

        if is_mouse_button_released(MouseButton::Left) {
            if self.selection.is_active {
                selection_turned_inactive = true;
            }
            self.selection.is_active = false;
        }

        if self.selection.is_active {
            self.selection.end = mouse_position;
        }

        if selection_turned_inactive {
            self.perform_selection_with_bounds(world, is_adding_to_selection, is_removing_from_selection);
        }

    }

    fn get_entity_under_cursor(&self, world: &World) -> Option<Entity> {

        let mut closest_entity = None;
        let mut closest_distance = f32::MAX;
        let current_mouse_world_position: Vec2 = mouse_position().into();

        for (e, (transform, bounds)) in world.query::<(&Transform, &Bounds)>().iter() {

            let current_distance_to_mouse = current_mouse_world_position.distance(transform.world_position);
            let is_position_within_bounds = current_distance_to_mouse < bounds.as_radius();
            
            if is_position_within_bounds && current_distance_to_mouse < closest_distance {
                closest_distance = current_distance_to_mouse;
                closest_entity = Some(e);
            }

        }

        closest_entity
        
    }

    fn is_entity_attackable(&self, entity: Entity, world: &World) -> bool {
        let controller = world.get::<&Controller>(entity).expect("must have controller!");
        controller.id != self.player_id // #TODO: alliances, teams?
    }

    fn is_entity_friendly(&self, entity: Entity, world: &World) -> bool {
        let controller = world.get::<&Controller>(entity).expect("must have controller!");
        controller.id == self.player_id // #TODO: alliances, teams?
    }

    fn is_entity_controllable(&self, entity: Entity, world: &World) -> bool {
        self.is_entity_friendly(entity, world)
    }

    fn handle_order(&mut self, world: &mut World, lockstep: &mut LockstepClient) {

        let current_mouse_position: Vec2 = mouse_position().into();
        let should_cancel_current_orders: bool = is_key_released(KeyCode::S);

        if is_mouse_button_down(MouseButton::Right) {
            self.ordering.add_point(current_mouse_position);
        }

        if is_mouse_button_released(MouseButton::Right) {

            let should_add = is_key_down(KeyCode::LeftShift);
            let should_group = is_key_down(KeyCode::LeftControl);
            let current_selection_end_point = self.ordering.points[0];
            let entity_under_cursor = self.get_entity_under_cursor(world);

            if let Some(target_entity) = entity_under_cursor {

                if self.is_entity_friendly(target_entity, world) {

                    self.handle_repair_order(world, target_entity, lockstep, should_add);

                } else if self.is_entity_attackable(target_entity, world) {

                    self.handle_attack_order(world, target_entity, lockstep, should_add);
                }

            } else {

                self.handle_move_order(world, current_selection_end_point, current_mouse_position, lockstep, should_group, should_add);

            }

        }

        if should_cancel_current_orders {
            self.cancel_current_orders(world, lockstep);
        }

    }

    fn cancel_current_orders(&self, world: &mut World, lockstep: &mut LockstepClient,) {

        for (e, (orderable, selectable)) in world.query::<(&Orderable, &Selectable)>().iter() {

            if selectable.is_selected && self.is_entity_controllable(e, world) {
                lockstep.cancel_current_orders(e);
                println!("[RymdGameView] cancelled orders for: {:?}", e);
            }
    
        }

    }

    fn handle_attack_order(&mut self, world: &mut World, target_entity: Entity, lockstep: &mut LockstepClient, should_add: bool) {

    }

    fn handle_repair_order(&mut self, world: &mut World, target_entity: Entity, lockstep: &mut LockstepClient, should_add: bool) {

        let target_position = get_entity_position(world, target_entity).expect("target must have position!");

        for (e, (transform, orderable, selectable, constructor)) in world.query_mut::<(&Transform, &Orderable, &Selectable, &Constructor)>() {
    
            let is_target_self = target_entity == e;

            if selectable.is_selected && is_target_self == false {
                lockstep.send_repair_order(e, target_position, target_entity, should_add);
                println!("[RymdGameView] ordered: {:?} to repair: {:?}", e, target_entity);
            }
    
        }

    }

    fn handle_move_order(&mut self, world: &mut World, current_selection_end_point: Vec2, current_mouse_world_position: Vec2, lockstep: &mut LockstepClient, should_group: bool, should_add: bool) {
        
        // we need to know the number of selected orderables so that we can distribute units along the line we draw for movement
        let number_of_selected_orderables = world.query_mut::<(&Orderable, &Selectable)>().into_iter().filter(|e| e.1.1.is_selected).count();

        // order the selectables by their distance from the current selection end point, this way we mostly retain the current arrangement the units are in and they hopefully make sorta-optimal moves
        let mut selectables_ordered_by_distance_to_end_point: Vec<(Entity, (&Transform, &Orderable, &Selectable))> = world.query_mut::<(&Transform, &Orderable, &Selectable)>().into_iter().collect();
        selectables_ordered_by_distance_to_end_point.sort_by(|a, b| a.1.0.world_position.distance(current_selection_end_point).total_cmp(&b.1.0.world_position.distance(current_selection_end_point)));

        // calculate the centroid so that we can use it to figure out where units should go when moving as a group
        let centroid_of_selected_orderables = selectables_ordered_by_distance_to_end_point.iter().fold(Vec2::ZERO, |acc, v| acc + v.1.0.world_position) / selectables_ordered_by_distance_to_end_point.len() as f32;

        for (idx, (e, (transform, orderable, selectable))) in selectables_ordered_by_distance_to_end_point.into_iter().enumerate() {
    
            if selectable.is_selected {

                let current_order_point = if should_group {
                    let offset_from_centroid = centroid_of_selected_orderables - transform.world_position;
                    current_mouse_world_position - offset_from_centroid
                }
                else
                {
                    if number_of_selected_orderables > 1 {
                        self.ordering.get_point(number_of_selected_orderables, idx)
                    } else {
                        current_mouse_world_position
                    }
                };

                lockstep.send_move_order(e, current_order_point, should_add);
                println!("[RymdGameView] ordered: {:?} to move to: {}", e, current_mouse_world_position);

            }
    
        }
            
        self.ordering.clear_points();

    }

    fn can_select_unit(&self, controller: &Controller) -> bool {
        controller.id == self.player_id
    }

    fn perform_select_all(&mut self, world: &mut World) {

        for (e, (transform, orderable, selectable, controller)) in world.query_mut::<(&Transform, &Orderable, &mut Selectable, &Controller)>() {

            if self.can_select_unit(controller) == false {
                continue;
            }
            
            selectable.is_selected = true;
            println!("[RymdGameView] selected: {:?}", e);

        }

    }

    fn perform_selection_with_bounds(&mut self, world: &mut World, is_additive: bool, is_removing: bool) {

        let selection_rectangle = self.selection.as_rect();

        println!("[RymdGameView] attempted to select entities inside: {:?}", selection_rectangle);

        for (e, (transform, orderable, controller, bounds, selectable)) in world.query_mut::<(&Transform, &Orderable, &Controller, Option<&Bounds>, &mut Selectable)>() {

            if self.can_select_unit(controller) == false {
                continue;
            }

            let intersected_with_selection = if let Some(bounds) = bounds {
                let current_selectable_bounds = bounds.rect.offset(transform.world_position);
                current_selectable_bounds.intersect(selection_rectangle).is_some()
            } else {
                is_point_inside_rect(&transform.world_position, &selection_rectangle)
            };

            if is_removing && selectable.is_selected {
                selectable.is_selected = !(selectable.is_selected && intersected_with_selection);
            } else {
                selectable.is_selected = (selectable.is_selected && is_additive) || intersected_with_selection;
            }

            if selectable.is_selected {
                println!("[RymdGameView] selected: {:?}", e);
            }

        }

    }

    fn draw_ordering(&self) {
        
        if self.ordering.points.is_empty() {
            return;
        }

        let line_thickness = 1.0;
        let mut last_point: Vec2 = self.ordering.points[0];

        for p in self.ordering.points.iter().skip(1) {
            if last_point != *p {
                draw_line(
                    last_point.x,
                    last_point.y,
                    p.x,
                    p.y,
                    line_thickness,
                    GREEN
                );
            }
            last_point = *p;
        }

    }

    fn draw_selection(&self) {
        if self.selection.is_active {
            let selection_box = self.selection.as_rect();
            let thickness = 2.0;
            draw_rectangle_lines(
                selection_box.x,
                selection_box.y,
                selection_box.w,
                selection_box.h,
                thickness,
                GREEN
            );
        }
    }

    fn draw_orders(&self, model: &RymdGameModel) {

        for (e, (transform, orderable, selectable)) in model.world.query::<(&Transform, &Orderable, &Selectable)>().iter() {

            if selectable.is_selected == false {
                continue;
            }

            let mut current_line_start = transform.world_position;

            let order_line_thickness = 1.0;
            let order_line_head_size = 8.0;

            for (i, order) in orderable.orders.iter().enumerate() {
                if let Some(target_position) = order.get_target_position(model) {
                    if i == orderable.orders.len() - 1 {
                        draw_arrow(current_line_start.x, current_line_start.y, target_position.x, target_position.y, order_line_thickness, order_line_head_size, GREEN);
                    } else {
                        draw_line(current_line_start.x, current_line_start.y, target_position.x, target_position.y, order_line_thickness, GREEN);
                    }
                    current_line_start = target_position;
                }
            }

        }
        
    }

    pub fn update(&mut self, model: &mut RymdGameModel) {

        let mut selectable_components_to_add = Vec::new();
        let mut thruster_components_to_add = Vec::new();
        let mut bounds_components_to_add = Vec::new();

        for (e, (transform, orderable)) in model.world.query::<Without<(&Transform, &Orderable), &Selectable>>().iter() {
            let selectable = Selectable { is_selected: false };
            selectable_components_to_add.push((e, selectable));
        }

        for (e, (transform, thruster)) in model.world.query::<Without<(&Transform, &Thruster), &Particles>>().iter() {
            let emitter_config_name = if thruster.kind == ThrusterKind::Main { "STANDARD" } else { "STANDARD_TURN" };
            let particle_emitter = Particles { emitter: Emitter::new(self.resources.get_emitter_config_by_name(emitter_config_name)) };
            thruster_components_to_add.push((e, particle_emitter));
        }

        for (e, (transform, selectable, sprite, animated_sprite)) in model.world.query::<(&Transform, &Selectable, Option<&Sprite>, Option<&AnimatedSprite>)>().iter() {

            if let Some(sprite) = sprite {
                let sprite_v_frames = 1;
                let sprite_h_frames = 1;
                let sprite_is_centered = true;
                let sprite_texture_handle = self.resources.get_texture_by_name(&sprite.texture);
                let sprite_texture_bounds = Bounds { rect: calculate_sprite_bounds(sprite_texture_handle, sprite_h_frames, sprite_v_frames, sprite_is_centered) };
                bounds_components_to_add.push((e, sprite_texture_bounds));
            }

            if let Some(animated_sprite) = animated_sprite {
                let sprite_v_frames = 1;
                let sprite_is_centered = true;
                let sprite_texture_handle = self.resources.get_texture_by_name(&animated_sprite.texture);
                let sprite_texture_bounds = Bounds { rect: calculate_sprite_bounds(sprite_texture_handle, animated_sprite.h_frames, sprite_v_frames, sprite_is_centered) };
                bounds_components_to_add.push((e, sprite_texture_bounds));
            }

        }

        for (e, c) in selectable_components_to_add {
            let _ = model.world.insert_one(e, c);
        }

        for (e, c) in thruster_components_to_add {
            let _ = model.world.insert_one(e, c);
        }

        for (e, c) in bounds_components_to_add {
            let _ = model.world.insert_one(e, c);
        }

    }

    pub fn tick(&mut self, model: &mut RymdGameModel, lockstep: &mut LockstepClient) {

        if yakui_macroquad::has_input_focus() { return; }

        self.handle_selection(&mut model.world);
        self.handle_order(&mut model.world, lockstep);

    }

    fn draw_sprites(&self, world: &World) {
        for (e, (transform, sprite, building)) in world.query::<(&Transform, &Sprite, Option<&Building>)>().iter() {
            let sprite_texture_alpha = building_state_to_alpha(building);
            let sprite_texture_handle = self.resources.get_texture_by_name(&sprite.texture);
            draw_texture_centered_with_rotation(sprite_texture_handle, transform.world_position.x, transform.world_position.y, WHITE.with_alpha(sprite_texture_alpha), transform.world_rotation);
        }
    }

    fn draw_animated_sprites(&self, world: &World) {
        for (e, (transform, sprite, building)) in world.query::<(&Transform, &AnimatedSprite, Option<&Building>)>().iter() {
            let is_sprite_flipped = false;
            let sprite_texture_alpha = building_state_to_alpha(building);
            let sprite_texture_handle = self.resources.get_texture_by_name(&sprite.texture);
            draw_texture_centered_with_rotation_frame(sprite_texture_handle, transform.world_position.x, transform.world_position.y, WHITE.with_alpha(sprite_texture_alpha), transform.world_rotation, sprite.current_frame, sprite.h_frames, is_sprite_flipped);
        }
    }

    fn draw_selectables(&self, world: &mut World) {

        let bounds_thickness = 1.0;

        for (e, (transform, selectable, bounds)) in world.query::<(&Transform, &Selectable, &Bounds)>().iter() {
            if selectable.is_selected {
                draw_circle_lines(
                    transform.world_position.x,
                    transform.world_position.y,
                    bounds.as_radius() * 1.5,
                    bounds_thickness,
                    GREEN
                );
            }
        }

    }

    fn update_thrusters(&mut self, world: &World) {

        for (e, (transform, body, ship)) in world.query::<(&Transform, &DynamicBody, &Ship)>().iter() {
            for &t in &ship.thrusters {

                if let Ok(mut query) = world.query_one::<(&Transform, &Thruster, &mut Particles)>(t) && let Some((thruster_transform, thruster, particles)) = query.get() {

                    let thruster_direction = thruster.direction.rotated_by(transform.world_rotation);
                    let thruster_emit_direction = thruster.direction.rotated_by(transform.world_rotation + thruster.angle);

                    if thruster.kind == ThrusterKind::Attitude {

                        let thruster_alignment = body.kinematic.angular_velocity.as_vector().normalize().dot(thruster_direction);
                        if thruster_alignment > 0.0 && body.kinematic.angular_velocity.abs() > 0.2 {
                            particles.emitter.config.initial_direction = thruster_emit_direction;
                            particles.emitter.config.initial_velocity = thruster.power * 4.0;
                            particles.emitter.config.lifetime = 0.25; // ARBITRARY NUMBERS WOO
                            particles.emitter.emit(vec2(0.0, 0.0), ((thruster.power * thruster_alignment) / 4.0) as usize);
                        }

                    } else {

                        let ship_velocity = body.kinematic.velocity.length();
                        let ship_alignment = body.kinematic.velocity.dot(thruster_direction);

                        if ship_velocity > 0.0 && ship_alignment < 0.0 {
                            particles.emitter.config.initial_direction = thruster_emit_direction;
                            particles.emitter.config.initial_velocity = thruster.power * 4.0;
                            particles.emitter.config.lifetime = 1.0; // ARBITRARY NUMBERS WOO
                            particles.emitter.emit(vec2(0.0, 0.0), (ship_alignment.abs() / thruster.power * 2.0) as usize);
                        }
                        
                    }

                } else {
                    continue;
                };

            }
        }

    }

    fn draw_particles(&mut self, world: &mut World) {

        for (_, emitter) in &mut self.resources.particle_emitters {
            emitter.draw(Vec2::ZERO);
        }

        for (e, (transform, particles)) in world.query_mut::<(&Transform, &mut Particles)>() {
            particles.emitter.draw(transform.world_position);
        }

    }

    fn draw_debug_ui(&self, model: &RymdGameModel, debug: &mut DebugText) {
        
        debug.draw_text(format!("mouse position: {:?}", mouse_position()), TextPosition::TopLeft, WHITE);
        debug.draw_text(format!("number of entities: {}", model.world.len()), TextPosition::TopLeft, WHITE);
        debug.draw_text(format!("collision responses: {}", model.physics_manager.number_of_active_collision_responses()), TextPosition::TopLeft, WHITE);

    }

    fn draw_background_texture(&self, w: f32, h: f32, position: Vec2) {

        let bg_texture = self.resources.get_texture_by_name("BG_TEXTURE");

        set_default_camera();
    
        let parallax_scale = 0.1;
        let scaled_position = position * parallax_scale;
    
        let bg_w = bg_texture.width();
        let bg_h = bg_texture.height();
    
        let offset_x = scaled_position.x % bg_w;
        let offset_y = scaled_position.y % bg_h;
    
        let num_images_w = (w / bg_w).ceil() as i32 + 2;
        let num_images_h = (h / bg_h).ceil() as i32 + 2;
    
        for x in 0..num_images_w {
            for y in 0..num_images_h {
                draw_texture(bg_texture, (x - 1) as f32 * bg_w - offset_x, (y - 1) as f32 * bg_h - offset_y, WHITE);
            }
        }
    
    }

    fn current_selection_has_constructor_unit(&self, world: &World) -> bool {

        for (e, (selectable, constructor)) in world.query::<(&Selectable, &Constructor)>().iter() {
            if selectable.is_selected {
                return true;
            }
        }

        false
        
    }

    fn get_available_blueprints_from_current_selection(&self, world: &World) -> Vec<BlueprintID> {

        let mut blueprints = Vec::new();

        for (e, (selectable, constructor)) in world.query::<(&Selectable, &Constructor)>().iter() {
            if selectable.is_selected {
                for id in &constructor.constructibles {
                    if blueprints.contains(id) == false {
                        blueprints.push(*id);
                    }
                }
            }
        }

        blueprints

    }

    fn order_build_blueprint(&mut self, entity: Entity, world: &mut World) {

    }

    fn draw_construction_ui(&mut self, model: &mut RymdGameModel, lockstep: &mut LockstepClient) {

        let should_add_to_queue = is_key_down(KeyCode::LeftShift);
        let available_blueprints = self.get_available_blueprints_from_current_selection(&model.world);

        let mut selected_constructors_query = model.world.query::<(&Transform, &Orderable, &Selectable, &Constructor)>();
        let selected_constructor_units: Vec<(Entity, (&Transform, &Orderable, &Selectable, &Constructor))> = selected_constructors_query.into_iter().filter(|(q, (t, o, s, c))| s.is_selected).collect();
        let current_build_position: Vec2 = mouse_position().into();

        yakui::align(Alignment::CENTER_LEFT, || {
            yakui::colored_box_container(yakui::Color::GRAY, || {
                yakui::pad(yakui::widgets::Pad::all(4.0), || {

                    yakui_min_column(|| {
                        for id in available_blueprints.into_iter() {
                            let blueprint = model.blueprint_manager.get_blueprint(id).expect("could not find blueprint? this is a bug!");
                            if yakui::button(blueprint.name.to_string()).clicked {
                                
                                for (e, (t, o, s, c)) in &selected_constructor_units {
                                    lockstep.send_build_order(*e, current_build_position, blueprint.id, should_add_to_queue);
                                    println!("[RymdGameView] attempted to send build order for position: {} and blueprint: {}", current_build_position, blueprint.id);
                                }

                            }
                        }
                    });

                });
            });
        });

    }

    fn draw_text_construction_ui(&mut self, model: &mut RymdGameModel, debug: &mut DebugText) {

        let available_blueprints = self.get_available_blueprints_from_current_selection(&model.world);

        debug.skip_line(TextPosition::BottomRight);

        for id in available_blueprints {
            let blueprint = model.blueprint_manager.get_blueprint(id).unwrap();
            debug.draw_text(format!(" > {} ({:?})", blueprint.name, blueprint.shortcut), TextPosition::BottomRight, WHITE);
            if is_key_released(blueprint.shortcut) {
                self.construction.preview_blueprint(id);
            }
        }

        debug.draw_text("constructibles", TextPosition::BottomRight, WHITE);

    }

    fn draw_ui(&mut self, model: &mut RymdGameModel, debug: &mut DebugText, lockstep: &mut LockstepClient) {

        yakui::align(Alignment::TOP_CENTER, || {

            let current_metal = 100;
            let current_metal_income = 0;
            let current_metal_excess = 0;

            let current_energy = 100;
            let current_energy_income = 0;
            let current_energy_excess = 0;

            yakui::label(format!("current metal: {}, current energy: {}", current_metal, current_energy));

        });

        if self.current_selection_has_constructor_unit(&model.world) {
            self.draw_text_construction_ui(model, debug);
            // self.draw_construction_ui(model, lockstep);
        }

    }

    fn draw_health_labels(&self, world: &World) {

        for (e, (transform, health)) in world.query::<(&Transform, &Health)>().iter() {
            let health_label_position = transform.world_position + vec2(0.0, -32.0);
            draw_text_centered(&format!("{}/{}", health.current_health, health.full_health), health_label_position.x, health_label_position.y, 24.0, WHITE);
        }

    }

    pub fn draw(&mut self, model: &mut RymdGameModel, debug: &mut DebugText, lockstep: &mut LockstepClient) {

        self.update_thrusters(&model.world);

        let screen_center: Vec2 = vec2(screen_width(), screen_height()) / 2.0;
        self.draw_background_texture(screen_width(), screen_height(), screen_center);

        self.draw_orders(&model);
        self.draw_selection();
        self.draw_ordering();

        self.draw_sprites(&model.world);
        self.draw_animated_sprites(&model.world);
        self.draw_selectables(&mut model.world);

        self.construction.tick_and_draw(model, &self.resources, lockstep);

        self.draw_particles(&mut model.world);
        self.draw_health_labels(&model.world);
        self.draw_ui(model, debug, lockstep);
        self.draw_debug_ui(model, debug);

    }

}

impl Drop for RymdGameView {
    fn drop(&mut self) {
        self.unload_resources();
    }
}