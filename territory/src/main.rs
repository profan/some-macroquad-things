use macroquad::prelude::*;

const TILE_SIZE: i32 = 32;

/// Rasterizes a given tile for marching squares where i is in \[0, 16\], should be called with a render target active.
fn rasterize_tile(offset: Vec2, i: i32, color: Color) {

    let line_thickness = 1.0;

    let mut draw_line = |start_x: i32, start_y: i32, end_x: i32, end_y: i32, v: u8| {
        draw_line(
            offset.x + start_x as f32,
            offset.y + start_y as f32,
            offset.x + end_x as f32,
            offset.y + end_y as f32,
            line_thickness,
            color
        );
    };

    match i {
        0 => {},
        1 => {

            let start = ivec2(0, TILE_SIZE / 2);
            let end = ivec2(TILE_SIZE / 2, TILE_SIZE);
            draw_line(start.x, start.y, end.x, end.y, 1);

            // fill bottom left
            
        },
        2 => {
            
            let start = ivec2(TILE_SIZE / 2, TILE_SIZE);
            let end = ivec2(TILE_SIZE, TILE_SIZE / 2);
            draw_line(start.x, start.y, end.x, end.y, 1);
            
            // fill bottom right

        },
        3 => {

            let start = ivec2(0, TILE_SIZE / 2);
            let end = ivec2(TILE_SIZE, TILE_SIZE / 2);
            draw_line(start.x, start.y, end.x, end.y, 1);

            // fill below

        },
        4 => {

            let start = ivec2(TILE_SIZE / 2, 0);
            let end = ivec2(TILE_SIZE, TILE_SIZE / 2);
            draw_line(start.x, start.y, end.x, end.y, 1);

            // fill top right

        },
        5 => {

            let start_top_left = ivec2(0, TILE_SIZE / 2);
            let end_top_left = ivec2(TILE_SIZE / 2, 0);

            draw_line(start_top_left.x, start_top_left.y, end_top_left.x, end_top_left.y, 1);

            let start_bottom_right = ivec2(TILE_SIZE / 2, TILE_SIZE);
            let end_bottom_right = ivec2(TILE_SIZE, TILE_SIZE / 2);

            draw_line(start_bottom_right.x, start_bottom_right.y, end_bottom_right.x, end_bottom_right.y, 1);

            // fill middle

        },
        6 => {

            let start = ivec2(TILE_SIZE / 2, 0);
            let end = ivec2(TILE_SIZE / 2, TILE_SIZE);
            draw_line(start.x, start.y, end.x, end.y, 1);

            // fill right
            
        },
        7 => {

            let start = ivec2(0, TILE_SIZE / 2);
            let end = ivec2(TILE_SIZE / 2, 0);
            draw_line(start.x, start.y, end.x, end.y, 1);

            // fill bottom right

        },
        8 => {

            let start = ivec2(0, TILE_SIZE / 2);
            let end = ivec2(TILE_SIZE / 2, 0);
            draw_line(start.x, start.y, end.x, end.y, 1);

            // fill top left

        },
        9 => {


            let start = ivec2(TILE_SIZE / 2, 0);
            let end = ivec2(TILE_SIZE / 2, TILE_SIZE);
            draw_line(start.x, start.y, end.x, end.y, 1);

            // fill left

        },
        10 => {

            let start_bottom_left = ivec2(0, TILE_SIZE / 2);
            let end_bottom_left = ivec2(TILE_SIZE / 2, TILE_SIZE);

            draw_line(start_bottom_left.x, start_bottom_left.y, end_bottom_left.x, end_bottom_left.y, 1);

            let start_top_right = ivec2(TILE_SIZE / 2, 0);
            let end_top_right = ivec2(TILE_SIZE, TILE_SIZE / 2);

            draw_line(start_top_right.x, start_top_right.y, end_top_right.x, end_top_right.y, 1);

            // fill middle

        },
        11 => {

            let start = ivec2(TILE_SIZE / 2, 0);
            let end = ivec2(TILE_SIZE, TILE_SIZE / 2);
            draw_line(start.x, start.y, end.x, end.y, 1);

            // fill bottom left
            
        },
        12 => {

            let start = ivec2(0, TILE_SIZE / 2);
            let end = ivec2(TILE_SIZE, TILE_SIZE / 2);
            draw_line(start.x, start.y, end.x, end.y, 1);

            // fill above

        },
        13 => {

            let start = ivec2(TILE_SIZE / 2, TILE_SIZE);
            let end = ivec2(TILE_SIZE, TILE_SIZE / 2);
            draw_line(start.x, start.y, end.x, end.y, 1);

            // fill top left

        },
        14 => {
            
            let start = ivec2(0, TILE_SIZE / 2);
            let end = ivec2(TILE_SIZE / 2, TILE_SIZE);
            draw_line(start.x, start.y, end.x, end.y, 1);

            // fill top right

        },
        15 => {
            // fill maybe?
        },
        _ => {}
    }

}

/// Rasterizes all the tiles for marching squares into a texture atlas (single row).
fn rasterize_tile_atlas(color: Color) -> RenderTarget {

    // 16 is the number of tiles!
    let texture_width = TILE_SIZE * 16;
    let texture_height = TILE_SIZE;

    let render_target = render_target(texture_width as u32, texture_height as u32);
    render_target.texture.set_filter(FilterMode::Nearest);

    let render_target_camera = create_render_target_camera(render_target);

    set_camera(&render_target_camera);

    for i in 0..16 {
        let current_offset = vec2((TILE_SIZE * i) as f32, 0.0);
        rasterize_tile(current_offset, i, color);
    }

    render_target

}

