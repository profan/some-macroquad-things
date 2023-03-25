use macroquad::{math::Vec3, prelude::Camera3D};

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