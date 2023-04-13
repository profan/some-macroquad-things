use std::process::exit;
use std::{f32::consts::PI, thread::current};
use std::sync::Arc;

use hecs::{World, Bundle};
use macroquad::prelude::*;
use utility::{GameCamera, DebugText, create_camera_from_game_camera, TextPosition, intersect_ray_with_plane, draw_cube_ex, draw_cube_wires_ex, rotate_relative_to_origin, AdjustHue, draw_with_transformation, AsAngle, RotatedBy, normalize, is_point_inside_screen, is_point_inside_rect, draw_circle_lines_3d, FromRotationArcAround};
use rhai::{Engine, EvalAltResult, AST, NativeCallContext, Scope, OptimizationLevel};

const WORLD_UP: Vec3 = Vec3::Y;

struct SelectionBox {
    active: bool,
    select: bool,
    start: Vec2,
    end: Vec2
}

enum SelectionKind {
    Line
}

struct SelectionState {
    active: bool,
    order_points: Vec<Vec3>
}

impl SelectionState {
    pub fn new() -> SelectionState {
        SelectionState {
            active: false,
            order_points: Vec::new()
        }
    }
}

impl SelectionBox {

    pub fn new() -> SelectionBox {
        SelectionBox {
            active: false,
            select: false,
            start: Vec2::ZERO,
            end: Vec2::ZERO
        }
    }

    pub fn bounds(&self) -> Rect {

        let min_x = self.start.x.min(self.end.x);
        let min_y = self.start.y.min(self.end.y);

        let max_x = self.start.x.max(self.end.x);
        let max_y = self.start.y.max(self.end.y);

        Rect {
            x: min_x,
            y: min_y,
            w: max_x - min_x,
            h: max_y - min_y
        }

    }

}

#[derive(Clone)]
struct Transform {
    position: Vec3,
    rotation: Quat,
    velocity: Vec3
}

struct Selectable {
    is_selected: bool
}

impl Selectable {
    pub fn new() -> Selectable {
        Selectable {
            is_selected: false
        }
    }
}

struct Orderable {
    target: Option<Vec3>
}

impl Orderable {
    pub fn new() -> Orderable {
        Orderable {
            target: None
        }
    }
}

#[derive(Clone)]
struct Character {
    current_state: String,
    movement_speed: f32,
    script: Arc<AST>
}

impl Character {
    pub fn new(movement_speed: f32, script: Arc<AST>) -> Character {
        Character {
            current_state: String::new(),
            movement_speed,
            script
        }
    }
}

struct Game {

    camera: GameCamera,
    debug_text: DebugText,
    engine: Engine,
    world: World,

    fallback_script: Arc<AST>,
    character_script: Option<Arc<AST>>

}

impl Game {

    pub fn new() -> Game {

        let camera = GameCamera::new();
        let debug_text = DebugText::new();

        let mut engine =  Engine::new();

        let max_expr_depth = 128;
        let max_function_expr_depth = 64;
        engine.set_max_expr_depths(max_expr_depth, max_function_expr_depth);

        let world = World::new();
        let fallback_script = Arc::new(AST::empty());

        Game {
            camera,
            debug_text,
            engine,
            world,
            fallback_script,
            character_script: None
        }

    }

}

