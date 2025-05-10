use std::f32::consts::PI;

use fnv::FnvHashMap;
use lockstep_client::game::GameContext;
use macroquad_particles::{EmitterConfig, Emitter};
use puffin_egui::egui::{self, Align2};
use utility::{draw_arrow, draw_rectangle_lines_centered_with_rotation, draw_text_centered, draw_texture_centered, draw_texture_centered_with_rotation, draw_texture_centered_with_rotation_frame, is_point_inside_rect, AsPerpendicular, AsVector, AverageLine2D, DebugText, RotatedBy, TextPosition, WithAlpha};
use lockstep_client::step::LockstepClient;
use macroquad_particles::*;
use macroquad::{prelude::*, text};
use hecs::*;

use crate::PlayerID;
use crate::game::RymdGameParameters;
use crate::model::{current_energy, current_energy_income, current_metal, current_metal_income, existing_static_body_within_bounds, max_energy, max_metal, Attacker, Beam, Blueprint, BlueprintID, BlueprintIdentity, Blueprints, Building, Commander, Effect, EntityState, Extractor, GameOrder, GameOrderType, Impact, PhysicsBody, ResourceSource, Spawner};
use crate::model::{RymdGameModel, Orderable, Transform, Sprite, AnimatedSprite, GameOrdersExt, DynamicBody, Thruster, Ship, ThrusterKind, Constructor, Controller, Health, get_entity_position};

use super::{calculate_sprite_bounds, GameCamera2D};

fn entity_state_to_alpha(state: Option<&EntityState>) -> f32 {
    if let Some(state) = state {
        match state {
            crate::model::EntityState::Ghost => 0.75,
            crate::model::EntityState::Destroyed => 1.0,
            crate::model::EntityState::Constructed => 1.0,
            crate::model::EntityState::Inactive => 1.0
        }
    } else {
        1.0
    }
}

