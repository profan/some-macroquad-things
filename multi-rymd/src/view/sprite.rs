use std::f32::consts::PI;
use macroquad::prelude::*;
use utility::{AsPerpendicular, AsVector, load_texture_from_image, set_texture_filter};

fn slice_sprite(texture: Texture2D, num_pieces: i32, w: u16, h: u16) -> Vec<Image> {
    assert!(num_pieces > 0);

    let image = texture.get_texture_data();
    let center = vec2(w as f32 / 2.0, h as f32 / 2.0);

    let (slice_w, slice_h) = (w, h); // FIXME: this is wrong but the simplest way for now :)
    let slice_size = (PI * 2.0) / num_pieces as f32;
    let mut slices = Vec::new();
    
    for i in 0..num_pieces {
        
        fn are_clockwise(v1: Vec2, v2: Vec2) -> bool {
            v1.perpendicular().dot(v2) > 0.0
        }

        let mut new_image = Image::gen_image_color(slice_w, slice_h, WHITE);

        let start_angle = i as f32 * slice_size;
        let end_angle = start_angle + slice_size;

        for x in 0..w {
            for y in 0..h {

                let p = vec2(x as f32, y as f32);
                let arm_normal = (p - center).normalize();
                let is_clockwise_from_start = !are_clockwise(start_angle.as_vector(), arm_normal);
                let is_counter_clockwise_from_end = are_clockwise(end_angle.as_vector(), arm_normal);
                let is_in_slice = is_clockwise_from_start && is_counter_clockwise_from_end;
                // let is_in_radius = p.distance_to(center) < w as f32 / 2.0;

                let colour_value = if is_in_slice { image.get_pixel(x as u32, y as u32) } else { Color::new(0.0, 0.0, 0.0, 0.0) };
                new_image.set_pixel(x as u32, y as u32, colour_value);

            }
        }

        slices.push(new_image);

    }

    slices
    
}

fn upload_sprite_slices(slices: Vec<Image>) -> Vec<Texture2D> {
    slices.iter().map(|image| {
        let texture = load_texture_from_image(image);
        set_texture_filter(texture, FilterMode::Nearest);
        texture
    }).collect()
}