fn handle_camera_input(active: &mut GameCamera, dt: f32) {

    let is_turn_left_pressed = is_key_down(KeyCode::Q);
    let is_turn_right_pressed = is_key_down(KeyCode::E);

    let is_forwards_pressed = is_key_down(KeyCode::W) || is_key_down(KeyCode::Up);
    let is_backwards_pressed = is_key_down(KeyCode::S) || is_key_down(KeyCode::Down);
    let is_left_pressed = is_key_down(KeyCode::A) || is_key_down(KeyCode::Left);
    let is_right_pressed = is_key_down(KeyCode::D) || is_key_down(KeyCode::Right);

    let is_up_pressed = is_key_down(KeyCode::Space);
    let is_down_pressed = is_key_down(KeyCode::LeftControl);

    let mut camera_movement_delta = Vec3::ZERO;
    let mut camera_rotation_delta = Quat::IDENTITY;

    let forward_in_plane = vec3(active.forward().x, 0.0, active.forward().z);
    let left_in_plane = vec3(active.left().x, 0.0, active.left().z);
    let up_in_plane = WORLD_UP;

    if is_forwards_pressed {
        camera_movement_delta += forward_in_plane * active.parameters.movement_speed * dt;
    }

    if is_backwards_pressed {
        camera_movement_delta -= forward_in_plane * active.parameters.movement_speed * dt;
    }

    if is_left_pressed {
        camera_movement_delta += left_in_plane * active.parameters.movement_speed * dt;
    }

    if is_right_pressed {
        camera_movement_delta -= left_in_plane * active.parameters.movement_speed * dt;
    }

    if is_up_pressed {
        camera_movement_delta += up_in_plane * active.parameters.movement_speed * dt;
    }

    if is_down_pressed {
        camera_movement_delta -= up_in_plane * active.parameters.movement_speed * dt;
    }

    if is_turn_left_pressed {
        let current_direction = (active.target - active.position).normalize();
        let new_desired_direction = Quat::from_rotation_y(active.parameters.rotation_speed * dt) * current_direction;
        camera_rotation_delta = Quat::from_rotation_arc(current_direction, new_desired_direction);
    }

    if is_turn_right_pressed {
        let current_direction = (active.target - active.position).normalize();
        let new_desired_direction = Quat::from_rotation_y(-(active.parameters.rotation_speed * dt)) * current_direction;
        camera_rotation_delta = Quat::from_rotation_arc(current_direction, new_desired_direction);
    }

    active.position += camera_movement_delta;
    active.target += camera_movement_delta;
    
    // #NOTE: position and target can be flipped here for basically orbit camera
    active.target = rotate_relative_to_origin(active.position, active.target, camera_rotation_delta);

}

fn draw_debug_text(game: &mut Game) {

    let dt = get_frame_time() * 1000.0;
    game.debug_text.draw_text(format!("frametime: {:.2} ms", dt), TextPosition::TopRight, BLACK);

}

fn draw_character_with_script(engine: &Engine, transform: &Transform, character: &Character) {
    
    draw_with_transformation(transform.position, transform.rotation, || {

        let mut current_scope = Scope::new();
        current_scope.push_constant("transform", transform.clone());
        
        let error = engine.eval_ast_with_scope::<()>(&mut current_scope, &character.script);
        if let Result::Err(error) = error {
            println!("got error: {}", error.to_string());
        }

    });

}

fn draw_part(position: Vec3, rotation: Quat, size: Vec3) {

    let wireframe = true;
    let solid = true;

    if wireframe {
        draw_cube_wires_ex(
            position,
            rotation,
            size,
            WHITE.darken(0.25)
        );
    } 
    
    if solid {
        draw_cube_ex(
            position,
            rotation,
            size,
            None,
            WHITE
        );
    }

}

fn draw_selectable(transform: &Transform) {

    let selectable_circle_size = 1.0;
    let selectable_circle_thickness = 1.0;

    draw_circle_lines_3d(
        transform.position,
        selectable_circle_size,
        selectable_circle_thickness,
        GREEN
    );

}

fn draw_characters(engine: &Engine, world: &World) {

    for (_entity, (transform, character)) in world.query::<(&Transform, &Character)>().iter() {
        draw_character_with_script(engine, transform, character);
    }

}

fn draw_selectables(world: &World) {

    for (_entity, (transform, selectable)) in world.query::<(&Transform, &Selectable)>().iter() {

        if selectable.is_selected {
            draw_selectable(transform);
        }

    }

}

fn draw_selection_state(world: &World) {

    let mut selection_query = world.query::<(&SelectionBox, &SelectionState)>();
    let (_selection_box_entity, (selection_box, selection_state)) = selection_query.iter().nth(0).expect("could not find the selection state entity? this is an error!");

    if selection_state.active {

        for points in selection_state.order_points.chunks(2) {

            let current = points[0];
            let next = points[1];

            draw_line_3d(current, next, GREEN);

        }

    }

}

fn draw_orderables_state(world: &World) {

    for (_entity, (transform, selectable, orderable)) in world.query::<(&Transform, &Selectable, &Orderable)>().iter() {

        if selectable.is_selected == false {
            continue
        }

        if let Some(target) = orderable.target {
            draw_line_3d(transform.position, target, GREEN);
        }

    }

}

