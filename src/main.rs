#![feature(let_chains)]

use chocho::prelude::*;

use handler::YiriHandler;

mod handler;
mod talk;

#[chocho::main(handler = YiriHandler::new())]
async fn main(_client: RQClient) {}
