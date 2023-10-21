use macroquad::prelude::*;

pub fn load_texture_from_image(image: &Image) -> Texture2D {
    Texture2D::from_rgba8(image.width, image.height, &image.bytes)
}

pub fn set_texture_filter(texture: Texture2D, filter_mode: FilterMode) {
    texture.set_filter(filter_mode);
}