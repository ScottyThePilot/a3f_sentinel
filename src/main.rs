#[macro_use] extern crate serde;
#[macro_use] extern crate util_macros;
extern crate serenity;
extern crate singlefile;
extern crate tokio;

#[macro_use] mod macros;
mod commands;
mod data;
mod error;
mod handler;
mod util;

use crate::util::ResultExt;

#[tokio::main]
async fn main() {
  crate::handler::launch().await.report();
}
