use macroquad::prelude::*;

pub fn load_texture_from_image(image: &Image) -> Texture2D {
    Texture2D::from_rgba8(image.width, image.height, &image.bytes)
}