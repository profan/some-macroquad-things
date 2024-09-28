use macroquad::prelude::*;

/// Creates a camera given a render target.
fn create_render_target_camera(render_target: RenderTarget) -> Camera2D {

    let size = vec2(render_target.texture.width(), render_target.texture.height());
    let mut render_target_camera = Camera2D::from_display_rect(Rect { x: 0.0, y: 0.0, w: size.x, h: size.y});
    render_target_camera.render_target = Some(render_target);

    render_target_camera

}

fn render_something_into_render_target() -> RenderTarget {

    let texture_width = 1;
    let texture_height = 1;

    let render_target = render_target(texture_width as u32, texture_height as u32);
    render_target.texture.set_filter(FilterMode::Linear);

    let render_target_camera = create_render_target_camera(render_target.clone());

    set_camera(&render_target_camera);
    clear_background(PINK);

    render_target

}

#[macroquad::main("render-targets-be-like")]
async fn main() {

    let mut current_render_target: Option<RenderTarget> = None;
    
    loop {

        if current_render_target.is_none() {
            current_render_target = Some(render_something_into_render_target());
            
            // NOTE #1: uncomment this to get it to render the render target texture properly
            set_default_camera();
        }

        set_camera(&Camera2D::default());
        clear_background(WHITE);

        draw_texture(
            &current_render_target.as_ref().unwrap().texture,
            0.0, 0.0,
            WHITE
        );

        // NOTE #2: if you uncomment this, the white clear background renders properly, otherwise you just get a black screen as well as no pink square
        // set_default_camera();
        next_frame().await;

    }

}