/// Creates a camera given a render target.
fn create_render_target_camera(render_target: RenderTarget) -> Camera2D {

    let size = vec2(render_target.texture.width(), render_target.texture.height());
    let mut render_target_camera = Camera2D::from_display_rect(Rect { x: 0.0, y: 0.0, w: size.x, h: size.y});
    render_target_camera.render_target = Some(render_target);

    render_target_camera

}

struct Heightmap {
    data: Vec<u8>,
    size: UVec2
}

impl Heightmap {

    pub fn new(w: u32, h: u32) -> Heightmap {
        Heightmap {
            data: vec![0; (w * h) as usize],
            size: uvec2(w, h)
        }
    }

    pub fn get(&self, x: i32, y: i32) -> u8 {
        let idx = x + y * self.size.x as i32;
        if idx < 0 || idx >= self.data.len() as i32 { 0 } else { self.data[idx as usize] }
    }

    pub fn set(&mut self, x: u32, y: u32, v: u8) {
        let idx = (x + y * self.size.x) as usize;
        self.data[idx] = v;
    }

    pub fn width(&self) -> u32 {
        self.size.x
    }

    pub fn height(&self) -> u32 {
        self.size.y
    }

}

fn create_height_field_buffer_texture(map: &Heightmap) -> Texture2D {

    let bytes = map.data.as_slice();

    let miniquad_texture = unsafe {
        miniquad::Texture::from_data_and_format(
            get_internal_gl().quad_context,
            bytes,
            miniquad::TextureParams {
                format: miniquad::TextureFormat::Alpha,
                wrap: miniquad::TextureWrap::Clamp,
                filter: miniquad::FilterMode::Nearest,
                width: map.size.x,
                height: map.size.y
            }
        )
    };

    Texture2D::from_miniquad_texture(miniquad_texture)

}

fn create_height_field(w: u32, h: u32) -> Heightmap {

    let mut new_heightmap = Heightmap::new(w, h);

    let w = w as i32;
    let h = h as i32;

    for x in 0..w {
        for y in 0..h {
            if x <= 2 || x >= w - 2 || y <= 1 || y >= h - 3 {
                //new_heightmap.set(x as u32, y as u32, 0);
                continue;
            }
            new_heightmap.set(x as u32, y as u32, 3);
        }
    }

    new_heightmap

}

fn height_map_position_to_index(map: &Heightmap, x: u32, y: u32) -> u8 {

    let x = x as i32;
    let y = y as i32;

    let isovalue = 1;

    let top_left_bit = ((map.get(x, y) > isovalue) as u8) << 0;
    let top_right_bit = ((map.get(x + 1, y) > isovalue) as u8) << 1;
    let bottom_right_bit = ((map.get(x + 1, y + 1) > isovalue) as u8) << 2;
    let bottom_left_bit = ((map.get(x, y + 1) > isovalue) as u8) << 3;

    let final_index = top_left_bit | top_right_bit | bottom_left_bit | bottom_right_bit;

    final_index

}

fn draw_height_field(map: &Heightmap, atlas: Texture2D, render_debug_text: bool) {

    for x in 0..map.width() {
        for y in 0..map.height() {

            let idx = height_map_position_to_index(map, x, y);
            let offset_x = (TILE_SIZE * idx as i32) as f32;

            let dest_size = vec2(
                TILE_SIZE as f32,
                TILE_SIZE as f32
            );

            let source_rect = Rect {
                x: offset_x,
                y: 0.0,
                w: TILE_SIZE as f32,
                h: TILE_SIZE as f32
            };

            if render_debug_text {

                let text_x = (x * TILE_SIZE as u32) as f32 + (TILE_SIZE / 2) as f32;
                let text_y = (y * TILE_SIZE as u32) as f32 + (TILE_SIZE / 2) as f32;
                let v = map.get(x as i32, y as i32);

                draw_text(
                    v.to_string().as_str(),
                    text_x, text_y + (TILE_SIZE as f32) / 2.0,
                    12.0,
                    if v > 0 { GREEN } else { RED }
                );

                draw_text(
                    idx.to_string().as_str(),
                    text_x, text_y,
                    12.0,
                    BLACK
                );

            }

            draw_texture_ex(
                atlas,
                (x * TILE_SIZE as u32) as f32,
                (y * TILE_SIZE as u32) as f32,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(dest_size),
                    source: Some(source_rect),
                    ..Default::default()
                }
            )

        }
    }

}

#[macroquad::main("territory")]
async fn main() {

    // step 1. rasterize the 16 tiles for marching squares into a buffer.
    // step 2. upload the texture with the rasterized tile data.
    // step 3. create a buffer that represents the map data.
    // step 4. upload a texture with the map data.
    // step 5. ???

    let mut has_rasterized_tile_atlas = false;
    let mut rasterized_tile_atlas: Option<RenderTarget> = None;

    let height_field = create_height_field(24, 18);
    let height_field_texture = create_height_field_buffer_texture(&height_field);
    let mut render_debug_text = false;
    
    loop {

        // re-rasterize atlas if necessary

        if has_rasterized_tile_atlas == false {

            if let Some(atlas) = rasterized_tile_atlas {
                atlas.delete();
            }

            rasterized_tile_atlas = Some(rasterize_tile_atlas(BLACK));
            has_rasterized_tile_atlas = true;

        }

        set_default_camera();
        clear_background(WHITE);

        draw_height_field(&height_field, rasterized_tile_atlas.unwrap().texture, render_debug_text);

        if is_key_pressed(KeyCode::Escape) {
            break;
        }

        next_frame().await;

    }

}