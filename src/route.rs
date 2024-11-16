use crate::db::DbConnection;
use crate::ws::QUEUE;
use crate::ws::USERS;
use crate::{util::uniqid, ws::GroupMsg, ws::QueueItem, ws::WsConn};
use actix_web::http::header::{CACHE_CONTROL, REFERER, USER_AGENT};
use actix_web::{get, post, web, HttpMessage, HttpRequest, HttpResponse, Responder};
use mongodb::bson::Document;

use std::sync::Arc;

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

#[get("/stat/error_log/status")]
async fn status() -> impl Responder {
    let data = USERS.stat();
    HttpResponse::Ok()
        .insert_header((
            CACHE_CONTROL,
            format!("public,max-age={}", QUEUE.read().unwrap().len()),
        ))
        .json(data)
}

#[get("/aggregate/{group:[\\w\\-]{1,20}}")]
async fn aggregate(
    params: web::Query<Document>,
    db_conn: web::Data<Arc<DbConnection>>,
    group: web::Path<String>,
) -> impl Responder {
    let group = group.into_inner();
    let res = db_conn.aggregate(&group, params.into_inner()).await;
    if res.is_err() {
        return HttpResponse::InternalServerError().body(format!("{:?}", res.err().unwrap()));
    }
    HttpResponse::Ok().json(res.unwrap())
}

#[get("/ws/{group:[\\w\\-]{1,20}}")]
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
    let (ua, ip, refer) = extract_headers(&req);
    if ctype.contains("text") || ctype.contains("json") {
        return match std::str::from_utf8(&bytes) {
            Err(e) => HttpResponse::BadRequest().body(format!("{:?}", e)),
            Ok(data) => {
                QUEUE.write().unwrap().push(QueueItem {
                    ua,
                    ip,
                    refer,
                    data: Arc::new(GroupMsg {
                        group,
                        data: data.trim().to_owned(),
                        bytes: web::Bytes::new(),
                    }),
                });
                HttpResponse::Ok().body("")
            }
        };
    }
    QUEUE.write().unwrap().push(QueueItem {
        ua,
        ip,
        refer,
        data: Arc::new(GroupMsg {
            group,
            data: "".to_owned(),
            bytes,
        }),
    });
    HttpResponse::Ok().body("")
}

fn extract_headers(req: &HttpRequest) -> (String, String, String) {
    let ua = match req.headers().get(USER_AGENT) {
        None => "".to_owned(),
        Some(v) => v.to_str().unwrap_or_default().to_owned(),
    };
    let ip = req
        .connection_info()
        .realip_remote_addr()
        .unwrap_or("")
        .to_owned();
    let refer = match req.headers().get(REFERER) {
        None => "".to_owned(),
        Some(v) => v.to_str().unwrap_or_default().to_owned(),
    };
    (ua, ip, refer)
}
