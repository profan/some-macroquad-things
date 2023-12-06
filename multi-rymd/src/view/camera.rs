use macroquad::prelude::*;
use utility::screen_dimensions;

struct GameCameraParameters {
    min_zoom: f32,
    max_zoom: f32,
    move_speed: f32,
    zoom_speed: f32
}

pub struct GameCamera {
    size: Vec2,
    camera_zoom: f32,
    camera: Camera2D,
    last_mouse_position: Vec2,
    parameters: GameCameraParameters
}

impl GameCamera {

    pub fn new() -> GameCamera {

        let size = screen_dimensions();
        let camera = Camera2D::from_display_rect(
            Rect { x: 0.0, y: 0.0, w: size.x, h: size.y }
        );

        let last_mouse_position: Vec2 = mouse_position().into();

        let parameters = GameCameraParameters {
            min_zoom: 0.5,
            max_zoom: 4.0,
            move_speed: 256.0,
            zoom_speed: 10.0
        };

        GameCamera {
            size: size,
            camera: camera,
            camera_zoom: 1.0,
            last_mouse_position: mouse_position().into(),
            parameters
        }

    }

    pub fn world_position(&self) -> Vec2 {
        self.camera.target
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

    pub fn world_to_screen_scale(&self, v: Vec2) -> Vec2 {
        v / self.camera_zoom
    }

    pub fn screen_to_world_scale(&self, v: Vec2) -> Vec2 {
        v * self.camera_zoom
    }

    pub fn world_to_screen_scale_v(&self, v: f32) -> f32 {
        v / self.camera_zoom
    }

    pub fn screen_to_world_scale_v(&self, v: f32) -> f32 {
        v * self.camera_zoom
    }

    pub fn world_to_screen_rect(&self, mut rect: Rect) -> Rect {
        let screen_position = self.world_to_screen(vec2(rect.x, rect.y));
        let screen_scale = 1.0 / self.camera_zoom;
        rect.scale(screen_scale, screen_scale);
        rect.x = screen_position.x;
        rect.y = screen_position.y;
        rect
    }

    pub fn screen_to_world_rect(&self, mut rect: Rect) -> Rect {
        let world_position = self.screen_to_world(vec2(rect.x, rect.y));
        let world_scale = self.camera_zoom;
        rect.scale(world_scale, world_scale);
        rect.x = world_position.x;
        rect.y = world_position.y;
        rect
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

    let camera_speed = active.parameters.move_speed * active.camera_zoom;

    let is_up_pressed = is_key_down(KeyCode::Up);
    let is_down_pressed = is_key_down(KeyCode::Down);
    let is_left_pressed = is_key_down(KeyCode::Left);
    let is_right_pressed = is_key_down(KeyCode::Right);

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

    let min_zoom = active.parameters.min_zoom;
    let max_zoom = active.parameters.max_zoom;

    let new_zoom = (active.camera_zoom - mouse_wheel_delta_y * active.parameters.zoom_speed * dt).clamp(min_zoom, max_zoom);
    let new_size = active.size * new_zoom;

    let new_target = if new_zoom < active.camera_zoom {
        let vector_to_mouse_world_position = active.mouse_world_position() - active.world_position();
        let current_vector_to_world_position = active.world_position() + (vector_to_mouse_world_position / 4.0) / active.camera_zoom;
        current_vector_to_world_position
    } else {
        active.camera.target
    };

    let new_camera = Camera2D::from_display_rect(
        Rect {
            x: new_target.x - (new_size.x / 2.0),
            y: new_target.y - (new_size.y / 2.0),
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