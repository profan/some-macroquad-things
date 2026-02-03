
use macroquad::prelude::*;

#[macroquad::main("collapse")]
async fn main() {
    loop {
        clear_background(WHITE);
        next_frame().await;
    }
}
