use macroquad::{math::Vec3, prelude::{Camera3D, Vec2, Mat4, Camera, vec2, vec3}, window::{screen_width, screen_height}};

pub struct GameCameraParameters {

    /// movement speed in world units per second
    pub movement_speed: f32,

    /// rotation speed in radians per second
    pub rotation_speed: f32,

    /// zoom speed in world units per second?
    pub zoom_speed: f32

}

pub struct GameCamera {

    pub parameters: GameCameraParameters,

    pub position: Vec3,
    pub target: Vec3,
    pub up: Vec3,

}

impl GameCamera {

    /// Creates a new camera, uses Y+ as up by default, will be looking in the -Z direction.
    pub fn new() -> GameCamera {
        GameCamera {
            parameters: GameCameraParameters {
                movement_speed: 1.0,
                rotation_speed: 1.0,
                zoom_speed: 1.0
            },
            position: Vec3::ZERO,
            target: Vec3::ZERO + Vec3::NEG_Z,
            up: Vec3::Y
        }
    }

    pub fn forward(&self) -> Vec3 {
        (self.target - self.position).normalize()
    }

    pub fn left(&self) -> Vec3 {
        self.up.cross(self.forward()).normalize()
    }

    /// Returns a normalized vector that points towards the projected world position.
    pub fn screen_to_world_ray(&self, screen_pos: Vec2) -> Vec3 {
        (self.screen_to_world(screen_pos, 1000.0) - self.screen_to_world(screen_pos, 0.0)).normalize()
    }

    /// Returns the projected world position given a screen position and a depth.
    pub fn screen_to_world(&self, screen_pos: Vec2, depth: f32) -> Vec3 {

        let projection_matrix = self.projection_matrix();
        let viewport_size = vec2(screen_width(), screen_height());

        let point_on_near_plane = vec2(
            (screen_pos.x / viewport_size.x) * 2.0 - 1.0,
            1.0 - (screen_pos.y / viewport_size.y) * 2.0
        );

        let point_on_near_plane_to_transform = vec3(
            point_on_near_plane.x,
            point_on_near_plane.y,
            0.0
        );

        let projected_point_on_near_plane = projection_matrix.inverse().project_point3(point_on_near_plane_to_transform);
        let projected_point_in_world_space = projected_point_on_near_plane + (projected_point_on_near_plane - self.position).normalize_or_zero() * depth;

        projected_point_in_world_space
        
    }

    /// Returns the projected screen position of a given world position.
    pub fn world_to_screen(&self, world_pos: Vec3) -> Vec2 {

        let projection_matrix = self.projection_matrix();
        let screen_pos = projection_matrix.project_point3(world_pos);

        vec2(
            (screen_pos.x / 2. + 0.5) * screen_width(),
            (0.5 - screen_pos.y / 2.) * screen_height(),
        )

    }

    pub fn projection_matrix(&self) -> Mat4 {
        let camera = create_camera_from_game_camera(self);
        camera.matrix()
    }

}

pub fn create_camera(position: Vec3, up: Vec3, target: Vec3) -> Camera3D {
    Camera3D {
        position: position,
        up: up,
        target: target,
        ..Default::default()
    }
}

pub fn create_camera_from_game_camera(camera: &GameCamera) -> Camera3D {
    create_camera(camera.position, camera.up, camera.target)
}