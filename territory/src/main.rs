use macroquad::prelude::*;

#[macroquad::main("territory")]
async fn main() {
    
    loop {

        clear_background(WHITE);

        if is_key_pressed(KeyCode::Escape) {
            break;
        }

        next_frame().await;

    }

}