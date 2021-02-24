#[macro_use] extern crate serde;
extern crate async_std;
extern crate ron;
extern crate serenity;
extern crate tokio;

#[macro_use] mod macros;
mod commands;
mod config;
mod error;
mod handler;

#[tokio::main]
async fn main() {
  crate::handler::launch().await.unwrap();
}
