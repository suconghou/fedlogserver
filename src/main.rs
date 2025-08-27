use actix_web::{
    App, HttpResponse, HttpServer,
    dev::Service,
    http::header::ACCESS_CONTROL_ALLOW_ORIGIN,
    middleware,
    web::{self, Data},
};
use futures::FutureExt;
use std::{env, sync::Arc};
use tokio::sync::mpsc;

mod db;
mod route;
mod stat;
mod util;
mod ws;

fn extract_auth_token(req: &actix_web::dev::ServiceRequest) -> Option<String> {
    req.headers()
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
        .or_else(|| req.cookie("auth_key").map(|v| v.value().to_string()))
        .or_else(|| {
            req.query_string().split('&').find_map(|param| {
                let mut parts = param.splitn(2, '=');
                if let (Some(key), Some(value)) = (parts.next(), parts.next()) {
                    if key == "auth_key" {
                        return Some(value.to_string());
                    }
                }
                None
            })
        })
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let db_conn = Arc::new(db::DbConnection::new().await);
    let (tx, rx) = mpsc::channel(1024);
    tokio::spawn(ws::taskloop(db_conn.clone(), rx));
    HttpServer::new(move || {
        App::new()
            .app_data(Data::new(db_conn.clone()))
            .app_data(Data::new(tx.clone()))
            .wrap(middleware::DefaultHeaders::new().add((ACCESS_CONTROL_ALLOW_ORIGIN, "*")))
            .service(route::hello)
            .service(route::status)
            .service(route::error_log)
            .service(
                web::scope("/stat/error_log")
                    .wrap_fn(|req, srv| {
                        let key = env::var("AUTH_KEY").unwrap_or("".to_owned());
                        if !key.is_empty() {
                            let t = extract_auth_token(&req).unwrap_or_default();
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