fn draw_selection_box(world: &World) {

    let mut selection_query = world.query::<&SelectionBox>();
    let (_selection_box_entity, selection_box) = selection_query.iter().nth(0).expect("could not find the selection box entity? this is an error!");

    if selection_box.active {
        let selection_bounds = selection_box.bounds();
        draw_rectangle_lines(
            selection_bounds.x,
            selection_bounds.y,
            selection_bounds.w,
            selection_bounds.h,
            1.0,
            GREEN
        );
    }

}

fn draw_scene(game: &Game) {

    set_camera(&create_camera_from_game_camera(&game.camera));

    let plane_size = 8.0;
    let plane_slice_size = 1.0;
    let number_of_slices = plane_size as u32 / plane_slice_size as u32;

    // 3d elements
    draw_grid(number_of_slices, plane_slice_size, RED, GRAY);
    draw_characters(&game.engine, &game.world);
    draw_selectables(&game.world);
    draw_selection_state(&game.world);
    draw_orderables_state(&game.world);
    
    // 2d elements
    set_default_camera();
    draw_selection_box(&game.world);
    
}

fn draw_scene_debug(game: &mut Game) {
    
    // draw scene debug info
    game.debug_text.draw_text(format!("number of characters: {}", game.world.query::<&Character>().into_iter().count()), TextPosition::TopLeft, BLACK);

}

fn handle_update_selection_box(game: &mut Game, dt: f32) -> Option<Rect> {

    let mouse_position: Vec2 = mouse_position().into();
    let is_selection_active = is_mouse_button_down(MouseButton::Left);
    let is_selection_requested = is_mouse_button_released(MouseButton::Left);

    let (_selection_box_entity, selection_box) = game.world.query_mut::<(&mut SelectionBox)>().into_iter().nth(0).expect("could not find the selection box entity? this is an error!");
    if selection_box.active == false {
        selection_box.start = mouse_position;
    }

    selection_box.active = is_selection_active;
    selection_box.select = is_selection_requested;
    selection_box.end = mouse_position;

    if is_selection_requested {
        Some(selection_box.bounds())
    } else {
        None
    }

}

fn handle_update_selection_state(game: &mut Game, dt: f32) {

    let point_distance = 1.0;
    let mouse_position: Vec2 = mouse_position().into();
    let is_drawing_order = is_mouse_button_down(MouseButton::Right);
    let was_order_to_move_given = is_mouse_button_released(MouseButton::Right);

    let (_selection_box_entity, selection_state) = game.world.query_mut::<(&mut SelectionState)>().into_iter().nth(0).expect("could not find the selection box entity? this is an error!");
    selection_state.active = is_drawing_order;

    if is_drawing_order == false && was_order_to_move_given == false {
        selection_state.order_points.clear();
    }

    if is_drawing_order {
        if let Some(position) = try_pick_position_on_world_plane(&game.camera, mouse_position) {

            if selection_state.order_points.len() == 0 {

                selection_state.order_points.push(position);
                selection_state.order_points.push(position);
                
            } else {

                let last_point = *selection_state.order_points.get(selection_state.order_points.len() - 1).unwrap();
                let distance_to_last_point = position.distance(*selection_state.order_points.get(selection_state.order_points.len() - 1).unwrap());
                if distance_to_last_point >= point_distance {
                    selection_state.order_points.push(last_point);
                    selection_state.order_points.push(position);
                }

            }

        }
    }

}

fn handle_update_selection(game: &mut Game, dt: f32) {

    if let Some(selection_box) = handle_update_selection_box(game, dt) {

        for (_entity, (transform, selectable)) in game.world.query_mut::<(&Transform, &mut Selectable)>() {
            let projected_screen_position = game.camera.world_to_screen(transform.position);
            if is_point_inside_rect(&projected_screen_position, &selection_box) {
                selectable.is_selected = true;
            } else {
                selectable.is_selected = false;
            }
        }

    }

}

