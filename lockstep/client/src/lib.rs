#![feature(let_chains)]
#![allow(async_fn_in_trait)]

pub const IS_DEBUGGING: bool = false;

pub mod app;
pub mod step;
pub mod game;
pub mod network;
pub mod extensions;
pub mod relay;