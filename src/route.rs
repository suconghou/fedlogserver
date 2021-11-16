use actix::{Actor, StreamHandler};
use actix_web::{get, post, web::Bytes, Error, HttpResponse, Responder};
use actix_web_actors::ws::{Message, ProtocolError, WebsocketContext};

pub struct WsConn {
    pub nick: String,
}

impl Actor for WsConn {
    type Context = WebsocketContext<Self>;
    /// 连接上
    fn started(&mut self, _: &mut Self::Context) {
        println!("{} join!", self.nick);
    }

    /// 断开连接
    fn stopped(&mut self, _: &mut Self::Context) {
        println!("{} exit!", self.nick);
    }
}

/// Handler for Message message
impl StreamHandler<Result<Message, ProtocolError>> for WsConn {
    fn handle(&mut self, msg: Result<Message, ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(Message::Ping(msg)) => ctx.pong(&msg),
            Ok(Message::Text(text)) => ctx.text(text),
            Ok(Message::Binary(bin)) => ctx.binary(bin),
            _ => (),
        }
    }
}

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

#[post("/stat/error_log/")]
async fn error_log(bytes: Bytes) -> impl Responder {
    std::str::from_utf8(&bytes)
        .map_err(Error::from)
        .map(|name| format!("Hello, {}!\nYou are user \n", name))
}