fn handle_update_orderables(game: &mut Game, dt: f32) {

    let was_order_to_move_given = is_mouse_button_released(MouseButton::Right);

    let (_selection_box_entity, selection_state) = game.world.query_mut::<(&SelectionState)>().into_iter().nth(0).expect("could not find the selection box entity? this is an error!");
    let current_target_order_positions = selection_state.order_points.clone();

    let orderable_query = game.world.query_mut::<(&mut Orderable, &Selectable)>();
    let number_of_entities_in_query = orderable_query.into_iter().filter(|s| s.1.1.is_selected).count();

    // update orderable targets of all selected entities
    for (idx, (_entity, (orderable, _selectable))) in game.world.query_mut::<(&mut Orderable, &Selectable)>().into_iter().filter(|s| s.1.1.is_selected).enumerate() {

        if was_order_to_move_given {

            let current_position_idx = ((current_target_order_positions.len() as f32 / number_of_entities_in_query as f32) * idx as f32) as usize;
            let current_position = current_target_order_positions[current_position_idx];
            orderable.target = Some(current_position);

        }

    }

    // update current orderable state
    for (_entity, (transform, orderable, _selectable)) in game.world.query_mut::<(&Transform, &mut Orderable, &Selectable)>().into_iter() {

        if let Some(target) = orderable.target {

            let reached_target_threshold = 0.1;
            let has_reached_target = (target - transform.position).length() < reached_target_threshold;
            
            if has_reached_target {
                orderable.target = None;
            }

        }

    }

}

fn handle_update_characters(game: &mut Game, dt: f32) {

    for (_entity, (transform, orderable, character)) in game.world.query_mut::<(&mut Transform, &Orderable, &Character)>() {

        if let Some(target) = orderable.target {

            let normalized_vector_to_target = (target - transform.position).normalize();
            let normalized_vector_in_the_plane = (target.xz() - transform.position.xz()).normalize();
            
            let target_forward_vector = normalized_vector_in_the_plane;
            let current_forward_vector = (transform.rotation * Vec3::Z).xz().normalize();
            let next_forward_vector = current_forward_vector.lerp(target_forward_vector, 0.125).normalize();

            // transform.velocity += -(vec3(next_forward_vector.x, 0.0, next_forward_vector.y) * character.movement_speed * dt);
            // let current_movement_speed = character.movement_speed.min((target - transform.position).length()).min(character.movement_speed / 2.0);

            transform.velocity = -(vec3(next_forward_vector.x, 0.0, next_forward_vector.y) * character.movement_speed);
            transform.rotation *= Quat::from_rotation_arc_2d_around_y(current_forward_vector, next_forward_vector);
            
        } else {
            transform.velocity = Vec3::ZERO;
        }

    }

}

fn handle_update_transforms(game: &mut Game, dt: f32) {

    for (_entity, transform) in game.world.query_mut::<&mut Transform>() {
        transform.position += transform.velocity * dt;
        // transform.velocity *= 0.965;
    }

}

fn update_scene(game: &mut Game, dt: f32) {

    handle_reset_world_state(game);
    handle_picking_world_position(game);

    handle_update_selection(game, dt);
    handle_update_selection_state(game, dt);
    
    handle_update_orderables(game, dt);
    handle_update_characters(game, dt);
    handle_update_transforms(game, dt);

}

fn update_camera(game: &mut Game, dt: f32) {

    handle_camera_input(&mut game.camera, dt);

}

fn create_character(world: &mut World, position: Vec3, script: Arc<AST>) {

    let character_movement_speed = 4.0;

    world.spawn(
        (
            Transform {
                position: position,
                rotation: Quat::IDENTITY,
                velocity: Vec3::ZERO
            },
            Selectable::new(),
            Orderable::new(),
            Character::new(character_movement_speed, script)
        )
    );

}