fn get_blueprint_bounds(resources: &Resources, blueprint: &Blueprint) -> Rect {
    let texture = resources.get_texture_by_name(&blueprint.name);
    Rect {
        x: -(texture.width() / 2.0),
        y: -(texture.height() / 2.0),
        w: texture.width(),
        h: texture.height()
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
    
    fn get_number_to_build() -> i32 {
        let should_build_five = is_key_down(KeyCode::LeftShift);
        let should_build_twenty = is_key_down(KeyCode::LeftControl);
        let should_build_one_hundred = is_key_down(KeyCode::LeftShift) && is_key_down(KeyCode::LeftControl);
        if should_build_one_hundred {
            100
        } else if should_build_twenty {
            20
        } else if should_build_five {
            5
        } else {
            1
        }
    }

    fn finalize_blueprint(&mut self, model: &RymdGameModel, camera: &GameCamera2D, lockstep: &mut LockstepClient) {
        
        if let Some(blueprint_id) = self.current_blueprint_id {

            let should_add_to_queue = is_key_down(KeyCode::LeftShift);
    
            for (e, (transform, _o, selectable, constructor, spawner)) in model.world.query::<(&Transform, &Orderable, &Selectable, &Constructor, Option<&Spawner>)>().iter() {

                if selectable.is_selected == false || constructor.has_blueprint(blueprint_id) == false {
                    continue;
                }

                if let Some(spawner) = spawner {
                    let is_self_order = true;
                    let current_build_position: Vec2 = transform.world_position + spawner.position;                
                    for i in 0..Self::get_number_to_build() {
                        lockstep.send_build_order(e, current_build_position, blueprint_id, should_add_to_queue, is_self_order);
                        println!("[RymdGameView] attempted to send build order for unit at position: {} and blueprint: {}", current_build_position, blueprint_id);
                    }
                } else {
                    let is_self_order = false;
                    let current_build_position: Vec2 = camera.mouse_world_position();
                    lockstep.send_build_order(e, current_build_position, blueprint_id, should_add_to_queue, is_self_order);
                    println!("[RymdGameView] attempted to send build order for building at position: {} and blueprint: {}", current_build_position, blueprint_id);
                }

            }

            self.current_blueprint_id = None;

        }

    }

    fn tick_and_draw(&mut self, model: &RymdGameModel, camera: &GameCamera2D, resources: &Resources, lockstep: &mut LockstepClient) {

        if let Some(blueprint_id) = self.current_blueprint_id {

            let blueprint = model.blueprint_manager.get_blueprint(blueprint_id).expect("could not find the blueprint in the manager somehow, should be impossible!");

            if blueprint.is_building {
                self.preview_building(resources, blueprint, model, camera, lockstep);
            } else {
                self.finalize_blueprint(model, camera, lockstep);
            }
            
        }

    }

    fn draw_building(resources: &Resources, blueprint: &Blueprint, position: Vec2, is_blocked: bool) {

        let blueprint_preview_alpha = 0.5;
        let blueprint_preview_texture = resources.get_texture_by_name(&blueprint.texture);
        let blueprint_preview_position = position;

        let blueprint_preview_color = if is_blocked == false { WHITE.with_alpha(blueprint_preview_alpha) } else { RED.with_alpha(blueprint_preview_alpha) };

        draw_texture_centered(
            &blueprint_preview_texture,
            blueprint_preview_position.x,
            blueprint_preview_position.y,
            blueprint_preview_color
        );
        
    }

    fn preview_building(&mut self, resources: &Resources, blueprint: &Blueprint, model: &RymdGameModel, camera: &GameCamera2D, lockstep: &mut LockstepClient) {

        let mouse_world_position: Vec2 = camera.mouse_world_position();
        let blueprint_preview_position = mouse_world_position;

        let is_build_position_blocked = existing_static_body_within_bounds(&model.world, get_blueprint_bounds(resources, blueprint), blueprint_preview_position);

        let wants_to_build = is_mouse_button_released(MouseButton::Left);
        let should_cancel = (is_mouse_button_released(MouseButton::Right) || is_mouse_button_released(MouseButton::Middle)) || (wants_to_build && is_build_position_blocked);
        let should_build = wants_to_build && is_build_position_blocked == false;

        Self::draw_building(resources, blueprint, blueprint_preview_position, is_build_position_blocked);

        // if is_build_position_blocked {
        //     let blueprint_bounds = get_blueprint_bounds(resources, blueprint).offset(blueprint_preview_position);
        //     draw_rectangle_lines(
        //         blueprint_bounds.x,
        //         blueprint_bounds.y,
        //         blueprint_bounds.w,
        //         blueprint_bounds.h,
        //         2.0,
        //         GREEN
        //     );
        // }

        if should_build {
            self.finalize_blueprint(model, camera, lockstep);
        }

        if should_cancel {
            self.cancel_blueprint()
        }

    }

}

struct OrderingState {
    line: AverageLine2D
}

impl OrderingState {

    fn new() -> OrderingState {
        let min_point_distance = 4.0;
        OrderingState {
            line: AverageLine2D::new(min_point_distance)
        }
    }

    fn is_empty(&self) -> bool {
        self.line.is_empty()
    }

    fn points(&self) -> &Vec<Vec2> {
        self.line.points()
    }

    fn add_point(&mut self, point: Vec2) {
        self.line.add_point(point);
    }

    fn get_point(&self, count: usize, idx: usize) -> Vec2 {
        let fraction = idx as f32 / count as f32;
        self.line.get_point(fraction)
    }
    
    fn clear_points(&mut self) {
        self.line.clear_points();
    }

}

struct SelectionState {
    last_click_time: f64,
    is_active: bool,
    start: Vec2,
    end: Vec2
}

impl SelectionState {

    fn new() -> SelectionState {
        SelectionState {
            last_click_time: 0.0,
            is_active: false,
            start: Vec2::ZERO,
            end: Vec2::ZERO
        }
    }

    fn register_click(&mut self) {
        self.last_click_time = get_time();
    }

    fn was_double_click(&self) -> bool {
        let double_click_time = 0.5;
        let current_time = get_time();
        
        (current_time - self.last_click_time) < double_click_time
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

struct ParticleBeam {
    emitter: Emitter,
    offset: Vec2
}

struct Resources {
    placeholder_texture: Texture2D,
    particle_emitters: FnvHashMap<String, Emitter>,
    particle_emitter_configs: FnvHashMap<String, EmitterConfig>,
    textures: FnvHashMap<String, Texture2D>
}

impl Resources {

    fn create_placeholder_texture() -> Texture2D {
        let placeholder_size = 32;
        let placeholder_image = Image::gen_image_color(placeholder_size, placeholder_size, WHITE);
        let placeholder_texture = Texture2D::from_image(&placeholder_image);
        placeholder_texture.set_filter(FilterMode::Nearest);
        placeholder_texture
    }

    fn new() -> Resources {
        Resources {
            placeholder_texture: Self::create_placeholder_texture(),
            particle_emitters: FnvHashMap::default(),
            particle_emitter_configs: FnvHashMap::default(),
            textures: FnvHashMap::default()
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
            texture.clone()
        } else {
            self.placeholder_texture.weak_clone()
        }
    }

    async fn load_texture_or_placeholder(&mut self, name: &str, path: &str, filter: FilterMode) {
        let texture = load_texture(path).await.unwrap_or(self.placeholder_texture.weak_clone());
        if texture != self.placeholder_texture {
            texture.set_filter(filter);
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
                colors_curve: ColorCurve { start: WHITE, mid: WHITE.with_alpha(0.5), end: WHITE.with_alpha(0.0) },
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
                colors_curve: ColorCurve { start: WHITE, mid: WHITE.with_alpha(0.5), end: WHITE.with_alpha(0.0) },
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
                colors_curve: ColorCurve { start: WHITE, mid: WHITE.with_alpha(0.5), end: WHITE.with_alpha(0.0) },
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
                colors_curve: ColorCurve { start: WHITE, mid: WHITE.with_alpha(0.5), end: WHITE.with_alpha(0.25) },
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
                colors_curve: ColorCurve { start: WHITE, mid: WHITE.with_alpha(0.875), end: WHITE.with_alpha(0.75) },
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

        let repair_emitter_config = EmitterConfig {
            local_coords: false,
            texture: Some(self.get_texture_by_name("EXHAUST")),
            ..repair()
        };

        let repair_emitter = Emitter::new(repair_emitter_config.clone());

        self.register_emitter("REPAIR", repair_emitter);
        self.register_emitter_config("REPAIR", repair_emitter_config);

        let impact_emitter_config = EmitterConfig {
            local_coords: false,
            texture: Some(self.get_texture_by_name("EXHAUST")),
            ..impact()
        };

        let impact_emitter = Emitter::new(impact_emitter_config.clone());

        self.register_emitter("IMPACT", impact_emitter);
        self.register_emitter_config("IMPACT", impact_emitter_config);

    }

    async fn load(&mut self) {

        let placeholder_size = 32;
        let placeholder_image = Image::gen_image_color(placeholder_size, placeholder_size, WHITE);
        let placeholder_texture = Texture2D::from_image(&placeholder_image);

        // environment, background, etc
        self.load_texture_or_placeholder("BG_TEXTURE", "raw/space.png", FilterMode::Nearest).await;

        // ship types of various kinds
        self.load_texture_or_placeholder("PLAYER_SHIP", "raw/player_ship.png", FilterMode::Nearest).await;
        self.load_texture_or_placeholder("ENEMY_SHIP", "raw/enemy_spiker.png", FilterMode::Nearest).await;
        self.load_texture_or_placeholder("ENEMY_GRUNT", "raw/enemy_grunt.png", FilterMode::Nearest).await;
        self.load_texture_or_placeholder("ENEMY_MEDIUM_GRUNT", "raw/enemy_medium_grunt.png", FilterMode::Nearest).await;
        self.load_texture_or_placeholder("ENEMY_GRUNT_REPAIR", "raw/enemy_grunt_repair.png", FilterMode::Nearest).await;
        self.load_texture_or_placeholder("HAMMERHEAD", "raw/hammerhead.png", FilterMode::Nearest).await;
        self.load_texture_or_placeholder("ARROWHEAD", "raw/arrowhead.png", FilterMode::Nearest).await;
        self.load_texture_or_placeholder("EXTRACTOR", "raw/extractor.png", FilterMode::Nearest).await;
        
        // buildings of various kinds
        self.load_texture_or_placeholder("SHIPYARD", "raw/shipyard.png", FilterMode::Nearest).await;
        self.load_texture_or_placeholder("POWER_STATION", "raw/power_station.png", FilterMode::Nearest).await;
        self.load_texture_or_placeholder("POWER_STATION_TURRET", "raw/power_station_turret.png", FilterMode::Nearest).await;
        self.load_texture_or_placeholder("SOLAR_COLLECTOR", "raw/solar_collector_light.png", FilterMode::Nearest).await;
        self.load_texture_or_placeholder("ENERGY_STORAGE", "raw/energy_storage_small.png", FilterMode::Nearest).await;
        self.load_texture_or_placeholder("METAL_STORAGE", "raw/metal_storage_small.png", FilterMode::Nearest).await;

        // bullets
        self.load_texture_or_placeholder("SIMPLE_BULLET", "raw/simple_bullet.png", FilterMode::Nearest).await;
        self.load_texture_or_placeholder("SMALL_SIMPLE_BULLET", "raw/small_simple_bullet.png", FilterMode::Nearest).await;

        // one-off effects (explosions, etc)
        self.load_texture_or_placeholder("MUZZLE_FLASH", "raw/muzzle_flash.png", FilterMode::Nearest).await;
        self.load_texture_or_placeholder("EXPLOSION", "raw/explosion_1_small.png", FilterMode::Nearest).await;

        // particle effect textures
        self.load_texture_or_placeholder("EXHAUST", "raw/exhaust.png", FilterMode::Linear).await;
        self.load_texture_or_placeholder("EXHAUST_SMALL", "raw/exhaust_small.png", FilterMode::Linear).await;

        // environmental props?
        self.load_texture_or_placeholder("ASTEROID", "raw/asteroid.png", FilterMode::Nearest).await;

        // now create/load our particle emitters as well
        self.create_particle_emitter_configurations();
        
    }

}

#[derive(Debug)]
struct ControlGroup {
    id: i32,
    entities: Vec<Entity>
}

#[derive(Debug)]
struct ControlGroupState {
    groups: Vec<ControlGroup>
}

impl ControlGroupState {

    pub fn new() -> ControlGroupState {
        ControlGroupState {
            groups: Vec::new()
        }
    }

    pub fn set(&mut self, id: i32, entities: &[Entity]) {

        self.groups.retain(|g| g.id != id);

        let control_group = ControlGroup { id, entities: Vec::from(entities) };
        self.groups.push(control_group);

    }

    pub fn get(&mut self, id: i32) -> &[Entity] {

        if let Some(control_group) = self.groups.iter().find(|g| g.id == id) {
            return &control_group.entities;
        }

        &[]
        
    }

}

pub struct RymdGameView {

    game_player_id: PlayerID,
    game_parameters: RymdGameParameters,

    camera: GameCamera2D,
    construction: ConstructionState,
    control_groups: ControlGroupState,
    selection: SelectionState,
    ordering: OrderingState,
    resources: Resources,
    
    debug: RymdGameDebug

}

struct RymdGameDebug {
    render_bounds: bool,
    render_kinematic: bool,
    render_spatial: bool
}

impl RymdGameDebug {
    pub fn new() -> RymdGameDebug {
        RymdGameDebug {
            render_bounds: false,
            render_kinematic: false,
            render_spatial: false
        }
    }
}

impl RymdGameView {

    pub fn new() -> RymdGameView {
        RymdGameView {
            game_player_id: 0,
            game_parameters: RymdGameParameters::new(),
            camera: GameCamera2D::new(),
            construction: ConstructionState::new(),
            control_groups: ControlGroupState::new(),
            ordering: OrderingState::new(),
            selection: SelectionState::new(),
            resources: Resources::new(),
            debug: RymdGameDebug::new()
        }
    }

    fn switch_player_id_to_next(&mut self, world: &mut World) {

        self.perform_unselect_all(world);
        
        let mut found_current_player_id = false;
        let mut next_player_id = None;
        let mut last_player_id = None;

        for player in &self.game_parameters.players {

            if player.id != self.game_player_id && last_player_id.is_none() {
                last_player_id = Some(player.id);
            }

            if player.id == self.game_player_id {
                found_current_player_id = true;
            }

            if player.id != self.game_player_id && found_current_player_id {
                next_player_id = Some(player.id);
            }

        }

        if let Some(next_player_id) = next_player_id {
            self.game_player_id = next_player_id;
        } else if let Some(last_player_id) = last_player_id {
            self.game_player_id = last_player_id;
        } else {
            panic!("the game should always have at least two players, or the game state is not valid!");
        }

    }

    pub fn start(&mut self, game_parameters: RymdGameParameters, game_player_id: PlayerID) {
        self.construction = ConstructionState::new();
        self.camera = GameCamera2D::new();
        self.game_player_id = game_player_id;
        self.game_parameters = game_parameters;
    }

    pub async fn load_resources(&mut self) {
        self.resources.load().await;
    }

    pub fn unload_resources(&mut self) {

    }

    fn perform_unselect_all_non_constructor_units(&mut self, world: &mut World) {

        for (entity, (controller, selectable)) in world.query_mut::<Without<(&Controller, &mut Selectable), &Constructor>>() {

            if self.can_select_unit(controller) == false {
                continue;
            }

            selectable.is_selected = false;

        }

    }

    fn perform_unselect_all_units_without_blueprint(&mut self, world: &mut World, blueprint_id: BlueprintID) {

        for (entity, (blueprint_identity, controller, selectable)) in world.query_mut::<(&BlueprintIdentity, &Controller, &mut Selectable)>() {

            if self.can_select_unit(controller) == false {
                continue;
            }

            if blueprint_identity.blueprint_id != blueprint_id {
                selectable.is_selected = false;
            }

        }

    }

    fn perform_select_next_commander(&mut self, world: &mut World) {

        // #FIXME: each faction will probably have unique commander blueprints, this shouldn't be hardcoded really
        self.perform_unselect_all_units_without_blueprint(world, Blueprints::Commander as i32);

        for (entity, (blueprint_identity, controller, constructor, selectable, orderable)) in world.query_mut::<(&BlueprintIdentity, &Controller, &Constructor, &mut Selectable, &Orderable)>() {

            if self.can_select_unit(controller) == false {
                continue;
            }

            if selectable.is_selected {
                selectable.is_selected = false;
                continue;
            }

            if blueprint_identity.blueprint_id == (Blueprints::Commander as i32) {
                selectable.is_selected = true;
                return;
            }

        }

    }

    fn perform_select_next_idle_constructor(&mut self, world: &mut World) {

        self.perform_unselect_all_non_constructor_units(world);

        for (entity, (controller, constructor, selectable, orderable)) in world.query_mut::<(&Controller, &Constructor, &mut Selectable, &Orderable)>() {

            if self.can_select_unit(controller) == false {
                continue;
            }

            if orderable.is_queue_empty(GameOrderType::Construct) == false || orderable.is_queue_empty(GameOrderType::Order) == false {
                continue;
            }

            if selectable.is_selected {
                selectable.is_selected = false;
                continue;
            }

            selectable.is_selected = true;
            return;

        }

    }

    fn is_any_number_key_pressed() -> bool {
        is_key_pressed(KeyCode::Key0)
            || is_key_pressed(KeyCode::Key1)
            || is_key_pressed(KeyCode::Key2)
            || is_key_pressed(KeyCode::Key3)
            || is_key_pressed(KeyCode::Key4)
            || is_key_pressed(KeyCode::Key5)
            || is_key_pressed(KeyCode::Key6)
            || is_key_pressed(KeyCode::Key7)
            || is_key_pressed(KeyCode::Key8)
            || is_key_pressed(KeyCode::Key9)
    }

    fn get_first_number_key_pressed() -> Option<i32> {

        if is_key_pressed(KeyCode::Key0) {
            return Some(0);
        }
        
        if is_key_pressed(KeyCode::Key1) {
            return Some(1);
        }

        if is_key_pressed(KeyCode::Key2) {
            return Some(2);
        }

        if is_key_pressed(KeyCode::Key3) {
            return Some(3);
        }

        if is_key_pressed(KeyCode::Key4) {
            return Some(4);
        }

        if is_key_pressed(KeyCode::Key5) {
            return Some(5);
        }

        if is_key_pressed(KeyCode::Key6) {
            return Some(6);
        }

        if is_key_pressed(KeyCode::Key7) {
            return Some(7);
        }

        if is_key_pressed(KeyCode::Key8) {
            return Some(8);
        }

        if is_key_pressed(KeyCode::Key9) {
            return Some(9);
        }

        None

    }

    fn get_entity_control_group(&self, entity: Entity) -> Option<i32> {
        for control_group in &self.control_groups.groups {
            if control_group.entities.contains(&entity) {
                return Some(control_group.id);
            }
        }
        None
    }

    fn is_entity_in_control_group(&self, entity: Entity, control_group_id: i32) -> bool {
        for control_group in &self.control_groups.groups {
            if control_group.id == control_group_id && control_group.entities.contains(&entity) {
                return true;
            }
        }
        false
    }

    fn perform_retrieve_and_select_control_group(&mut self, world: &mut World) {

        if is_key_down(KeyCode::LeftShift) == false {
            self.perform_unselect_all(world);
        }

        let control_group_id = Self::get_first_number_key_pressed().expect("there must be a number key pressed when calling this function, there was none!");
        let control_group_entities: Vec<Entity> = self.control_groups.get(control_group_id).to_vec();

        for e in control_group_entities {
            if let Ok((controller, selectable)) = world.query_one_mut::<(&Controller, &mut Selectable)>(e) {
                if self.can_select_unit(controller) {
                    selectable.is_selected = true;
                }
            }
        }

    }

    fn perform_assign_control_group(&mut self, world: &mut World) {

        let control_group_id = Self::get_first_number_key_pressed().expect("there must be a number key pressed when calling this function, there was none!");

        let mut collected_entities = Vec::new();
        for (e, selectable) in world.query_mut::<&Selectable>() {
            if selectable.is_selected {
                collected_entities.push(e);
            }
        }
        
        self.control_groups.set(control_group_id, &collected_entities);

    }
    
    fn handle_selection(&mut self, world: &mut World) {

        if self.construction.is_previewing() {
            return;
        }

        // 0-9 keys allow you to retrieve previously set control groups
        let is_retrieving_and_selecting_control_group = Self::is_any_number_key_pressed() && is_key_down(KeyCode::LeftControl) == false;
        if is_retrieving_and_selecting_control_group {
            self.perform_retrieve_and_select_control_group(world);
            return;
        }

        // CTRL+0-9 allows you to group units into control groups that you can summon again by pressing 0-9
        let is_assigning_control_group = is_key_down(KeyCode::LeftControl) && Self::is_any_number_key_pressed();
        if is_assigning_control_group {
            self.perform_assign_control_group(world);
            return;
        }

        // CTRL+A should select all units
        let is_selecting_all = is_key_down(KeyCode::LeftControl) && is_key_pressed(KeyCode::A);
        if is_selecting_all {
            self.perform_select_all(world);
            return;
        }

        // CTRL+B selects the next idle constructor
        let is_finding_next_idle_constructor = is_key_down(KeyCode::LeftControl) && is_key_pressed(KeyCode::B);
        if is_finding_next_idle_constructor {
            self.perform_select_next_idle_constructor(world);
            return;
        }

        // CTRL+C selects your next commander unit
        let is_finding_next_commander = is_key_down(KeyCode::LeftControl) && is_key_pressed(KeyCode::C);
        if is_finding_next_commander {
            self.perform_select_next_commander(world);
            return;
        }

        // CTRL+Z allows you to select all units of the same kind as in the selection
        let is_selecting_all_units_of_same_kind = is_key_down(KeyCode::LeftControl) && is_key_pressed(KeyCode::Z);
        if is_selecting_all_units_of_same_kind {
            let all_selected_units = self.get_all_currently_selected_units(world);
            self.perform_selection_of_all_units_matching_type(all_selected_units, world);
            return;
        }

        let mouse_position: Vec2 = self.camera.mouse_screen_position();
        let is_adding_to_selection: bool = is_key_down(KeyCode::LeftShift);
        let is_removing_from_selection = is_key_down(KeyCode::LeftControl);
        let mut selection_turned_inactive = false;

        // allow to select all units under the mouse of the given type with Shift + Left Mouse x2
        let is_selecting_all_of_type = is_key_down(KeyCode::LeftShift) && is_mouse_button_released(MouseButton::Left) && self.selection.was_double_click();

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
            self.selection.register_click();
        }

        if self.selection.is_active {
            self.selection.end = mouse_position;
        }

        if selection_turned_inactive {
            self.perform_selection_with_bounds(world, is_adding_to_selection, is_removing_from_selection, is_selecting_all_of_type);
        }

    }

    fn get_entity_under_cursor(&self, world: &World) -> Option<Entity> {

        let mut closest_entity = None;
        let mut closest_distance = f32::MAX;
        let mouse_world_position: Vec2 = self.camera.mouse_world_position();

        for (e, (transform, bounds, body)) in world.query::<(&Transform, &Bounds, Option<&DynamicBody>)>().iter() {

            let current_distance_to_mouse = mouse_world_position.distance(transform.world_position);
            let is_position_within_bounds = if let Some(body) = body {
                let physics_bounds = body.physics_bounds();
                is_point_inside_rect(&mouse_world_position, &physics_bounds)
            } else {
                is_point_inside_rect(&mouse_world_position, &bounds.rect.offset(transform.world_position))
            };
            
            if is_position_within_bounds && current_distance_to_mouse < closest_distance {
                closest_distance = current_distance_to_mouse;
                closest_entity = Some(e);
            }

        }

        closest_entity
        
    }

    fn is_controller_attackable(&self, model: &RymdGameModel, controller: &Controller) -> bool {
        model.is_controller_attackable_by(self.game_player_id, controller)
    }

    fn is_controller_friendly(&self, model: &RymdGameModel, controller: &Controller) -> bool {
        model.is_controller_friendly_to(self.game_player_id, controller)
    }

    fn is_controller_controllable(&self, model: &RymdGameModel, controller: &Controller) -> bool {
        model.is_controller_controllable_by(self.game_player_id, controller)
    }

    fn is_entity_attackable(&self, entity: Entity, model: &RymdGameModel) -> bool {
        model.is_entity_attackable_by(self.game_player_id, entity)
    }

    fn is_entity_extractable(&self, entity: Entity, world: &World) -> bool {
        world.satisfies::<&ResourceSource>(entity).unwrap_or(false)
    }

    fn is_entity_extractor(&self, entity: Entity, world: &World) -> bool {
        world.satisfies::<&Extractor>(entity).unwrap_or(false)
    }

    fn is_entity_friendly(&self, entity: Entity, model: &RymdGameModel) -> bool {
        if let Ok(controller) = model.world.get::<&Controller>(entity) {
            model.is_controller_friendly_to(self.game_player_id, &controller)
        } else {
            false
        }
    }

    fn is_entity_controllable(&self, entity: Entity, model: &RymdGameModel) -> bool {
        if let Ok(controller) = model.world.get::<&Controller>(entity) {
            model.is_controller_controllable_by(self.game_player_id, &controller)
        } else {
            false
        }
    }

    fn handle_order(&mut self, model: &mut RymdGameModel, lockstep: &mut LockstepClient) {

        let mouse_position: Vec2 = self.camera.mouse_world_position();
        let should_cancel_current_orders: bool = is_key_released(KeyCode::S);

        let about_to_issue_attack_move_order = is_key_down(KeyCode::LeftControl) == false && is_key_released(KeyCode::LeftControl) == false && is_key_down(KeyCode::A);
        let about_to_issue_order = is_mouse_button_down(MouseButton::Right);
        let about_to_issue_any_order = about_to_issue_order || about_to_issue_attack_move_order;

        let should_issue_attack_move_order = is_key_down(KeyCode::LeftControl) == false && is_key_released(KeyCode::LeftControl) == false && is_key_released(KeyCode::A);
        let should_issue_order = is_mouse_button_released(MouseButton::Right);
        let should_issue_any_order = should_issue_order || should_issue_attack_move_order;

        if should_issue_any_order {

            let should_add = is_key_down(KeyCode::LeftShift);
            let should_group = is_key_down(KeyCode::LeftControl);
            let current_selection_end_point = self.ordering.points()[0];
            let entity_under_cursor = self.get_entity_under_cursor(&model.world);

            if let Some(target_entity) = entity_under_cursor {

                if self.is_entity_extractable(target_entity, &model.world) {
                    self.handle_extract_order(&mut model.world, target_entity, lockstep, should_add);
                } else if self.is_entity_friendly(target_entity, model) {
                    self.handle_repair_order(&mut model.world, target_entity, lockstep, should_add);
                } else if self.is_entity_attackable(target_entity, model) {
                    self.handle_attack_order(&mut model.world, target_entity, lockstep, should_add);
                }

            } else {
                self.handle_move_order(&mut model.world, current_selection_end_point, mouse_position, lockstep, should_group, should_add, should_issue_attack_move_order);
            }

        } else if about_to_issue_any_order {
            self.ordering.add_point(mouse_position);
        } else {
            self.ordering.clear_points();
        }

        if should_cancel_current_orders {
            self.cancel_current_orders(model, lockstep);
        }

    }

    fn cancel_current_orders(&self, model: &mut RymdGameModel, lockstep: &mut LockstepClient,) {

        for (e, (orderable, selectable)) in model.world.query::<(&Orderable, &Selectable)>().iter() {

            if selectable.is_selected && self.is_entity_controllable(e, model) {
                lockstep.cancel_current_orders(e);
                println!("[RymdGameView] cancelled orders for: {:?}", e);
            }
    
        }

    }

    fn handle_attack_order(&mut self, world: &mut World, target_entity: Entity, lockstep: &mut LockstepClient, should_add: bool) {

        for (e, (transform, orderable, selectable, attacker)) in world.query_mut::<(&Transform, &Orderable, &Selectable, &Attacker)>() {
    
            let is_target_self = target_entity == e;

            if selectable.is_selected && is_target_self == false {
                lockstep.send_attack_order(e, target_entity, should_add);
                println!("[RymdGameView] ordered: {:?} to attack: {:?}", e, target_entity);
            }
    
        }

    }

    fn handle_repair_order(&mut self, world: &mut World, target_entity: Entity, lockstep: &mut LockstepClient, should_add: bool) {

        let target_position = get_entity_position(world, target_entity).expect("target must have position!");

        for (e, (transform, orderable, selectable, constructor)) in world.query_mut::<(&Transform, &Orderable, &Selectable, &Constructor)>() {
    
            let is_target_self = target_entity == e;
            let is_capable_of_assisting = constructor.can_assist;

            if selectable.is_selected && is_target_self == false && is_capable_of_assisting {
                lockstep.send_repair_order(e, target_position, target_entity, should_add);
                println!("[RymdGameView] ordered: {:?} to repair: {:?}", e, target_entity);
            }
    
        }

    }

    fn handle_extract_order(&mut self, world: &mut World, target_entity: Entity, lockstep: &mut LockstepClient, should_add: bool) {

        for (e, (transform, orderable, selectable, extractor)) in world.query_mut::<(&Transform, &Orderable, &Selectable, &Extractor)>() {
    
            let is_target_self = target_entity == e;

            if selectable.is_selected && is_target_self == false {
                lockstep.send_extract_order(e, target_entity, should_add);
                println!("[RymdGameView] ordered: {:?} to extract from: {:?}", e, target_entity);
            }
    
        }

    }

    fn handle_move_order(&mut self, world: &mut World, current_selection_end_point: Vec2, current_mouse_world_position: Vec2, lockstep: &mut LockstepClient, should_group: bool, should_add: bool, should_attack: bool) {
        
        // we need to know the number of selected orderables so that we can distribute units along the line we draw for movement
        let number_of_selected_orderables = world.query_mut::<(&Orderable, &Selectable)>().into_iter().filter(|e| e.1.1.is_selected).count();

        // order the selectables by their distance from the current selection end point, this way we mostly retain the current arrangement the units are in and they hopefully make sorta-optimal moves
        let mut selectables_ordered_by_distance_to_end_point: Vec<(Entity, (&Transform, &Orderable, &Selectable))> = world.query_mut::<(&Transform, &Orderable, &Selectable)>().into_iter().filter(|e| e.1.2.is_selected).collect();
        selectables_ordered_by_distance_to_end_point.sort_by(|a, b| a.1.0.world_position.distance(current_selection_end_point).total_cmp(&b.1.0.world_position.distance(current_selection_end_point)));

        // calculate the centroid so that we can use it to figure out where units should go when moving as a group
        let centroid_of_selected_orderables = selectables_ordered_by_distance_to_end_point.iter().fold(Vec2::ZERO, |acc, v| acc + v.1.0.world_position) / selectables_ordered_by_distance_to_end_point.len() as f32;

        for (idx, (e, (transform, orderable, selectable))) in selectables_ordered_by_distance_to_end_point.into_iter().enumerate() {

            let current_order_point = if should_group {
                let offset_from_centroid = centroid_of_selected_orderables - transform.world_position;
                current_mouse_world_position - offset_from_centroid
            } else if number_of_selected_orderables > 1 {
                self.ordering.get_point(number_of_selected_orderables, idx)
            } else {
                current_mouse_world_position
            };

            if should_attack {
                lockstep.send_attack_move_order(e, current_order_point, should_add);
            } else {
                lockstep.send_move_order(e, current_order_point, should_add);
            }

            println!("[RymdGameView] ordered: {:?} to move to: {}", e, current_mouse_world_position);
    
        }
            
        self.ordering.clear_points();

    }

    fn is_anything_selected(&self, world: &World) -> bool {
        for (e, (transform, selectable, controller)) in world.query::<(&Transform, &Selectable, &Controller)>().iter() {
            if selectable.is_selected {
                return true;
            }
        }
        false
    }

    fn can_select_unit(&self, controller: &Controller) -> bool {
        controller.id == self.game_player_id
    }

    fn perform_select_all(&mut self, world: &mut World) {

        for (e, (transform, selectable, controller)) in world.query_mut::<(&Transform, &mut Selectable, &Controller)>() {

            if self.can_select_unit(controller) == false {
                continue;
            }
            
            selectable.is_selected = true;
            println!("[RymdGameView] selected: {:?}", e);

        }

    }

    fn perform_unselect_all(&mut self, world: &mut World) {

        for (e, (transform, selectable, controller)) in world.query_mut::<(&Transform, &mut Selectable, &Controller)>() {

            if self.can_select_unit(controller) == false {
                continue;
            }
            
            selectable.is_selected = false;

        }

        println!("[RymdGameView] unselected all");

    }

    fn perform_selection_with_bounds(&mut self, world: &mut World, is_additive: bool, is_removing: bool, is_all_of_type: bool) {

        let selection_rectangle = self.selection.as_rect();
        let world_selection_rectangle = self.camera.screen_to_world_rect(selection_rectangle);

        println!("[RymdGameView] attempted to select entities inside: {:?}", selection_rectangle);

        let mut now_selected_units = Vec::new();
        for (e, (transform, controller, bounds, body, selectable)) in world.query_mut::<(&Transform, &Controller, Option<&Bounds>, Option<&DynamicBody>, &mut Selectable)>() {

            if self.can_select_unit(controller) == false {
                continue;
            }

            let intersected_with_selection = if let Some(body) = body {
                let current_selectable_bounds = body.physics_bounds();
                current_selectable_bounds.intersect(world_selection_rectangle).is_some()
            } else if let Some(bounds) = bounds {
                let current_selectable_bounds = bounds.rect.offset(transform.world_position);
                current_selectable_bounds.intersect(world_selection_rectangle).is_some()
            } else {
                is_point_inside_rect(&transform.world_position, &world_selection_rectangle)
            };

            if is_removing && selectable.is_selected {
                selectable.is_selected = !(selectable.is_selected && intersected_with_selection);
            } else {
                selectable.is_selected = (selectable.is_selected && is_additive) || intersected_with_selection;
            }

            if selectable.is_selected {
                println!("[RymdGameView] selected: {:?}", e);
                now_selected_units.push(e);
            }

        }

        if is_all_of_type {
            self.perform_selection_of_all_units_matching_type(now_selected_units, world);
        }

    }

    fn get_all_currently_selected_units(&mut self, world: &mut World) -> Vec<Entity> {

        let mut all_selected_units = Vec::new();

        for (e, (controller, selectable)) in world.query_mut::<(&Controller, &Selectable)>() {
            if selectable.is_selected {
                all_selected_units.push(e);
            }
        }

        all_selected_units

    }

    fn perform_selection_of_all_units_matching_type(&mut self, selected_units: Vec<Entity>, world: &mut World) {
        for e in selected_units {
    
            let entity_blueprint_id = world.get::<&BlueprintIdentity>(e).unwrap().blueprint_id;
    
            for (e, (controller, blueprint_identity, selectable)) in world.query_mut::<(&Controller, &BlueprintIdentity, &mut Selectable)>() {
    
                if self.can_select_unit(controller) == false {
                    continue;
                }
    
                if blueprint_identity.blueprint_id == entity_blueprint_id {
                    selectable.is_selected = true;
                }
    
            }
    
        }
    }

    fn is_currently_issuing_an_attack_order(&self) -> bool {
        is_key_down(KeyCode::LeftControl) == false && is_key_released(KeyCode::LeftControl) == false && is_key_down(KeyCode::A)
    }
    
    fn draw_ordering(&self, world: &World) {
        
        if self.ordering.line.is_empty() {
            return;
        }

        if self.is_anything_selected(world) == false {
            return;
        }

        let line_thickness = 1.0;
        let mut last_point: Vec2 = self.ordering.points()[0];

        let is_currently_issuing_an_attack_order = self.is_currently_issuing_an_attack_order();
        let line_colour = if is_currently_issuing_an_attack_order { RED } else { GREEN };

        for p in self.ordering.points().iter().skip(1) {
            if last_point != *p {
                let current_screen_point = self.camera.world_to_screen(*p);
                let last_screen_point = self.camera.world_to_screen(last_point);
                draw_line(
                    last_screen_point.x,
                    last_screen_point.y,
                    current_screen_point.x,
                    current_screen_point.y,
                    line_thickness,
                    line_colour
                );
            }
            last_point = *p;
        }

        // let last_screen_point = self.camera.world_to_screen(last_point);
        // let current_screen_point: Vec2 = mouse_position().into();
        // draw_line(
        //     last_screen_point.x,
        //     last_screen_point.y,
        //     current_screen_point.x,
        //     current_screen_point.y,
        //     line_thickness,
        //     line_colour
        // );

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
    
    fn draw_pending_orders(&self, model: &RymdGameModel) {

    }

    fn draw_orders(&self, model: &RymdGameModel) {

        for (e, (transform, orderable, selectable)) in model.world.query::<(&Transform, &Orderable, &Selectable)>().iter() {

            if selectable.is_selected == false {
                continue;
            }

            let mut current_line_start = transform.world_position;
            let current_orders = orderable.orders(GameOrderType::Order);

            let order_line_thickness = 1.0;
            let order_line_head_size = self.camera.world_to_screen_scale_v(8.0);

            let order_line_colour_attack = RED.with_alpha(0.5);
            let order_line_colour_attack_move = RED.with_alpha(0.5);
            let order_line_colour_move = GREEN.with_alpha(0.5);

            for (i, order) in current_orders.iter().enumerate() {

                let order_line_colour = if let GameOrder::Attack(_) = order {
                    order_line_colour_attack
                } else if let GameOrder::AttackMove(_) = order {
                    order_line_colour_attack_move
                } else {
                    order_line_colour_move
                };

                if let Some(target_position) = order.get_target_position(model) {
                    let current_screen_position = self.camera.world_to_screen(current_line_start);
                    let target_screen_position = self.camera.world_to_screen(target_position);
                    if i == current_orders.len() - 1 {
                        draw_arrow(current_screen_position.x, current_screen_position.y, target_screen_position.x, target_screen_position.y, order_line_thickness, order_line_head_size, order_line_colour);
                    } else {
                        draw_line(current_screen_position.x, current_screen_position.y, target_screen_position.x, target_screen_position.y, order_line_thickness, order_line_colour);
                    }
                    current_line_start = target_position;
                }

            }

        }
        
    }

    pub fn move_camera_to_first_unselected_commander(&mut self, model: &mut RymdGameModel) {

        for (e, (commander, transform, selectable)) in model.world.query_mut::<(&Commander, &Transform, &Selectable)>() {

            if selectable.is_selected == false {
                self.camera.move_camera_to_position(transform.world_position);
                return;
            }

        }

    }

    pub fn update(&mut self, model: &mut RymdGameModel) {

        // this is tick 2 because at tick 0 and 1, the world isn't really initialized yet properly lol

        if model.current_tick == 2 {
            // #HACK: move the camera to the first unselected commander when the game starts
            self.move_camera_to_first_unselected_commander(model);
        }

        let mut beam_components_to_add = Vec::new();
        let mut selectable_components_to_add = Vec::new();
        let mut thruster_components_to_add = Vec::new();
        let mut bounds_components_to_add = Vec::new();

        for (e, (transform, constructor_or_extractor)) in model.world.query::<Without<(&Transform, Or<&Constructor, &Extractor>), &ParticleBeam>>().iter() {
            let emitter_config_name = "REPAIR";
            let particle_emitter = Emitter::new(self.resources.get_emitter_config_by_name(emitter_config_name));

            let constructor_beam_offset = match constructor_or_extractor {
                Or::Left(constructor) => constructor.beam_offset,
                Or::Right(extractor) => extractor.beam_offset,
                Or::Both(constructor, extractor) => constructor.beam_offset,
            };

            let constructor_beam = ParticleBeam { emitter: particle_emitter, offset: constructor_beam_offset };
            beam_components_to_add.push((e, constructor_beam));
        }

        for (e, (transform, controller, orderable)) in model.world.query::<Without<(&Transform, &Controller, Or<&Orderable, &Building>), &Selectable>>().iter() {
            let selectable = Selectable { is_selected: false };
            selectable_components_to_add.push((e, selectable));
        }

        for (e, (transform, thruster)) in model.world.query::<Without<(&Transform, &Thruster), &Particles>>().iter() {
            let emitter_config_name = if thruster.kind == ThrusterKind::Main { "STANDARD" } else { "STANDARD_TURN" };
            let particle_emitter = Particles { emitter: Emitter::new(self.resources.get_emitter_config_by_name(emitter_config_name)) };
            thruster_components_to_add.push((e, particle_emitter));
        }

        for (e, (transform, sprite, animated_sprite)) in model.world.query::<Without<(&Transform, Option<&Sprite>, Option<&AnimatedSprite>), &Bounds>>().iter() {

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

        for (e, c) in beam_components_to_add {
            let _ = model.world.insert_one(e, c);
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

    pub fn tick(&mut self, model: &mut RymdGameModel, ctx: &mut GameContext, dt: f32) {

        self.handle_selection(&mut model.world);
        self.handle_order(model, ctx.lockstep_mut());

    }

    fn draw_build_queue(&self, model: &RymdGameModel) {

        for (e, (orderable, controller)) in model.world.query::<(&Orderable, &Controller)>().iter() {

            if self.is_controller_friendly(model, controller) == false {
                continue
            }

            for order in orderable.orders(GameOrderType::Order) {
                if let GameOrder::Construct(order) = order
                    && let Some(blueprint_id) = order.blueprint_id
                    && order.is_self_order == false
                {
                    let position = vec2(order.x, order.y);
                    if let Some(blueprint) = model.blueprint_manager.get_blueprint(blueprint_id) {
                        ConstructionState::draw_building(&self.resources, blueprint, position, false);
                    }
                }
            }
        }

    }

    fn draw_beam_weapons(&self, world: &World) {

        let beam_thickness = 1.0;

        for (e, (beam, effect)) in world.query::<(&Beam, &Effect)>().iter() {

            // adjust period of sin so we get a nice fade in/out (halfway is fullly opaque, starts as transparent, ends as transparent)
            let current_beam_alpha = (PI*effect.current_lifetime_fraction()).sin();

            draw_line(
                beam.position.x,
                beam.position.y,
                beam.target.x,
                beam.target.y,
                beam_thickness,
                beam.color.with_alpha(current_beam_alpha)
            );  
        }

    }

    fn draw_sprites(&self, world: &World) {

        for (e, (transform, sprite, state)) in world.query::<(&Transform, Or<&Sprite, &AnimatedSprite>, Option<&EntityState>)>().iter() {
            match sprite {
                Or::Left(sprite) => self.draw_sprite(state, sprite, transform),
                Or::Right(animated_sprite) => self.draw_animated_sprite(state, animated_sprite, transform),
                Or::Both(sprite, animated_sprite) => {
                    self.draw_sprite(state, sprite, transform);
                    self.draw_animated_sprite(state, animated_sprite, transform);
                },
            }
        }

    }

    fn draw_animated_sprite(&self, state: Option<&EntityState>, sprite: &AnimatedSprite, transform: &Transform) {
        let is_sprite_flipped = false;
        let sprite_texture_alpha = entity_state_to_alpha(state);
        let sprite_texture_handle = self.resources.get_texture_by_name(&sprite.texture);
        draw_texture_centered_with_rotation_frame(
            &sprite_texture_handle,
            transform.world_position.x,
            transform.world_position.y,
            WHITE.with_alpha(sprite_texture_alpha),
            transform.world_rotation,
            sprite.current_frame,
            sprite.h_frames,
            is_sprite_flipped
        );
    }

    fn draw_sprite(&self, state: Option<&EntityState>, sprite: &Sprite, transform: &Transform) {
        let sprite_texture_alpha = entity_state_to_alpha(state);
        let sprite_texture_handle = self.resources.get_texture_by_name(&sprite.texture);
        draw_texture_centered_with_rotation(
            &sprite_texture_handle,
            transform.world_position.x,
            transform.world_position.y,
            WHITE.with_alpha(sprite_texture_alpha),
            transform.world_rotation
        );
    }

    fn draw_selectables(&self, world: &mut World) {

        let bounds_thickness = 1.0;
        let bounds_colour = GREEN.with_alpha(0.5);

        for (e, (transform, selectable, bounds)) in world.query::<(&Transform, &Selectable, &Bounds)>().iter() {

            if selectable.is_selected {
                let screen_position = self.camera.world_to_screen(transform.world_position);
                let screen_radius = self.camera.world_to_screen_scale_v(bounds.as_radius() * 1.5);
                draw_circle_lines(
                    screen_position.x,
                    screen_position.y,
                    screen_radius,
                    bounds_thickness,
                    bounds_colour
                );

                let mut current_offset = 0;
                for i in 0..9 {
                    if self.is_entity_in_control_group(e, i) {
                        let screen_text_size = 24.0;
                        let screen_position = self.camera.world_to_screen(transform.world_position);
                        let screen_radius = self.camera.world_to_screen_scale_v(bounds.as_radius() * 1.5);
                        let screen_position_offset = screen_position + vec2(screen_radius, screen_radius);
                        let screen_offset = screen_text_size * current_offset as f32;
                        
                        draw_text(&i.to_string(), screen_position_offset.x + screen_offset, screen_position_offset.y, screen_text_size, WHITE);
                        current_offset += 1;
                    }
                }
            }

        }

    }

    fn update_constructor_beams(&mut self, model: &RymdGameModel) {

        for (e, (transform, body, orderable, constructor, beam)) in model.world.query::<(&Transform, &DynamicBody, &Orderable, &Constructor, &mut ParticleBeam)>().iter() {

            let center_of_dynamic_body = body.bounds().center();

            if let Some(current_order @ GameOrder::Construct(_)) = orderable.first_order(GameOrderType::Order) && constructor.is_constructing() {

                let current_target_position = current_order.get_target_position(model).unwrap();

                emit_construction_beam(
                    beam,
                    transform,
                    current_target_position,
                    body.velocity(),
                    constructor.build_speed as f32 / 100.0
                );

            }

        }

    }

    fn update_extractor_beams(&mut self, model: &RymdGameModel) {

        for (e, (transform, body, orderable, extractor, beam)) in model.world.query::<(&Transform, &DynamicBody, &Orderable, &Extractor, &mut ParticleBeam)>().iter() {

            let center_of_dynamic_body = body.bounds().center();

            if let Some(current_order @ GameOrder::Extract(_)) = orderable.first_order(GameOrderType::Order) && extractor.is_extracting() {

                let Some(current_target_position) = current_order.get_target_position(model) else { continue; };

                emit_reclaim_beam(
                    beam,
                    transform,
                    current_target_position,
                    body.velocity(),
                    extractor.extraction_speed as f32 / 100.0
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

    fn update_impacts(&mut self, world: &World) {

        let impact_velocity = 64.0;

        for (e, (transform, effect, impact)) in world.query::<(&Transform, &Effect, &Impact)>().iter() {

            let emitter = self.resources.get_emitter_by_name("IMPACT");
            
            emitter.config.lifetime = effect.total_lifetime;
            emitter.config.initial_velocity = impact_velocity;
            emitter.config.initial_direction = transform.world_rotation.as_vector();
            emitter.config.size = 1.0;
            emitter.emit(transform.world_position, 4);

        }

    }

    fn draw_particles(&mut self, world: &mut World) {

        for (_, emitter) in &mut self.resources.particle_emitters {
            emitter.draw(Vec2::ZERO);
        }

        for (e, (transform, beam)) in world.query_mut::<(&Transform, &mut ParticleBeam)>() {
            beam.emitter.draw(transform.world_position);
        }

        for (e, (transform, particles)) in world.query_mut::<(&Transform, &mut Particles)>() {
            particles.emitter.draw(transform.world_position);
        }

    }

    fn draw_debug_ui(&mut self, model: &mut RymdGameModel, ctx: &mut GameContext) {
        
        let mouse_world_position = self.camera.mouse_world_position();
        ctx.debug_text().draw_text(format!("mouse (screen) position: {:?}", mouse_position()), TextPosition::TopLeft, WHITE);
        ctx.debug_text().draw_text(format!("mouse (world) position: ({:.1}, {:.1})", mouse_world_position.x, mouse_world_position.y), TextPosition::TopLeft, WHITE);
        ctx.debug_text().draw_text(format!("number of entities: {}", model.world.len()), TextPosition::TopLeft, WHITE);
        ctx.debug_text().draw_text(format!("collision responses: {}", model.physics_manager.number_of_active_collision_responses()), TextPosition::TopLeft, WHITE);
        ctx.debug_text().draw_text(format!(" - shift+c to toggle bounds debug (enabled: {})", self.debug.render_bounds), TextPosition::TopLeft, WHITE);
        ctx.debug_text().draw_text(format!(" - shift+k to toggle kinematics debug (enabled: {})", self.debug.render_kinematic), TextPosition::TopLeft, WHITE);
        ctx.debug_text().draw_text(format!(" - shift+s to toggle spatial debug (enabled: {})", self.debug.render_spatial), TextPosition::TopLeft, WHITE);

        if ctx.lockstep_mut().is_singleplayer() {
            ctx.debug_text().draw_text("press tab to switch the current player!", TextPosition::TopLeft, WHITE);
            if is_key_pressed(KeyCode::Tab) {
                self.switch_player_id_to_next(&mut model.world);
            }
        }

        let should_toggle_debug_bounds = is_key_down(KeyCode::LeftShift) && is_key_released(KeyCode::C);
        if should_toggle_debug_bounds {
            self.debug.render_bounds = !self.debug.render_bounds;
        }

        let should_toggle_kinematic_debug = is_key_down(KeyCode::LeftShift) && is_key_released(KeyCode::K);
        if should_toggle_kinematic_debug {
            self.debug.render_kinematic = !self.debug.render_kinematic
        }

        let should_toggle_spatial_debug = is_key_down(KeyCode::LeftShift) && is_key_released(KeyCode::S);
        if should_toggle_spatial_debug {
            self.debug.render_spatial = !self.debug.render_spatial
        }

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
                draw_texture(&bg_texture, (x - 1) as f32 * bg_w - offset_x, (y - 1) as f32 * bg_h - offset_y, WHITE);
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

    fn draw_text_construction_queue_ui(&mut self, model: &mut RymdGameModel, debug: &mut DebugText) {

        for (e, (constructor, orderable, selectable)) in model.world.query::<(&Constructor, &Orderable, &Selectable)>().iter() {

            if orderable.is_queue_empty(GameOrderType::Construct) {
                continue
            }

            if selectable.is_selected == false {
                continue;
            }

            debug.skip_line(TextPosition::BottomRight);

            let mut blueprints_in_construction = Vec::new();

            for order in orderable.orders(GameOrderType::Construct) {
                if let GameOrder::Construct(construct_order) = order {
                    
                    let current_blueprint_id = if let Some(blueprint_id) = construct_order.blueprint_id {
                        Some(blueprint_id)
                    } else if let Some(entity) = construct_order.entity() && let Ok(blueprint_identity) = model.world.get::<&BlueprintIdentity>(entity) {
                        Some(blueprint_identity.blueprint_id)
                    } else {
                        None
                    };

                    if let Some(current_blueprint_id) = current_blueprint_id {

                        if let Some((last_id, last_count)) = blueprints_in_construction.last().cloned() {
                            if last_id == current_blueprint_id {
                                blueprints_in_construction.pop();
                                blueprints_in_construction.push((current_blueprint_id, last_count + 1));
                            } else {
                                blueprints_in_construction.push((current_blueprint_id, 1));
                            }
                        } else {
                            blueprints_in_construction.push((current_blueprint_id, 1));
                        }

                    }

                }
            }

            for (blueprint_id, blueprint_count) in blueprints_in_construction {
                let Some(blueprint) = model.blueprint_manager.get_blueprint(blueprint_id) else { continue };
                debug.draw_text(format!(" - {}x {}", blueprint_count, blueprint.name), TextPosition::BottomRight, WHITE);
            }

            debug.draw_text("construction queue", TextPosition::BottomRight, WHITE);

        }

    }

    fn draw_game_ui(&mut self, ui_ctx: &egui::Context, model: &mut RymdGameModel, ctx: &mut GameContext) {

        egui::Window::new("")
            .title_bar(false)
            .collapsible(false)
            .resizable(false)
            .anchor(Align2::CENTER_TOP, (0.0, 16.0))
            .frame(egui::Frame::default())
            .show(ui_ctx, |ui| {

                let current_metal = current_metal(self.game_player_id, &model.world);
                let maximum_metal = max_metal(self.game_player_id, &model.world);
                let current_metal_income = current_metal_income(self.game_player_id, &model.world);
                let current_metal_excess = 0;

                let current_energy = current_energy(self.game_player_id, &model.world);
                let maximum_energy = max_energy(self.game_player_id, &model.world);
                let current_energy_income = current_energy_income(self.game_player_id, &model.world);
                let current_energy_excess = 0;

                let current_metal_proportion = current_metal / maximum_metal;
                let current_energy_proportion = current_energy / maximum_energy;

                let metal_progress_bar = egui::ProgressBar::new(current_metal_proportion).text(format!("Metal = {:.0} / {:.0} ({:.0})", current_metal, maximum_metal, current_metal_income));
                let energy_progress_bar = egui::ProgressBar::new(current_energy_proportion).text(format!("Energy = {:.0} / {:.0} ({:.0})", current_energy, maximum_energy, current_energy_income));

                ui.add(metal_progress_bar);
                ui.add(energy_progress_bar);

        });

        if self.current_selection_has_constructor_unit(&model.world) {
            self.draw_text_construction_ui(model, ctx.debug_text());
            self.draw_text_construction_queue_ui(model, ctx.debug_text());
        }

    }

    fn draw_health_labels(&self, world: &World) {

        for (e, (transform, health)) in world.query::<(&Transform, &Health)>().iter() {
            let health_label_position = transform.world_position + vec2(0.0, -32.0);
            draw_text_centered(&format!("{}/{}", health.current_health(), health.full_health()), health_label_position.x, health_label_position.y, 24.0, WHITE);
        }

    }

    fn draw_resource_labels(&self, world: &World) {
        
        for (e, (transform, resource_source)) in world.query::<(&Transform, &ResourceSource)>().iter() {
            let resource_label_position = transform.world_position + vec2(0.0, -32.0);

            if resource_source.total_energy > 0.0 && resource_source.total_metal > 0.0 {
                draw_text_centered(
                    &format!("{:.0}/{:.0} m, {:.0}/{:.0} e", resource_source.current_metal, resource_source.total_metal, resource_source.total_energy, resource_source.current_energy),
                    resource_label_position.x,
                    resource_label_position.y,
                    24.0,
                    WHITE
                );
            } else if resource_source.total_metal > 0.0 {
                draw_text_centered(
                    &format!("{:.0}/{:.0} m", resource_source.current_metal, resource_source.total_metal),
                    resource_label_position.x,
                    resource_label_position.y,
                    24.0,
                    WHITE
                );
            } else if resource_source.total_energy > 0.0 {
                draw_text_centered(
                    &format!("{:.0}/{:.0} e", resource_source.current_energy, resource_source.total_energy),
                    resource_label_position.x,
                    resource_label_position.y,
                    24.0,
                    WHITE
                );
            }

        }

    }

    fn draw_build_time_labels(&self, world: &World) {

        for (e, (transform, bounds, health, &state)) in world.query::<(&Transform, &Bounds, &Health, &EntityState)>().iter() {

            if state != EntityState::Ghost {
                continue
            }

            let health_difference = health.current_health() - health.last_health();
            let health_difference_to_max = (health.full_health() - health.current_health()).max(0.0);
            let health_difference_per_second = (1.0 / RymdGameModel::TIME_STEP) * health_difference;
            let time_remaining_seconds = health_difference_to_max / health_difference_per_second;
            
            let time_label_position = transform.world_position + vec2(0.0, -bounds.as_radius());
            draw_text_centered(&format!("{:.0}s", time_remaining_seconds), time_label_position.x, time_label_position.y, 24.0, WHITE);

        }

    }

    fn draw_body_bounds(&self, world: &World) {

        let bounds_line_thickness = 2.0;

        for (e, body) in world.query::<&DynamicBody>().iter() {
            let bounds_colour = if body.is_enabled { GREEN } else { YELLOW };
            let screen_bounds = self.camera.world_to_screen_rect(body.bounds());
            draw_rectangle_lines_centered_with_rotation(screen_bounds.x, screen_bounds.y, screen_bounds.w, screen_bounds.h, bounds_line_thickness, bounds_colour, body.kinematic.orientation);
        }

    }

    fn draw_kinematic_debug(&self, world: &World) {

        let kinematic_line_thickness = 1.0;
        let kinematic_line_head_size = 8.0;

        for (e, body) in world.query::<&DynamicBody>().iter() {

            let body_position = body.position();
            let body_direction = body.orientation().as_vector();

            let body_velocity = body.velocity();
            let body_angular_velocity = body.angular_velocity();

            let screen_body_position = self.camera.world_to_screen(body_position);
            let screen_body_velocity_position = self.camera.world_to_screen(body_position + body_velocity);

            draw_arrow(
                screen_body_position.x,
                screen_body_position.y,
                screen_body_velocity_position.x,
                screen_body_velocity_position.y,
                kinematic_line_thickness,
                kinematic_line_head_size,
                GREEN
            );

            let body_left = body_direction.perpendicular_ccw();
            let body_turn_direction_normalised = body_left * body_angular_velocity / body_angular_velocity.abs();
            let body_turn_direction = body_turn_direction_normalised * body_angular_velocity.abs();

            let screen_body_turn_vector_length = 16.0;
            let screen_body_turn_position = self.camera.world_to_screen(body_position + body_turn_direction * screen_body_turn_vector_length);

            draw_arrow(
                screen_body_position.x,
                screen_body_position.y,
                screen_body_turn_position.x,
                screen_body_turn_position.y,
                kinematic_line_thickness,
                kinematic_line_head_size,
                RED
            );

        }

    }

    fn draw_spatial_debug(&self, model: &RymdGameModel) {

        let spatial_line_thickness = 2.0;
        let spatial_line_font_size = 16.0;

        for (bucket_position, bucket) in model.spatial_manager.buckets() {

            let screen_space_bucket_position = self.camera.world_to_screen(bucket_position.as_vec2());
            let screen_space_bucket_size = self.camera.world_to_screen_scale_v(RymdGameModel::SPATIAL_BUCKET_SIZE as f32);
            let screen_space_bucket_center_position = screen_space_bucket_position + vec2(screen_space_bucket_size, screen_space_bucket_size) * 0.5;

            draw_rectangle_lines(
                screen_space_bucket_position.x,
                screen_space_bucket_position.y,
                screen_space_bucket_size,
                screen_space_bucket_size,
                spatial_line_thickness,
                GREEN
            );

            draw_text_centered(
                &format!("{}", bucket.len()),
                screen_space_bucket_center_position.x,
                screen_space_bucket_center_position.y + spatial_line_font_size * 0.5,
                spatial_line_font_size,
                WHITE
            );

        }

    }

    pub fn draw(&mut self, model: &mut RymdGameModel, ctx: &mut GameContext, dt: f32) {

        self.camera.tick(dt);

        self.update_constructor_beams(model);
        self.update_extractor_beams(model);
        self.update_thrusters(&model.world);
        self.update_impacts(&model.world);

        self.draw_background_texture(screen_width(), screen_height(), self.camera.world_position());

        self.draw_pending_orders(model);
        self.draw_orders(model);

        self.draw_selection();
        self.draw_selectables(&mut model.world);
        self.draw_ordering(&model.world);

        if self.debug.render_bounds {
            self.draw_body_bounds(&model.world);
        }

        if self.debug.render_kinematic {
            self.draw_kinematic_debug(&model.world);
        }

        if self.debug.render_spatial {
            self.draw_spatial_debug(model);
        }

        self.camera.push();

        self.draw_build_queue(model);

        self.construction.tick_and_draw(model, &self.camera, &self.resources, ctx.lockstep_mut());

        self.draw_particles(&mut model.world);
        self.draw_beam_weapons(&model.world);
        self.draw_sprites(&model.world);

        // self.draw_health_labels(&model.world);
        self.draw_resource_labels(&model.world);
        self.draw_build_time_labels(&model.world);
        
        self.camera.pop();

    }

    pub fn draw_ui(&mut self, ui_ctx: &egui::Context, model: &mut RymdGameModel, ctx: &mut GameContext) {

        self.draw_game_ui(ui_ctx, model, ctx);
        self.draw_debug_ui(model, ctx);

    }

}

fn emit_construction_beam(beam: &mut ParticleBeam, source_world_transform: &Transform, target_world_position: Vec2, source_body_velocity: Vec2, emission_rate: f32) {

    let beam_emitter_offset = beam.offset.rotated_by(source_world_transform.world_rotation + (PI/2.0));
    let beam_emit_target = target_world_position;
    let beam_emit_delta = beam_emit_target - (source_world_transform.world_position + beam_emitter_offset);

    let beam_emit_direction = beam_emit_delta.normalize();
    let beam_emit_distance = beam_emit_delta.length();

    beam.emitter.config.initial_direction = beam_emit_direction;
    beam.emitter.config.initial_velocity = ((source_body_velocity + beam_emit_direction * 16.0).length()).max(48.0);

    // calculate lifetime depending on how far we want the particle to go (preferably reaching our target!)
    let lifetime = (beam_emit_distance / beam.emitter.config.initial_velocity) * 1.1;
    beam.emitter.config.lifetime = lifetime;

    beam.emitter.emit(beam.offset.rotated_by(source_world_transform.world_rotation + (PI/2.0)), emission_rate as usize);

}

fn emit_reclaim_beam(beam: &mut ParticleBeam, source_world_transform: &Transform, target_world_position: Vec2, source_body_velocity: Vec2, emission_rate: f32) {

    let beam_emitter_offset = beam.offset.rotated_by(source_world_transform.world_rotation + (PI/2.0));
    let beam_emit_target = target_world_position;
    let beam_emit_delta = beam_emit_target - (source_world_transform.world_position + beam_emitter_offset);

    let beam_emit_direction = -beam_emit_delta.normalize();
    let beam_emit_distance = beam_emit_delta.length();

    beam.emitter.config.initial_direction = beam_emit_direction;
    beam.emitter.config.initial_velocity = ((source_body_velocity + beam_emit_direction * 16.0).length()).max(48.0);

    // calculate lifetime depending on how far we want the particle to go (preferably reaching our target!)
    let lifetime = (beam_emit_distance / beam.emitter.config.initial_velocity) * 1.1;
    beam.emitter.config.lifetime = lifetime;

    beam.emitter.emit(beam.offset.rotated_by(source_world_transform.world_rotation + (PI/2.0)) + (beam_emit_target - source_world_transform.world_position), emission_rate as usize);

}

impl Drop for RymdGameView {
    fn drop(&mut self) {
        self.unload_resources();
    }
}
