use std::collections::HashMap;

use macroquad_particles::{EmitterConfig, Emitter};
use utility::{is_point_inside_rect, draw_texture_centered_with_rotation, set_texture_filter, draw_texture_centered_with_rotation_frame, DebugText, TextPosition, AsVector, RotatedBy, draw_arrow};
use lockstep_client::step::LockstepClient;
use macroquad_particles::*;
use macroquad::prelude::*;
use hecs::*;

use crate::model::{RymdGameModel, Orderable, Transform, Sprite, AnimatedSprite, GameOrdersExt, DynamicBody, Thruster, Ship, ThrusterKind};

struct SelectionBox {
    is_active: bool,
    start: Vec2,
    end: Vec2
}

impl SelectionBox {

    fn new() -> SelectionBox {
        SelectionBox {
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
    rect: Vec2
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
    selection: SelectionBox,
    resources: Resources
}

impl RymdGameView {

    pub fn new() -> RymdGameView {
        RymdGameView {
            selection: SelectionBox::new(),
            resources: Resources::new()
        }
    }

    pub async fn load_resources(&mut self) {
        self.resources.load().await;
    }

    pub fn unload_resources(&mut self) {

    }
    
    fn handle_selection(&mut self, world: &mut World) {

        let mouse_position: Vec2 = mouse_position().into();
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
            self.perform_selection(world);
        }

    }

    fn handle_order(&mut self, world: &mut World, lockstep: &mut LockstepClient) {

        if is_mouse_button_pressed(MouseButton::Right) {

            let should_add = is_key_down(KeyCode::LeftShift);
            let current_mouse_position: Vec2 = mouse_position().into();

            for (e, (orderable, selectable)) in world.query_mut::<(&Orderable, &Selectable)>() {
                if selectable.is_selected {
                    lockstep.send_move_order(e, current_mouse_position, should_add);
                    println!("[RymdGameView] ordered: {:?} to move to: {}", e, current_mouse_position);
                }
            }

        }

    }

    fn perform_selection(&mut self, world: &mut World) {

        let selection_rectangle = self.selection.as_rect();

        println!("[RymdGameView] attempted to select entities inside: {:?}", selection_rectangle);

        for (e, (transform, orderable, selectable)) in world.query_mut::<(&Transform, &Orderable, &mut Selectable)>() {
            selectable.is_selected = is_point_inside_rect(&transform.local_position, &selection_rectangle);
            if selectable.is_selected {
                println!("[RymdGameView] selected: {:?}", e);
            }
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

    fn draw_orders(&self, world: &mut World) {

        for (e, (transform, orderable, selectable)) in world.query::<(&Transform, &Orderable, &Selectable)>().iter() {

            if selectable.is_selected == false {
                continue;
            }

            let mut current_line_start = transform.world_position;

            let order_line_thickness = 1.0;
            let order_line_head_size = 8.0;

            for (i, order) in orderable.orders.iter().enumerate() {
                if let Some(target_position) = order.get_target_position(world) {
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

        for (e, (transform, orderable)) in model.world.query::<Without<(&Transform, &Orderable), &Selectable>>().iter() {
            let selectable = Selectable { is_selected: false };
            selectable_components_to_add.push((e, selectable));
        }

        for (e, (transform, thruster)) in model.world.query::<Without<(&Transform, &Thruster), &Particles>>().iter() {
            let emitter_config_name = if thruster.kind == ThrusterKind::Main { "STANDARD" } else { "STANDARD_TURN" };
            let particle_emitter = Particles { emitter: Emitter::new(self.resources.get_emitter_config_by_name(emitter_config_name)) };
            thruster_components_to_add.push((e, particle_emitter));
        }

        for (e, c) in selectable_components_to_add {
            let _ = model.world.insert_one(e, c);
        }

        for (e, c) in thruster_components_to_add {
            let _ = model.world.insert_one(e, c);
        }

    }

    pub fn tick(&mut self, world: &mut World, lockstep: &mut LockstepClient) {
        self.handle_selection(world);
        self.handle_order(world, lockstep);
    }

    fn draw_sprites(&self, world: &mut World) {
        for (e, (transform, sprite)) in world.query::<(&Transform, &Sprite)>().iter() {
            let sprite_texture_handle = self.resources.get_texture_by_name(&sprite.texture);
            draw_texture_centered_with_rotation(sprite_texture_handle, transform.world_position.x, transform.world_position.y, WHITE, transform.world_rotation);
        }
    }

    fn draw_animated_sprites(&self, world: &mut World) {
        for (e, (transform, body, sprite)) in world.query::<(&Transform, Option<&DynamicBody>, &AnimatedSprite)>().iter() {
            let is_sprite_flipped = false;
            let sprite_texture_handle = self.resources.get_texture_by_name(&sprite.texture);
            draw_texture_centered_with_rotation_frame(sprite_texture_handle, transform.world_position.x, transform.world_position.y, WHITE, transform.world_rotation, sprite.current_frame, sprite.v_frames, is_sprite_flipped);
        }
    }

    fn update_thrusters(&mut self, world: &mut World) {

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

    fn draw_thrusters(&mut self, world: &mut World) {

        for (_, emitter) in &mut self.resources.particle_emitters {
            emitter.draw(Vec2::ZERO);
        }

        for (e, (transform, particles)) in world.query_mut::<(&Transform, &mut Particles)>() {
            particles.emitter.draw(transform.world_position);
        }

    }

    fn draw_debug_ui(&self, debug: &mut DebugText) {
        
        debug.draw_text(format!("mouse position: {:?}", mouse_position()), TextPosition::TopLeft, WHITE);

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

    pub fn draw(&mut self, world: &mut World, debug: &mut DebugText) {

        self.update_thrusters(world);

        let screen_center: Vec2 = vec2(screen_width(), screen_height()) / 2.0;
        self.draw_background_texture(screen_width(), screen_height(), screen_center);

        self.draw_thrusters(world);
        self.draw_orders(world);

        self.draw_sprites(world);
        self.draw_animated_sprites(world);
        self.draw_selection();

        self.draw_debug_ui(debug);

    }

}

impl Drop for RymdGameView {
    fn drop(&mut self) {
        self.unload_resources();
    }
}