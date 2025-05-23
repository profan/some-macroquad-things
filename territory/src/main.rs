use macroquad::{miniquad::MipmapFilterMode, prelude::*, rand::gen_range};

use noise::{
    utils::{PlaneMapBuilder, NoiseMapBuilder}, Add, Perlin, ScaleBias, Turbulence
};

use utility::{screen_dimensions, AdjustHue, Camera2DExt, DebugText, TextPosition, WithAlpha};

const TILE_PADDING: i32 = 2;
const REAL_TILE_SIZE: i32 = 32;
const TILE_SIZE: i32 = 32 + TILE_PADDING / 2;

/// Rasterizes a given tile for marching squares where i is in \[0, 16\], should be called with a render target active.
fn rasterize_tile(offset: Vec2, i: i32, line_color: Color, fill_color: Color, line_thickness: f32, should_fill: bool) {

    let draw_line = |start_x: i32, start_y: i32, end_x: i32, end_y: i32| {
        draw_line(
            offset.x + start_x as f32,
            offset.y + start_y as f32,
            offset.x + end_x as f32,
            offset.y + end_y as f32,
            line_thickness,
            line_color
        );
    };

    let draw_triangles = |triangles: &[[IVec2; 3]]| {

        if should_fill == false {
            return;
        }

        for &[a, b, c] in triangles {
            draw_triangle(
                offset + a.as_vec2(),
                offset + b.as_vec2(),
                offset + c.as_vec2(),
                fill_color
            );
        }

    };

    match i {
        0 => {},
        1 => {

            let start = ivec2(0, TILE_SIZE / 2);
            let end = ivec2(TILE_SIZE / 2, TILE_SIZE);

            // fill bottom left
            draw_triangles(
                &[
                    [start, end, ivec2(0, TILE_SIZE)]
                ]
            );

            // draw border
            draw_line(start.x, start.y, end.x, end.y);
            
        },
        2 => {
            
            let start = ivec2(TILE_SIZE / 2, TILE_SIZE);
            let end = ivec2(TILE_SIZE, TILE_SIZE / 2);

            // fill bottom right
            draw_triangles(
                &[
                    [start, end, ivec2(TILE_SIZE, TILE_SIZE)]
                ]
            );

            // draw border
            draw_line(start.x, start.y, end.x, end.y);
            
        },
        3 => {

            let start = ivec2(0, TILE_SIZE / 2);
            let end = ivec2(TILE_SIZE, TILE_SIZE / 2);

            // fill below
            draw_triangles(
                &[
                    [start, end, ivec2(TILE_SIZE, TILE_SIZE)],
                    [ivec2(TILE_SIZE, TILE_SIZE), ivec2(0, TILE_SIZE), start],
                ]
            );

            // draw border
            draw_line(start.x, start.y, end.x, end.y);

        },
        4 => {

            let start = ivec2(TILE_SIZE / 2, 0);
            let end = ivec2(TILE_SIZE, TILE_SIZE / 2);

            // fill top right
            draw_triangles(
                &[
                    [start, ivec2(TILE_SIZE, 0), end]
                ]
            );

            // draw border
            draw_line(start.x, start.y, end.x, end.y);

        },
        5 => {

            let start_top_left = ivec2(0, TILE_SIZE / 2);
            let end_top_left = ivec2(TILE_SIZE / 2, 0);

            let start_bottom_right = ivec2(TILE_SIZE / 2, TILE_SIZE);
            let end_bottom_right = ivec2(TILE_SIZE, TILE_SIZE / 2);  

            // fill middle
            draw_triangles(
                &[
                    [end_top_left, ivec2(TILE_SIZE, 0), end_bottom_right],
                    [end_bottom_right, start_bottom_right, start_top_left],
                    [start_top_left, end_top_left, end_bottom_right],
                    [ivec2(0, TILE_SIZE), start_top_left, start_bottom_right]
                ]
            );

            // draw border
            draw_line(start_top_left.x, start_top_left.y, end_top_left.x, end_top_left.y);
            draw_line(start_bottom_right.x, start_bottom_right.y, end_bottom_right.x, end_bottom_right.y);

        },
        6 => {

            let start = ivec2(TILE_SIZE / 2, 0);
            let end = ivec2(TILE_SIZE / 2, TILE_SIZE);

            // fill right
            draw_triangles(
                &[
                    [start, ivec2(TILE_SIZE, 0), ivec2(TILE_SIZE, TILE_SIZE)],
                    [ivec2(TILE_SIZE, TILE_SIZE), end, start],
                ]
            );

            // draw border
            draw_line(start.x, start.y, end.x, end.y);
            
        },
        7 => {

            let start = ivec2(0, TILE_SIZE / 2);
            let end = ivec2(TILE_SIZE / 2, 0);

            // fill bottom right
            draw_triangles(
                &[
                    [start, end, ivec2(TILE_SIZE, 0)],
                    [ivec2(TILE_SIZE, 0), ivec2(TILE_SIZE, TILE_SIZE), start],
                    [start, ivec2(TILE_SIZE, TILE_SIZE), ivec2(0, TILE_SIZE)]
                ]
            );

            // draw border
            draw_line(start.x, start.y, end.x, end.y);

        },
        8 => {

            let start = ivec2(0, TILE_SIZE / 2);
            let end = ivec2(TILE_SIZE / 2, 0);

            // fill top left
            draw_triangles(
                &[
                    [start, ivec2(0, 0), end]
                ]
            );

            // draw border
            draw_line(start.x, start.y, end.x, end.y);

        },
        9 => {


            let start = ivec2(TILE_SIZE / 2, 0);
            let end = ivec2(TILE_SIZE / 2, TILE_SIZE);

            // fill left
            draw_triangles(
                &[
                    [ivec2(0, 0), start, end],
                    [end, ivec2(0, TILE_SIZE), ivec2(0, 0)],
                ]
            );

            // draw border
            draw_line(start.x, start.y, end.x, end.y);

        },
        10 => {

            let start_bottom_left = ivec2(0, TILE_SIZE / 2);
            let end_bottom_left = ivec2(TILE_SIZE / 2, TILE_SIZE);

            let start_top_right = ivec2(TILE_SIZE / 2, 0);
            let end_top_right = ivec2(TILE_SIZE, TILE_SIZE / 2);

            // fill middle
            draw_triangles(
                &[
                    [start_bottom_left, ivec2(0, 0), start_top_right],
                    [start_bottom_left, start_top_right, end_top_right],
                    [end_top_right, end_bottom_left, start_bottom_left],
                    [end_top_right, ivec2(TILE_SIZE, TILE_SIZE), end_bottom_left]
                ]
            );

            // draw border

            draw_line(start_bottom_left.x, start_bottom_left.y, end_bottom_left.x, end_bottom_left.y);
            draw_line(start_top_right.x, start_top_right.y, end_top_right.x, end_top_right.y);

        },
        11 => {

            let start = ivec2(TILE_SIZE / 2, 0);
            let end = ivec2(TILE_SIZE, TILE_SIZE / 2);

            // fill bottom left
            draw_triangles(
                &[
                    [start, end, ivec2(TILE_SIZE, TILE_SIZE)],
                    [ivec2(TILE_SIZE, TILE_SIZE), ivec2(0, TILE_SIZE), start],
                    [start, ivec2(0, TILE_SIZE), ivec2(0, 0)]
                ]
            );
            
            // draw border
            draw_line(start.x, start.y, end.x, end.y);

        },
        12 => {

            let start = ivec2(0, TILE_SIZE / 2);
            let end = ivec2(TILE_SIZE, TILE_SIZE / 2);

            // fill above
            draw_triangles(
                &[
                    [start, ivec2(0, 0), ivec2(TILE_SIZE, 0)],
                    [ivec2(TILE_SIZE, 0), end, start],
                ]
            );

            // draw border
            draw_line(start.x, start.y, end.x, end.y);

        },
        13 => {

            let start = ivec2(TILE_SIZE / 2, TILE_SIZE);
            let end = ivec2(TILE_SIZE, TILE_SIZE / 2);

            // fill top left
            draw_triangles(
                &[
                    [start, end, ivec2(0, TILE_SIZE)],
                    [ivec2(0, TILE_SIZE), ivec2(0, 0), end],
                    [end, ivec2(0, 0), ivec2(TILE_SIZE, 0)]
                ]
            );

            // draw border
            draw_line(start.x, start.y, end.x, end.y);

        },
        14 => {
            
            let start = ivec2(0, TILE_SIZE / 2);
            let end = ivec2(TILE_SIZE / 2, TILE_SIZE);

            // fill top right
            draw_triangles(
                &[
                    [start, ivec2(0, 0), ivec2(TILE_SIZE, 0)],
                    [ivec2(TILE_SIZE, 0), ivec2(TILE_SIZE, TILE_SIZE), start],
                    [start, ivec2(TILE_SIZE, TILE_SIZE), end]
                ]
            );

            // draw border
            draw_line(start.x, start.y, end.x, end.y);

        },
        15 => {

            let top_left = ivec2(0, 0);
            let top_right = ivec2(TILE_SIZE, 0);
            let bottom_left = ivec2(0, TILE_SIZE);
            let bottom_right = ivec2(TILE_SIZE, TILE_SIZE);

            // fill
            draw_triangles(
                &[
                    [top_left, top_right, bottom_right],
                    [bottom_right, bottom_left, top_left],
                ]
            );
            
        },
        _ => {}
    }

}

