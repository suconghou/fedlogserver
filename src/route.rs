use crate::db::DbConnection;
use crate::ws::{QUEUE, USERS};
use crate::{util::uniqid, ws::GroupMsg, ws::QueueItem, ws::WsConn};
use actix_web::http::header::{CACHE_CONTROL, REFERER, USER_AGENT};
use actix_web::{HttpMessage, HttpRequest, HttpResponse, Responder, get, post, web};
use mongodb::bson::Document;
use std::sync::Arc;
use tokio::sync::mpsc;

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

#[get("/stat/error_log/status")]
async fn status() -> impl Responder {
    let data = USERS.stat();
    HttpResponse::Ok()
        .insert_header((CACHE_CONTROL, format!("public,max-age={}", QUEUE.count())))
        .json(data)
}

#[get("/aggregate/{group:[\\w\\-]{1,20}}")]
async fn aggregate(
    params: web::Query<Document>,
    db_conn: web::Data<Arc<DbConnection>>,
    group: web::Path<String>,
) -> impl Responder {
    let group = group.into_inner();
    match db_conn.aggregate(&group, params.into_inner()).await {
        Ok(res) => HttpResponse::Ok().json(res),
        Err(e) => HttpResponse::InternalServerError().body(format!("{:?}", e)),
    }
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
    queue_tx: web::Data<mpsc::Sender<QueueItem>>,
) -> impl Responder {
    let group = group.into_inner();
    let ctype = req.content_type();
    let msg = if ctype.contains("text") || ctype.contains("json") {
        match std::str::from_utf8(&bytes) {
            Ok(data) => Arc::new(GroupMsg {
                group,
                data: data.trim().to_owned(),
                bytes: web::Bytes::new(),
            }),
            Err(e) => return HttpResponse::BadRequest().body(format!("{:?}", e)),
        }
    } else {
        Arc::new(GroupMsg {
            group,
            data: "".to_owned(),
            bytes,
        })
    };
    let (ua, ip, refer) = extract_headers(&req);
    let item = QueueItem {
        ua,
        ip,
        refer,
        data: msg,
    };
    match queue_tx.send(item).await {
        Ok(_) => {
            QUEUE.add(1);
            HttpResponse::Ok().body("")
        }
        Err(e) => HttpResponse::InternalServerError().body(format!("{:?}", e)),
    }
}

fn extract_headers(req: &HttpRequest) -> (String, String, String) {
    let ua = req
        .headers()
        .get(USER_AGENT)
        .and_then(|v| v.to_str().ok())
        .unwrap_or_default()
        .to_owned();
    let ip = req
        .connection_info()
        .realip_remote_addr()
        .unwrap_or("")
        .to_owned();
    let refer = req
        .headers()
        .get(REFERER)
        .and_then(|v| v.to_str().ok())
        .unwrap_or_default()
        .to_owned();
    (ua, ip, refer)
}