fn register_types_for_rhai(engine: &mut Engine) {

    engine
        .register_fn("to_string", |v: &mut Vec2| v.to_string())
        .register_fn("to_debug", |v: &mut Vec2| format!("{v:?}"));

    engine
        .register_fn("to_string", |v: &mut Vec3| v.to_string())
        .register_fn("to_debug", |v: &mut Vec3| format!("{v:?}"));

    engine
        .register_fn("to_string", |v: &mut Quat| v.to_string())
        .register_fn("to_debug", |v: &mut Quat| format!("{v:?}"));

    engine.register_type::<Character>()
        .register_get("movement_speed", |c: &mut Character| c.movement_speed);

    engine.register_type::<Transform>()
        .register_get("position", |t: &mut Transform| t.position)
        .register_get("rotation", |t: &mut Transform| t.rotation)
        .register_get("velocity", |t: &mut Transform| t.velocity);

    engine.register_type::<Vec2>()

        .register_fn("vec2", Vec2::new)
        .register_fn("vec2_zero", || Vec2::ZERO)
    
        .register_fn("dot", Vec2::dot)
        .register_fn("perp_dot", Vec2::perp_dot)
        .register_fn("length", Vec2::length)
        .register_fn("length_squared", Vec2::length_squared)
        .register_fn("round", Vec2::round)
        .register_fn("ceil", Vec2::ceil)
        .register_fn("floor", Vec2::floor)
        .register_fn("lerp", Vec2::lerp)
        .register_fn("from_angle", Vec2::from_angle)
        .register_fn("angle_between", Vec2::angle_between)
        .register_fn("normalize", Vec2::normalize)
        .register_fn("normalize_or_zero", Vec2::normalize_or_zero)

        .register_get_set("x", |v: &mut Vec2| v.x, |v: &mut Vec2, n: f32| v.x = n)
        .register_get_set("y", |v: &mut Vec2| v.y, |v: &mut Vec2, n: f32| v.y = n)

        // unary operators
        .register_fn("-", |a: Vec2| -a)

        // binary operators
        .register_fn("+", |a: Vec2, b: Vec2| a + b)
        .register_fn("-", |a: Vec2, b: Vec2| a - b)
        .register_fn("*", |a: Vec2, b: f32| a * b)
        .register_fn("/", |a: Vec2, b: f32| a / b);

    engine.register_type::<Vec3>()

        .register_fn("vec3", Vec3::new)
        .register_fn("vec3_zero", || Vec3::ZERO)

        .register_fn("dot", Vec3::dot)
        .register_fn("length", Vec3::length)
        .register_fn("length_squared", Vec3::length_squared)
        .register_fn("round", Vec3::round)
        .register_fn("ceil", Vec3::ceil)
        .register_fn("floor", Vec3::floor)
        .register_fn("lerp", Vec3::lerp)
        .register_fn("normalize", Vec3::normalize)
        .register_fn("normalize_or_zero", Vec3::normalize_or_zero)

        .register_get_set("x", |v: &mut Vec3| v.x, |v: &mut Vec3, n: f32| v.x = n)
        .register_get_set("y", |v: &mut Vec3| v.y, |v: &mut Vec3, n: f32| v.y = n)
        .register_get_set("z", |v: &mut Vec3| v.z, |v: &mut Vec3, n: f32| v.z = n)

        // unary operators
        .register_fn("-", |a: Vec3| -a)

        // binary operators
        .register_fn("+", |a: Vec3, b: Vec3| a + b)
        .register_fn("-", |a: Vec3, b: Vec3| a - b)
        .register_fn("*", |a: Vec3, b: f32| a * b)
        .register_fn("/", |a: Vec3, b: f32| a / b);

    engine.register_type::<Quat>()

        .register_fn("quat", Quat::from_xyzw)
        .register_fn("quat_identity", || Quat::IDENTITY)

        .register_fn("from_rotation_x", Quat::from_rotation_x)
        .register_fn("from_rotation_y", Quat::from_rotation_y)
        .register_fn("from_rotation_z", Quat::from_rotation_z)
        .register_fn("conjugate", Quat::conjugate)
        .register_fn("inverse", Quat::inverse)
        .register_fn("slerp", Quat::slerp)
        .register_fn("lerp", Quat::lerp)

        .register_get_set("x", |v: &mut Quat| v.x, |v: &mut Quat, n: f32| v.x = n)
        .register_get_set("y", |v: &mut Quat| v.y, |v: &mut Quat, n: f32| v.y = n)
        .register_get_set("z", |v: &mut Quat| v.z, |v: &mut Quat, n: f32| v.z = n)
        .register_get_set("w", |v: &mut Quat| v.w, |v: &mut Quat, n: f32| v.w = n)

        // unary operators
        .register_fn("-", |a: Quat| -a)

        // binary operators
        .register_fn("*", |a: Quat, b: Vec3| a * b)
        .register_fn("*", |a: Quat, b: Quat| a * b);

    // register some drawing functions
    engine
        .register_fn(
            "draw_with_transformation",
            |context: NativeCallContext, position: Vec3, rotation: Quat, callback: rhai::FnPtr| {
                draw_with_transformation(position, rotation, || { let _ = callback.call_within_context::<()>(&context, ()); })
            }
        )
        .register_fn("draw_part", draw_part);

    // register some macroquad functions
    engine
        .register_fn("get_frame_time", get_frame_time)
        .register_fn("get_time", || get_time() as f32);
    
    // register some math functions
    engine
        .register_fn("normalize", normalize);

}

