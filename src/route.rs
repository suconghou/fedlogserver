use actix_web::{get, post, web, Error,HttpRequest, HttpResponse, Responder};

use crate::{util::uniqid, ws::{WsConn}};


#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}


#[get("/stat/error_log/ws/{group:[\\w\\-]{1,20}}")]
async fn ws( req: HttpRequest,
    stream: web::Payload,info: web::Path<String>) -> impl Responder {
    let info = info.into_inner();
    println!("Hello, {:?}!\nYou are user \n", &info);
    let conn = WsConn {
        id: uniqid(),
        group:info,
    };
    actix_web_actors::ws::start(conn, &req, stream)
}

#[post("/stat/error_log/{group:[\\w\\-]{1,20}}")]
async fn error_log(bytes: web::Bytes,info:web::Path<String>) -> impl Responder {
    let info = info.into_inner();
    std::str::from_utf8(&bytes)
        .map_err(Error::from)
        .map(|name| format!("Hello, {} {}!\nYou are user \n",info, name))
}
