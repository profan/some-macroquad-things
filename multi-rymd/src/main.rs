#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(async_fn_in_trait)]
#![feature(let_chains)]

use game::RymdGame;
use macroquad::prelude::*;

use lockstep_client::app::ApplicationState;

mod game;
mod model;
mod utils;
mod view;

use u64 as EntityID;
use lockstep_client::step::PeerID as PlayerID;

#[macroquad::main("multi-rymd")]
async fn main() {

    let mut main_loop_update_time_ms = 0.0;
    let mut app = ApplicationState::new("multi-rymd", RymdGame::new());

    app.set_target_host("94.13.52.142");
    app.set_debug_text_colour(WHITE);
    app.load_resources().await;

    loop {

        let dt = get_frame_time();

        app.get_game().stats.main_time_ms = main_loop_update_time_ms;
        measure_scope!(main_loop_update_time_ms);

        app.handle_messages();
        clear_background(Color::from_hex(0x181425));

        app.update();
        app.draw(dt);

        next_frame().await;

    }

}