/// Rasterizes all the tiles for marching squares into a texture atlas (single row).
fn rasterize_tile_atlas(line_color: Color, fill_color: Color, line_thickness: f32) -> RenderTarget {

    // 16 is the number of tiles!
    let texture_width = (TILE_SIZE * 16) + (TILE_PADDING * 15);
    let texture_height = TILE_SIZE;

    let render_target = render_target(texture_width as u32, texture_height as u32);
    render_target.texture.set_filter(FilterMode::Nearest);

    let render_target_camera = create_render_target_camera(render_target.clone());

    set_camera(&render_target_camera);
    clear_background(WHITE.with_alpha(0.0));

    for i in 0..16 {
        let should_fill_tile = true;
        let current_offset = vec2(((TILE_PADDING + TILE_SIZE) * i) as f32, 0.0);
        rasterize_tile(current_offset, i, line_color, fill_color, line_thickness, should_fill_tile);
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
    isolevels: Vec<u8>,
    data: Vec<u8>,
    size: UVec2
}

impl Heightmap {

    pub fn new(w: u32, h: u32, isolevels: Vec<u8>) -> Heightmap {
        Heightmap {
            isolevels: isolevels,
            data: vec![0; (w * h) as usize],
            size: uvec2(w, h)
        }
    }

    pub fn world_to_tile(&self, x: f32, y: f32) -> (i32, i32) {
        (x as i32 / REAL_TILE_SIZE as i32, y as i32 / REAL_TILE_SIZE as i32)
    }

    pub fn tile_to_world(&self, x: i32, y: i32) -> Vec2 {
        vec2(
            (x * REAL_TILE_SIZE as i32) as f32,
            (y * REAL_TILE_SIZE as i32) as f32
        )
    }

    pub fn get(&self, x: i32, y: i32) -> u8 {
        let idx = x + y * self.size.x as i32;
        if idx < 0 || idx >= self.data.len() as i32 { 0 } else { self.data[idx as usize] }
    }

    pub fn set(&mut self, x: i32, y: i32, v: u8) {
        let idx = x + y * self.size.x as i32;
        if (idx < 0 || idx >= self.data.len() as i32) == false {
            self.data[idx as usize] = v;
        }
    }

    pub fn add(&mut self, x: i32, y: i32, v: i8) {
        let idx = x + y * self.size.x as i32;
        if (idx < 0 || idx >= self.data.len() as i32) == false {
            if v > 0 {
                self.data[idx as usize] = self.data[idx as usize].saturating_add(v as u8);
            } else if v < 0 {
                self.data[idx as usize] = self.data[idx as usize].saturating_sub(v.abs() as u8);
            }
        }
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
        get_internal_gl().quad_context.new_texture_from_data_and_format(
            bytes,
            miniquad::TextureParams {
                format: miniquad::TextureFormat::Alpha,
                wrap: miniquad::TextureWrap::Clamp,
                width: map.size.x,
                height: map.size.y,
                kind: miniquad::TextureKind::Texture2D,
                min_filter: FilterMode::Nearest,
                mag_filter: FilterMode::Nearest,
                mipmap_filter: MipmapFilterMode::Nearest,
                allocate_mipmaps: true,
                sample_count: 0
            }
        )
    };

    Texture2D::from_miniquad_texture(miniquad_texture)

}

fn create_height_field(w: u32, h: u32) -> Heightmap {

    let num_isolevels = 4;
    let isolevels = (0..num_isolevels).map(|i| ((255.0 / num_isolevels as f32) * i as f32) as u8).collect();

    let mut new_heightmap = Heightmap::new(w, h, isolevels);

    let w = w as i32;
    let h = h as i32;

    for x in 0..w {
        for y in 0..h {
            if x <= 2 || x >= w - 2 || y <= 2 || y >= h - 2 {
                continue;
            }
            new_heightmap.set(x, y, 3);
        }
    }

    new_heightmap

}

fn apply_noise_to_height_field(map: &mut Heightmap, seed: u32) {

    let perlin = Perlin::new(seed);
    let noise = ScaleBias::new(
        Turbulence::<_, Perlin>::new(Perlin::new(seed * 4))
    ).set_scale(0.125);

    let combined_noise = Add::new(perlin, noise);

    let noise_map = PlaneMapBuilder::<_, 2>::new(combined_noise)
        .set_size((map.width() * 8) as usize, (map.height() * 8) as usize)
        .set_x_bounds(0.0, map.width() as f64 / 2.0)
        .set_y_bounds(0.0, map.height() as f64 / 2.0)
        .build();

    for x in 1..map.width() - 1 {
        for y in 1..map.height() - 1 {
            let v: f64 = noise_map.get_value(x as usize, y as usize) * 255.0;
            map.set(x as i32, y as i32, v as u8);
        }
    }

}

/// Given a tile position in the heightmap, returns the marching squares tile index to render.
fn height_map_position_to_index(map: &Heightmap, isovalue: u8, x: u32, y: u32) -> u8 {

    let x = x as i32;
    let y = y as i32;

    let top_left_bit = ((map.get(x, y) > isovalue) as u8) << 0;
    let top_right_bit = ((map.get(x + 1, y) > isovalue) as u8) << 1;
    let bottom_right_bit = ((map.get(x + 1, y + 1) > isovalue) as u8) << 2;
    let bottom_left_bit = ((map.get(x, y + 1) > isovalue) as u8) << 3;

    let final_index = top_left_bit | top_right_bit | bottom_left_bit | bottom_right_bit;

    final_index

}

fn is_tile_in_view(x: f32, y: f32) -> bool {

    let tile_rect = Rect {
        x: x, y: y,
        w: REAL_TILE_SIZE as f32,
        h: REAL_TILE_SIZE as f32
    };

    let screen_rect = Rect {
        x: 0.0, y: 0.0,
        w: screen_width(),
        h: screen_height()
    };

    tile_rect.intersect(screen_rect).is_some()
    
}

fn draw_height_field_layer(active: &GameCamera, map: &Heightmap, atlas: &Texture2D, isovalue: u8) {

    let height_field_offset = vec2(
        REAL_TILE_SIZE as f32 / 2.0,
        REAL_TILE_SIZE as f32 / 2.0
    );

    for x in 0..map.width() {

        let view_pos = active.camera.world_to_screen(
            height_field_offset + vec2(REAL_TILE_SIZE as f32 * x as f32, 0.0)
        );

        if view_pos.x < -(REAL_TILE_SIZE as f32) || view_pos.x > screen_width() {
            continue;
        }

        for y in 0..map.height() {

            let idx = height_map_position_to_index(map, isovalue, x, y);
            let offset_x = ((TILE_PADDING + TILE_SIZE) * idx as i32) as f32;
            let tile_pos = height_field_offset + vec2((x as i32 * REAL_TILE_SIZE) as f32, (y as i32 * REAL_TILE_SIZE) as f32);

            if idx == 0 {
                continue; // nothing to render for tile 0, which is empty
            }

            let dest_size = vec2(
                REAL_TILE_SIZE as f32,
                REAL_TILE_SIZE as f32
            );

            let source_rect = Rect {
                x: offset_x + (TILE_PADDING / 2) as f32 + 0.1,
                y: 0.0 + 0.1,
                w: REAL_TILE_SIZE as f32 - 0.2,
                h: REAL_TILE_SIZE as f32 - 0.2
            };

            let pleasant_earthy_green = Color::from_rgba(104, 118, 53, 255);

            draw_texture_ex(
                &atlas,
                tile_pos.x,
                tile_pos.y,
                pleasant_earthy_green.lighten(0.25 * (isovalue as f32 / 255.0)),
                DrawTextureParams {
                    dest_size: Some(dest_size),
                    source: Some(source_rect),
                    ..Default::default()
                }
            )

        }
    }

}

fn draw_height_field(active: &GameCamera, map: &Heightmap, atlas: &Texture2D, render_debug_text: bool) {

    for &isolevel in &map.isolevels {
        draw_height_field_layer(active, map, atlas, isolevel);
    }

    if render_debug_text {
        for x in 0..map.width() {
            for y in 0..map.height() {

                let first_isolevel = *map.isolevels.first().unwrap();
                let text_x = (x as i32 * TILE_SIZE) as f32 + (TILE_SIZE / 2) as f32;
                let text_y = (y as i32 * TILE_SIZE) as f32 + (TILE_SIZE / 2) as f32;
                let v = map.get(x as i32, y as i32);

                draw_text(
                    v.to_string().as_str(),
                    text_x, text_y + (TILE_SIZE as f32) / 2.0,
                    12.0,
                    if v > first_isolevel { GREEN } else { RED }
                );

            }
        }
    }

}

struct GameCamera {
    size: Vec2,
    camera_zoom: f32,
    camera: Camera2D,
}

impl GameCamera {

    pub fn new(size: Vec2) -> GameCamera {

        let camera = Camera2D::from_display_rect_fixed(
            Rect { x: 0.0, y: 0.0, w: size.x, h: size.y }
        );

        GameCamera {
            size: size,
            camera: camera,
            camera_zoom: 1.0
        }

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

    let mouse_wheel_delta = mouse_wheel();

    let min_zoom = 0.5;
    let max_zoom = 4.0;

    let new_zoom = (active.camera_zoom - mouse_wheel_delta.1 * dt).clamp(min_zoom, max_zoom);
    let new_size = active.size * new_zoom;

    let new_camera = Camera2D::from_display_rect_fixed(
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
    mouse_wheel_delta.1 != 0.0

}

fn handle_camera_panning(active: &mut GameCamera, last_mouse_position: Vec2, _dt: f32) {

    let is_middle_mouse_down = is_mouse_button_down(MouseButton::Middle);

    if is_middle_mouse_down {
        let mouse_position_v: Vec2 = mouse_position().into();
        let mouse_position_delta: Vec2 = last_mouse_position - mouse_position_v;
        active.camera.target += mouse_position_delta * active.camera_zoom;
    }

}

fn modify_in_radius(map: &mut Heightmap, center_x: i32, center_y: i32, radius: i32, v: i8) {

    for x in center_x - radius .. center_x + radius {
        for y in center_y - radius .. center_y + radius {
            let is_in_radius = Vec2::distance(vec2(center_x as f32, center_y as f32), vec2(x as f32, y as f32)) < radius as f32;
            if is_in_radius {
                map.add(x, y, v);
            }
        }
    }

}

fn handle_height_map_input(active: &mut GameCamera, map: &mut Heightmap, _dt: f32) -> bool {

    let radius = 2;
    let modification = 4;

    let is_mouse_left_down = is_mouse_button_down(MouseButton::Left);
    let is_mouse_right_down = is_mouse_button_down(MouseButton::Right);

    let Vec2 { x: mouse_x, y: mouse_y } = active.camera.screen_to_world(mouse_position().into());
    let (mouse_tile_x, mouse_tile_y) = map.world_to_tile(mouse_x, mouse_y);

    if is_mouse_left_down {
        modify_in_radius(map, mouse_tile_x, mouse_tile_y, radius, modification);
    }

    if is_mouse_right_down {
        modify_in_radius(map, mouse_tile_x, mouse_tile_y, radius, -modification);
    }

    is_mouse_left_down || is_mouse_right_down

}

/// Handles the game input, if any change happened, this returns true.
fn handle_game_input(active: &mut GameCamera, map: &mut Heightmap, out_seed: &mut u32, dt: f32) -> bool {

    let was_reload_key_pressed = is_key_pressed(KeyCode::R);

    if was_reload_key_pressed {
        let new_random_seed = gen_range(0, u32::MAX / 4);
        apply_noise_to_height_field(map, new_random_seed);
        *out_seed = new_random_seed;
    }

    let height_map_changed = handle_height_map_input(active, map, dt);

    height_map_changed

}

fn draw_debug_text(active: &GameCamera, map: &Heightmap, debug: &mut DebugText, seed: u32) {

    let world_pos = active.camera.screen_to_world(mouse_position().into());
    let tile_pos = map.world_to_tile(world_pos.x, world_pos.y);
    let tile = map.get(tile_pos.0, tile_pos.1);

    debug.draw_text(format!("world position under mouse: {}", world_pos), TextPosition::TopLeft, WHITE);
    debug.draw_text(format!("tile position under mouse: {:?}", tile_pos), TextPosition::TopLeft, WHITE);
    debug.draw_text(format!("tile under mouse: {:?}", tile), TextPosition::TopLeft, WHITE);
    debug.draw_text(format!("seed: {}", seed), TextPosition::TopLeft, WHITE);

}

#[macroquad::main("territory")]
async fn main() {

    // step 1. rasterize the 16 tiles for marching squares into a buffer.
    // step 2. upload the texture with the rasterized tile data.
    // step 3. create a buffer that represents the map data.
    // step 4. upload a texture with the map data.
    // step 5. ???

    // highly scientific
    let mut current_seed = 64;

    let mut debug_text = DebugText::new();

    let mut should_show_tile_atlas = false;
    let mut should_rasterize_tile_atlas = true;
    let mut rasterized_tile_atlas: Option<RenderTarget> = None;

    let mut height_field = create_height_field(128, 128);
    apply_noise_to_height_field(&mut height_field, current_seed);

    // #TODO: figure out what the purpose of this was, literally do not remember
    // let height_field_texture = create_height_field_buffer_texture(&height_field);

    let mut last_mouse_position: Vec2 = mouse_position().into();
    let mut active_camera = GameCamera::new(screen_dimensions());
    let render_debug_text = false;
    
    loop {

        active_camera.size = screen_dimensions();
        let dt = get_frame_time();

        // re-rasterize atlas if necessary
        if should_rasterize_tile_atlas {

            let line_thickness = active_camera.camera_zoom.max(1.0) * 2.0;
            rasterized_tile_atlas = Some(rasterize_tile_atlas(BLACK.lighten(0.75), WHITE, line_thickness));

            // NOTE: this flushes the render passes so that our tile atlas actually gets rendered before we try to actually use it, otherwise we get unitialized memory it seems like
            set_default_camera();

        }

        let camera_changed = handle_camera_input(&mut active_camera, last_mouse_position, dt);
        let game_changed = handle_game_input(&mut active_camera, &mut height_field, &mut current_seed, dt);
        should_rasterize_tile_atlas = camera_changed || game_changed;

        // update last mouse pos
        last_mouse_position = mouse_position().into();

        // let pleasant_earthy_green = Color::from_rgba(104, 118, 53, 255);
        let murky_ocean_blue = Color::from_rgba(21, 119, 136, 255);
        
        set_camera(&active_camera.camera);
        clear_background(murky_ocean_blue);

        // enable this to see the tile atlas generated
        if should_show_tile_atlas {
            let atlas_texture = &rasterized_tile_atlas.as_ref().unwrap().texture;
            draw_rectangle_lines(0.0, 0.0, atlas_texture.width(), atlas_texture.height(), 1.0, GREEN);
            draw_texture(
                atlas_texture,
                0.0, 0.0,
                WHITE
            );
        }

        draw_height_field(
            &active_camera,
            &height_field,
            &rasterized_tile_atlas.as_ref().unwrap().texture,
            render_debug_text
        );

        // now draw screen space stuff, debug text, etc

        set_default_camera();

        debug_text.new_frame();
        draw_debug_text(&active_camera, &height_field, &mut debug_text, current_seed);

        if is_key_pressed(KeyCode::Escape) {
            break;
        }

        next_frame().await;

    }

}