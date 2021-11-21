use std::sync::Arc;

use actix_web::HttpMessage;
use actix_web::{get, post, web, HttpRequest, HttpResponse, Responder};

use crate::ws::QUEUE;
use crate::ws::USERS;
use crate::{util::uniqid, ws::GroupMsg, ws::QueueItem, ws::WsConn};

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

#[get("/stat/error_log/status")]
async fn status() -> impl Responder {
    let data = USERS.stat();
    HttpResponse::Ok()
        .header(
            "Cache-Control",
            format!("public,max-age={}", QUEUE.read().unwrap().len()),
        )
        .json(data)
}

#[get("/stat/error_log/ws/{group:[\\w\\-]{1,20}}")]
async fn ws(req: HttpRequest, stream: web::Payload, group: web::Path<String>) -> impl Responder {
    let group = group.into_inner();
    let conn = WsConn {
        id: uniqid(),
        group,
    };
    actix_web_actors::ws::start(conn, &req, stream)
}

#[post("/stat/error_log/{group:[\\w\\-]{1,20}}")]
async fn error_log(
    req: HttpRequest,
    bytes: web::Bytes,
    group: web::Path<String>,
) -> impl Responder {
    let group = group.into_inner();
    let ctype = req.content_type();
    let ua = match req.headers().get("User-Agent") {
        None => "".to_owned(),
        Some(v) => v.to_str().unwrap_or_default().to_owned(),
    };
    let ip = req
        .connection_info()
        .realip_remote_addr()
        .unwrap_or("")
        .to_owned();
    if ctype.contains("text") || ctype.contains("json") {
        return match std::str::from_utf8(&bytes) {
            Err(e) => HttpResponse::BadRequest().body(format!("{:?}", e)),
            Ok(data) => {
                QUEUE.write().unwrap().push(Arc::new(QueueItem {
                    ua,
                    ip,
                    data: GroupMsg {
                        group,
                        data: data.to_string(),
                        bytes: web::Bytes::new(),
                    },
                }));
                HttpResponse::Accepted().body("")
            }
        };
    }
    QUEUE.write().unwrap().push(Arc::new(QueueItem {
        ua,
        ip,
        data: GroupMsg {
            group,
            data: "".to_owned(),
            bytes,
        },
    }));
    HttpResponse::Accepted().body("")
}
