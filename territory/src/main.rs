use macroquad::prelude::*;

const TILE_SIZE: i32 = 32;

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

fn rasterize_tile_atlas(color: Color) -> Texture2D {

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

    render_target.texture

}

fn create_render_target_camera(render_target: RenderTarget) -> Camera2D {

    let size = vec2(render_target.texture.width(), render_target.texture.height());
    let mut render_target_camera = Camera2D::from_display_rect(Rect { x: 0.0, y: 0.0, w: size.x, h: size.y});
    render_target_camera.render_target = Some(render_target);

    render_target_camera

}

#[macroquad::main("territory")]
async fn main() {

    // step 1. rasterize the 16 tiles for marching squares into a buffer.
    // step 2. upload the texture with the rasterized tile data.
    // step 3. create a buffer that represents the map data.
    // step 4. upload a texture with the map data.
    // step 5. ???

    let mut has_rasterized_tile_atlas = false;
    let mut rasterized_tile_atlas: Option<Texture2D> = None;
    
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

        draw_texture(
            rasterized_tile_atlas.unwrap(),
            0.,
            0.,
            WHITE,
        );

        if is_key_pressed(KeyCode::Escape) {
            break;
        }

        next_frame().await;

    }

}