#![allow(dead_code)]
#![allow(unused_variables)]
#![feature(let_chains)]

use game::RymdGame;
use macroquad::prelude::*;

use lockstep_client::app::ApplicationState;

mod game;
mod model;
mod utils;
mod view;

use u64 as EntityID;

#[macroquad::main("multi-rymd")]
async fn main() {

    let mut app = ApplicationState::new("multi-rymd", RymdGame::new());

    app.set_target_host("94.13.52.142");
    app.set_debug_text_colour(WHITE);
    app.load_resources().await;

    loop {

        app.handle_messages();
        clear_background(Color::from_hex(0x181425));

        app.update();
        app.draw();

        next_frame().await;

    }

}