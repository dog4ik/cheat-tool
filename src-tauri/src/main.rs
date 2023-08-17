// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![feature(let_chains)]

use cheat_toolkit::api::run;
#[tokio::main]
async fn main() {
    dotenvy::dotenv().unwrap();
    run().await
}
