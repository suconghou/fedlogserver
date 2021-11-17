use actix_web::{get, post, web, HttpRequest, HttpResponse, Responder};

use crate::ws::QUEUE;
use crate::{util::uniqid, ws::GroupMsg, ws::WsConn};

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
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
async fn error_log(bytes: web::Bytes, group: web::Path<String>) -> impl Responder {
    let group = group.into_inner();
    match std::str::from_utf8(&bytes) {
        Err(e) => HttpResponse::BadRequest().body(format!("{:?}", e)),
        Ok(data) => {
            QUEUE.write().unwrap().push(GroupMsg {
                group,
                data: data.to_string(),
            });
            HttpResponse::Accepted().body("")
        }
    }
}