fn create_character_script(game: &mut Game) {

    // load our character script, for now just one, but at least we have one :D
    let script_ast = game.engine.compile_file("scripts/character.rhai".into());
    
    if let Result::Ok(compiled_script) = script_ast {

        // create a Scope with all top-level constants
        let scope: Scope = compiled_script.iter_literal_variables(true, false).collect();
        
        // re-optimize the AST to propagate constants into functions
        let compiled_script = game.engine.optimize_ast(&scope, compiled_script, OptimizationLevel::Simple);

        game.character_script = Some(Arc::new(compiled_script));

        println!("successfully compiled new character script, using new version!");

    } else if let Result::Err(compilation_error) = script_ast {

        if let Some(_script) = &game.character_script {
            println!("failed to compile character script, continuing to use the old version, error: {}", compilation_error.to_string());
        } else {
            println!("failed to compile character script, no old version available, fell back to empty default!");
            game.character_script = Some(game.fallback_script.clone());
        }
        
    }

}

fn create_selection_box(world: &mut World) {
    world.spawn((SelectionBox::new(), SelectionState::new()));
}

fn create_default_scene(game: &mut Game) {

    let world_center = Vec3::ZERO;

    game.camera.position = world_center + vec3(0.0, 4.0, 8.0);
    game.camera.target = world_center;

    game.camera.parameters.movement_speed = 4.0;

    // clear game world first
    game.world.clear();

    // add an entity that is the current selection box
    create_selection_box(&mut game.world);

    // load our character script, for now just one, but at least we have one :D
    create_character_script(game);

    // add a bunch of characters around 0, 0, 0

    let area_size = 1;

    for x in -area_size..area_size {

        for y in -area_size..area_size {

            if x % 2 == 0 || y % 2 == 0 {
                continue
            }

            let spawn_x = x as f32;
            let spawn_z = y as f32;

            create_character(&mut game.world, vec3(spawn_x, 0.0, spawn_z), game.character_script.clone().unwrap());

        }

    }

}

fn try_pick_position_on_world_plane(camera: &GameCamera, screen_position: Vec2) -> Option<Vec3> {

    let ray_origin = camera.position;
    let ray_direction = camera.screen_to_world_ray(screen_position);

    intersect_ray_with_plane(ray_origin, ray_direction, -WORLD_UP, Vec3::ZERO)

}

fn handle_picking_world_position(game: &mut Game) {

    game.debug_text.draw_text(format!("camera position: {}", game.camera.position), TextPosition::TopLeft, BLACK);

    let mouse_position: Vec2 = mouse_position().into();

    if let Some(intersection) = try_pick_position_on_world_plane(&game.camera, mouse_position) {
        game.debug_text.draw_text(format!("world position under mouse: {}", intersection), TextPosition::TopLeft, BLACK);
    }

}

fn handle_reset_world_state(game: &mut Game) {

    let should_reset_simulation = is_key_pressed(KeyCode::R);

    if should_reset_simulation {
        create_default_scene(game);
    }

    game.debug_text.draw_text("press r to reset the world state", TextPosition::TopLeft, BLACK);

}

#[macroquad::main("procedural-animation")]
async fn main() {

    let mut game = Game::new();
    create_default_scene(&mut game);

    // register types and functions for rhai, only needs doing once
    register_types_for_rhai(&mut game.engine);

    loop {

        let dt = get_frame_time();
        clear_background(WHITE);

        // frame stats
        draw_debug_text(&mut game);

        // update
        update_scene(&mut game, dt);

        // draw
        draw_scene(&game);

        draw_scene_debug(&mut game);

        // update camera
        update_camera(&mut game, dt);
        
        game.debug_text.new_frame();
        next_frame().await;

    }
    
}
