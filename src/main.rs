#[macro_use]
extern crate lazy_static;

use actix_web::{http::header::ACCESS_CONTROL_ALLOW_ORIGIN, middleware, App, HttpServer};
use std::env;
use tokio::runtime::Builder;

mod db;
mod queue;
mod route;
mod util;
mod ws;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let rt = Builder::new_multi_thread()
        .enable_all()
        .worker_threads(2)
        .thread_name("db")
        .thread_stack_size(1024 * 1024)
        .build()
        .unwrap();
    rt.spawn(ws::taskloop());
    HttpServer::new(|| {
        App::new()
            .wrap(middleware::DefaultHeaders::new().header(ACCESS_CONTROL_ALLOW_ORIGIN, "*"))
            .service(route::hello)
            .service(route::status)
            .service(route::ws)
            .service(route::error_log)
    })
    .bind(opt())?
    .run()
    .await
}

fn opt() -> String {
    let mut opts: Vec<String> = vec![env::var("ADDR").unwrap_or("127.0.0.1:8080".to_string())];
    let mut index = 0;
    let mut first = true;
    for argument in env::args() {
        if first {
            first = false;
            continue;
        }
        if index >= 1 {
            break;
        }
        opts[index] = argument;
        index = index + 1;
    }
    opts[0].clone()
}
