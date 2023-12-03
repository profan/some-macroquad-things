use macroquad::prelude::*;
use utility::screen_dimensions;

pub struct GameCamera {
    size: Vec2,
    camera_zoom: f32,
    camera: Camera2D,
    last_mouse_position: Vec2
}

impl GameCamera {

    pub fn new() -> GameCamera {

        let size = screen_dimensions();
        let camera = Camera2D::from_display_rect(
            Rect { x: 0.0, y: 0.0, w: size.x, h: size.y }
        );

        let last_mouse_position: Vec2 = mouse_position().into();

        GameCamera {
            size: size,
            camera: camera,
            camera_zoom: 1.0,
            last_mouse_position: mouse_position().into()
        }

    }

    pub fn mouse_screen_position(&self) -> Vec2 {
        mouse_position().into()
    }

    pub fn mouse_world_position(&self) -> Vec2 {
        self.screen_to_world(self.mouse_screen_position())
    }

    pub fn smooth_move_camera_to_position(&self, world_position: Vec2) {

    }

    pub fn move_camera_to_position(&mut self, world_position: Vec2) {
        self.camera.target = world_position
    }

    pub fn screen_to_world(&self, screen_position: Vec2) -> Vec2 {
        self.camera.screen_to_world(screen_position)
    }

    pub fn world_to_screen(&self, world_position: Vec2) -> Vec2 {
        self.camera.world_to_screen(world_position)
    }

    pub fn push(&self) {
        push_camera_state();
        set_camera(&self.camera);
    }

    pub fn pop(&self) {
        pop_camera_state();
    }

    pub fn tick(&mut self, dt: f32) {

        self.size = screen_dimensions();
        handle_camera_input(self, self.last_mouse_position, dt);
        self.last_mouse_position = mouse_position().into();
        
    }

}

fn handle_camera_input(active: &mut GameCamera, last_mouse_position: Vec2, dt: f32) -> bool {

    handle_camera_movement(active, dt);
    let zoom_changed = handle_camera_zoom(active, dt);
    handle_camera_panning(active, last_mouse_position, dt);

    zoom_changed

}

fn handle_camera_movement(active: &mut GameCamera, dt: f32) {

    let camera_speed = 256.0 * active.camera_zoom;

    let is_up_pressed = is_key_down(KeyCode::Up) || is_key_down(KeyCode::W);
    let is_down_pressed = is_key_down(KeyCode::Down) || is_key_down(KeyCode::S);
    let is_left_pressed = is_key_down(KeyCode::Left) || is_key_down(KeyCode::A);
    let is_right_pressed = is_key_down(KeyCode::Right) || is_key_down(KeyCode::D);

    let mut camera_delta = Vec2::ZERO;

    if is_up_pressed {
        camera_delta += vec2(0.0, -1.0);
    }

    if is_down_pressed {
        camera_delta += vec2(0.0, 1.0);
    }

    if is_left_pressed {
        camera_delta += vec2(-1.0, 0.0);
    }

    if is_right_pressed {
        camera_delta += vec2(1.0, 0.0);
    }
    
    active.camera.target += camera_delta * camera_speed * dt;

}

fn handle_camera_zoom(active: &mut GameCamera, dt: f32) -> bool {

    let (mouse_wheel_delta_x, mouse_wheel_delta_y) = mouse_wheel();

    let min_zoom = 0.5;
    let max_zoom = 4.0;

    let new_zoom = (active.camera_zoom - mouse_wheel_delta_y * dt).clamp(min_zoom, max_zoom);
    let new_size = active.size * new_zoom;

    let new_camera = Camera2D::from_display_rect(
        Rect {
            x: active.camera.target.x - (new_size.x / 2.0),
            y: active.camera.target.y - (new_size.y / 2.0),
            w: new_size.x,
            h: new_size.y
        }
    );

    active.camera_zoom = new_zoom;
    active.camera = new_camera;

    // so we can do things on change
    mouse_wheel_delta_y != 0.0

}

fn handle_camera_panning(active: &mut GameCamera, last_mouse_position: Vec2, dt: f32) {

    let is_middle_mouse_down = is_mouse_button_down(MouseButton::Middle);

    if is_middle_mouse_down {
        let mouse_position_v: Vec2 = mouse_position().into();
        let mouse_position_delta: Vec2 = last_mouse_position - mouse_position_v;
        active.camera.target += mouse_position_delta * active.camera_zoom;
    }

}