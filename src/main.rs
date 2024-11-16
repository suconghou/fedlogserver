use actix_web::{
    dev::Service,
    http::header::ACCESS_CONTROL_ALLOW_ORIGIN,
    middleware,
    web::{self, Data},
    App, HttpResponse, HttpServer,
};
use futures::FutureExt;
use std::{env, sync::Arc};
use tokio::runtime::Builder;

mod db;
mod queue;
mod route;
mod util;
mod ws;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let db_conn = Arc::new(db::DbConnection::new().await);
    let rt = Builder::new_multi_thread()
        .enable_all()
        .worker_threads(2)
        .thread_name("db")
        .thread_stack_size(1024 * 1024)
        .build()
        .unwrap();
    rt.spawn(ws::taskloop(db_conn.clone()));
    HttpServer::new(move || {
        App::new()
            .app_data(Data::new(db_conn.clone()))
            .wrap(middleware::DefaultHeaders::new().add((ACCESS_CONTROL_ALLOW_ORIGIN, "*")))
            .service(route::hello)
            .service(route::status)
            .service(route::error_log)
            .service(
                web::scope("/stat/error_log")
                    .wrap_fn(|req, srv| {
                        let key = env::var("AUTH_KEY").unwrap_or("".to_owned());
                        if !key.is_empty() {
                            let t = match req.headers().get("Authorization") {
                                Some(v) => v.to_str().unwrap_or(""),
                                None => "",
                            };
                            if t != key {
                                return async {
                                    Ok(req.into_response(HttpResponse::Unauthorized()))
                                }
                                .boxed_local();
                            }
                        }
                        return srv.call(req);
                    })
                    .service(route::aggregate)
                    .service(route::ws),
            )
    })
    .bind(opt())?
    .run()
    .await
}

fn opt() -> String {
    env::args()
        .nth(1)
        .unwrap_or_else(|| env::var("ADDR").unwrap_or("127.0.0.1:8080".to_owned()))
}
